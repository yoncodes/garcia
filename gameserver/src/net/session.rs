use std::{net::SocketAddr, sync::Arc, time::Instant};

use super::gm_command::GmCommandEnvelope;
use super::{Kcp, app::AppState, connection::GatewaySession};
use tokio::sync::mpsc;

pub(super) struct Session {
    conv: u32,
    kcp: Kcp,
    gateway: GatewaySession,
    gm_rx: mpsc::UnboundedReceiver<GmCommandEnvelope>,
    last_seen: Instant,
}

impl Session {
    pub fn new(conv: u32, state: Arc<AppState>) -> Self {
        let (gm_tx, gm_rx) = mpsc::unbounded_channel();
        Self {
            conv,
            kcp: Kcp::new(conv),
            gateway: GatewaySession::new(state, gm_tx),
            gm_rx,
            last_seen: Instant::now(),
        }
    }

    pub async fn receive(&mut self, bytes: &[u8], peer: SocketAddr, now: u32) -> Vec<Vec<u8>> {
        self.last_seen = Instant::now();
        if let Err(error) = self.kcp.input(bytes, now) {
            tracing::debug!(%peer, conv = self.conv, ?error, "invalid KCP datagram");
        }
        while let Some(message) = self.kcp.recv() {
            match self.gateway.handle_packet(&message).await {
                Ok(responses) => {
                    for response in responses {
                        if let Err(error) = self.kcp.send(&response, now) {
                            tracing::warn!(%peer, conv = self.conv, %error, "gateway response rejected");
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(%peer, conv = self.conv, %error, "application packet failed");
                }
            }
        }
        self.kcp.take_output()
    }

    pub async fn update(&mut self, now: u32) -> Vec<Vec<u8>> {
        while let Ok(envelope) = self.gm_rx.try_recv() {
            let result = self.gateway.handle_gm_command(envelope.command).await;
            match result {
                Ok((outcome, packets)) => {
                    let mut error = None;
                    for packet in packets {
                        if let Err(send_error) = self.kcp.send(&packet, now) {
                            error = Some(send_error.to_string());
                            break;
                        }
                    }
                    let _ = envelope.response.send(error.map_or(Ok(outcome), Err));
                }
                Err(error) => {
                    let _ = envelope.response.send(Err(error.to_string()));
                }
            }
        }
        match self.gateway.poll_periodic().await {
            Ok(packets) => {
                for packet in packets {
                    if let Err(error) = self.kcp.send(&packet, now) {
                        tracing::warn!(conv = self.conv, %error, "periodic gateway push rejected");
                    }
                }
            }
            Err(error) => {
                tracing::warn!(conv = self.conv, %error, "periodic gateway update failed");
            }
        }
        self.kcp.update(now);
        self.kcp.take_output()
    }

    pub fn is_dead(&self) -> bool {
        self.kcp.is_dead()
    }

    pub fn is_expired(&self, now: Instant, ttl: std::time::Duration) -> bool {
        now.duration_since(self.last_seen) >= ttl
    }

    pub fn player_uid(&self) -> Option<i64> {
        self.gateway.player_uid()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.gateway.unregister();
    }
}
