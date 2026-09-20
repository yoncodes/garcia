use std::{io, path::PathBuf};

use tokio::net::{UdpSocket, lookup_host};

use crate::{Direction, inspector::Inspector};

pub(crate) async fn run(address: &str, upstream: &str, output: PathBuf) -> io::Result<()> {
    let upstream = lookup_host(upstream)
        .await?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "upstream not found"))?;
    let upstream_bind = if upstream.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };

    let client_socket = UdpSocket::bind(address).await?;
    let upstream_socket = UdpSocket::bind(upstream_bind).await?;
    upstream_socket.connect(upstream).await?;

    tracing::info!(%address, %upstream, path = %output.display(), "packet capture proxy listening over UDP");

    let mut client = None;
    let mut inspector = Inspector::new(&output);
    let mut client_buffer = [0_u8; 65_535];
    let mut upstream_buffer = [0_u8; 65_535];

    loop {
        tokio::select! {
            received = client_socket.recv_from(&mut client_buffer) => {
                let (length, peer) = received?;
                if client != Some(peer) {
                    tracing::info!(%peer, "capture client connected");
                    client = Some(peer);
                    inspector = Inspector::new(&output);
                }
                let bytes = &client_buffer[..length];
                upstream_socket.send(bytes).await?;
                inspector.observe(Direction::ClientToGateway, bytes);
            }
            received = upstream_socket.recv(&mut upstream_buffer) => {
                let length = received?;
                if let Some(peer) = client {
                    let bytes = &upstream_buffer[..length];
                    client_socket.send_to(bytes, peer).await?;
                    inspector.observe(Direction::GatewayToClient, bytes);
                }
            }
        }
    }
}
