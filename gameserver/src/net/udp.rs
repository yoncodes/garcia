use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::net::UdpSocket;

use super::{Kcp, app::AppState, session::Session};

const SESSION_TTL: Duration = Duration::from_secs(30);

pub async fn run(address: &str, state: Arc<AppState>) -> std::io::Result<()> {
    let socket = UdpSocket::bind(address).await?;
    let started = Instant::now();
    let mut sessions: HashMap<(SocketAddr, u32), Session> = HashMap::new();
    let mut update = tokio::time::interval(Duration::from_millis(10));
    let mut sweep = tokio::time::interval(Duration::from_secs(10));
    update.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    sweep.tick().await;
    let mut buffer = [0u8; 2048];

    tracing::info!(%address, "game server listening over KCP/UDP");
    loop {
        tokio::select! {
            received = socket.recv_from(&mut buffer) => {
                let (length, peer) = received?;
                let bytes = &buffer[..length];
                let Some(conv) = Kcp::conversation(bytes) else {
                    tracing::debug!(%peer, length, "ignored short UDP datagram");
                    continue;
                };
                let now = elapsed_ms(started);
                let session_key = (peer, conv);
                let (output, player_uid) = {
                    let session = sessions.entry(session_key).or_insert_with(|| {
                        tracing::info!(%peer, conv, "KCP session established");
                        Session::new(conv, state.clone())
                    });
                    let output = session.receive(bytes, peer, now).await;
                    (output, session.player_uid())
                };
                send_datagrams(&socket, peer, output).await;

                if let Some(uid) = player_uid {
                    sessions.retain(|key, session| {
                        let keep = *key == session_key || session.player_uid() != Some(uid);
                        if !keep {
                            tracing::info!(uid, old_peer = %key.0, old_conv = key.1, %peer, conv, "replaced stale player session");
                        }
                        keep
                    });
                }

                if sessions.get(&session_key).is_some_and(Session::is_dead) {
                    let uid = sessions.get(&session_key).and_then(Session::player_uid);
                    sessions.remove(&session_key);
                    tracing::info!(?uid, %peer, conv, "KCP session disconnected");
                }
            }
            _ = update.tick() => {
                let now = elapsed_ms(started);
                let mut output = Vec::new();
                for ((peer, _), session) in &mut sessions {
                    output.extend(session.update(now).await.into_iter().map(|bytes| (*peer, bytes)));
                }
                for (peer, bytes) in output {
                    if let Err(error) = socket.send_to(&bytes, peer).await {
                        tracing::warn!(%peer, %error, "UDP send failed");
                    }
                }
                sessions.retain(|(peer, conv), session| {
                    let keep = !session.is_dead();
                    if !keep {
                        tracing::info!(uid = ?session.player_uid(), %peer, %conv, "KCP session disconnected");
                    }
                    keep
                });
            }
            _ = sweep.tick() => {
                let now = Instant::now();
                sessions.retain(|(peer, conv), session| {
                    let keep = !session.is_expired(now, SESSION_TTL);
                    if !keep {
                        tracing::info!(uid = ?session.player_uid(), %peer, %conv, "KCP session timed out");
                    }
                    keep
                });
            }
        }
    }
}

async fn send_datagrams(socket: &UdpSocket, peer: SocketAddr, datagrams: Vec<Vec<u8>>) {
    for datagram in datagrams {
        if let Err(error) = socket.send_to(&datagram, peer).await {
            tracing::warn!(%peer, %error, "UDP send failed");
        }
    }
}

fn elapsed_ms(started: Instant) -> u32 {
    started.elapsed().as_millis() as u32
}
