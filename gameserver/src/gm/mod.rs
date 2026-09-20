mod boss_rush;
mod commands;
mod gacha;

use std::sync::Arc;

use muipserver::{GmRequest, GmResponse};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::net::{app::AppState, gm_command::GmCommand};

pub async fn run(address: &str, state: Arc<AppState>) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "MUIP GM bridge listening");
    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = handle(stream, state).await {
                tracing::warn!(%error, "MUIP GM request failed");
            }
        });
    }
}

async fn handle(stream: TcpStream, state: Arc<AppState>) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut line = String::new();
    BufReader::new(reader).read_line(&mut line).await?;
    let response = match serde_json::from_str::<GmRequest>(line.trim()) {
        Ok(request) => dispatch(&state, request).await,
        Err(error) => GmResponse::error(400, format!("invalid GM request: {error}")),
    };
    let mut payload = serde_json::to_vec(&response).map_err(std::io::Error::other)?;
    payload.push(b'\n');
    writer.write_all(&payload).await
}

async fn dispatch(state: &AppState, request: GmRequest) -> GmResponse {
    match request {
        GmRequest::Status => online_status(state, "online"),
        GmRequest::ListPlayers => {
            let players = state.online_players();
            let message = format!("{} player(s) online", players.len());
            GmResponse {
                online: players.len(),
                players,
                message,
                ..Default::default()
            }
        }
        GmRequest::Collection { player_uid } => {
            commands::send(state, player_uid, GmCommand::Collection).await
        }
        GmRequest::TeleportLocations { player_uid } => {
            commands::send(state, player_uid, GmCommand::TeleportLocations).await
        }
        GmRequest::Teleport {
            player_uid,
            anchor_id,
        } => commands::send(state, player_uid, GmCommand::Teleport(anchor_id)).await,
        GmRequest::TeleportMission {
            player_uid,
            task_id,
        } => commands::send(state, player_uid, GmCommand::TeleportMission(task_id)).await,
        GmRequest::GachaBanners => gacha::list(state).await,
        GmRequest::SetGachaBanner { gacha_id, enabled } => {
            gacha::set_enabled(state, gacha_id, enabled).await
        }
        GmRequest::BossRushSeasons => boss_rush::list(state).await,
        GmRequest::SetBossRushSeason { event_id, enabled } => {
            boss_rush::set_enabled(state, event_id, enabled).await
        }
        GmRequest::Execute {
            player_uid,
            command,
        } => commands::execute(state, player_uid, &command).await,
    }
}

fn online_status(state: &AppState, message: &str) -> GmResponse {
    let players = state.online_players();
    GmResponse {
        online: players.len(),
        players,
        ..GmResponse::ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dispatches_commands_to_the_live_player_session() {
        let config = common::load_config().unwrap();
        let tables = configs::GameTables::load(&config.paths.game_tables).unwrap();
        let state = AppState::new(database::connect_memory().await.unwrap(), tables);
        let (session_tx, mut session_rx) = tokio::sync::mpsc::unbounded_channel();
        state.register_session(7, session_tx);
        tokio::spawn(async move {
            let envelope = session_rx.recv().await.unwrap();
            assert!(matches!(envelope.command, GmCommand::GrantHeroes));
            envelope
                .response
                .send(Ok(crate::net::gm_command::GmOutcome {
                    message: "granted 23 heroes".into(),
                    changed: 23,
                    reconnect_required: true,
                    data: None,
                }))
                .unwrap();
        });

        let response = commands::execute(&state, 7, "hero grant all").await;
        assert_eq!(response.retcode, 0);
        assert_eq!(response.changed, 23);
        assert!(response.reconnect_required);
    }
}
