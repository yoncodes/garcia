mod capture;
mod inspector;
mod key_recovery;
mod proxy;

use std::path::PathBuf;

#[derive(Clone, Copy)]
pub(crate) enum Direction {
    ClientToGateway,
    GatewayToClient,
}

impl Direction {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ClientToGateway => "c2g",
            Self::GatewayToClient => "g2c",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::init_tracing();
    let config = common::load_config()?;
    common::init_config(config.clone());
    let listen = std::env::var("GARCIA_CAPTURE_LISTEN").unwrap_or(config.packetcapture.listen);
    let upstream =
        std::env::var("GARCIA_CAPTURE_UPSTREAM").unwrap_or(config.packetcapture.upstream);
    anyhow::ensure!(
        !upstream.trim().is_empty(),
        "packet capture upstream is empty"
    );
    let output = std::env::var_os("GARCIA_CAPTURE_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or(config.packetcapture.output);

    proxy::run(&listen, &upstream, output).await?;
    Ok(())
}
