mod routes;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GmRequest {
    Status,
    ListPlayers,
    Collection { player_uid: i64 },
    TeleportLocations { player_uid: i64 },
    Teleport { player_uid: i64, anchor_id: String },
    TeleportMission { player_uid: i64, task_id: i32 },
    GachaBanners,
    SetGachaBanner { gacha_id: i32, enabled: bool },
    BossRushSeasons,
    SetBossRushSeason { event_id: i32, enabled: bool },
    Execute { player_uid: i64, command: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GmResponse {
    pub retcode: i32,
    pub message: String,
    #[serde(default)]
    pub online: usize,
    #[serde(default)]
    pub players: Vec<i64>,
    #[serde(default)]
    pub changed: usize,
    #[serde(default)]
    pub reconnect_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl GmResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Self::default()
        }
    }

    pub fn error(retcode: i32, message: impl Into<String>) -> Self {
        Self {
            retcode,
            message: message.into(),
            ..Self::default()
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    common::init_tracing();
    let config = common::load_config()?;
    common::init_config(config.clone());
    let listener = TcpListener::bind(&config.muip.listen)
        .await
        .with_context(|| format!("failed to bind MUIP server on {}", config.muip.listen))?;
    tracing::info!(address = %config.muip.listen, "MUIP HTTP server listening");
    axum::serve(
        listener,
        routes::router(config.muip.token, config.muip.gm_listen),
    )
    .await?;
    Ok(())
}
