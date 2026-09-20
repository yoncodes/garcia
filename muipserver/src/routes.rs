use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{GmRequest, GmResponse};

static PANEL_HTML: &str = include_str!("../res/index.html");

#[derive(Clone)]
struct AppState {
    token: Arc<str>,
    gm_address: Arc<str>,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

#[derive(Deserialize)]
struct CommandBody {
    token: String,
    player_uid: i64,
    command: String,
}

#[derive(Deserialize)]
struct CollectionQuery {
    token: String,
    player_uid: i64,
}

#[derive(Deserialize)]
struct BannerBody {
    token: String,
    gacha_id: i32,
    enabled: bool,
}

#[derive(Deserialize)]
struct BossRushBody {
    token: String,
    event_id: i32,
    enabled: bool,
}

#[derive(Deserialize)]
struct TeleportBody {
    token: String,
    player_uid: i64,
    anchor_id: Option<String>,
    task_id: Option<i32>,
}

pub fn router(token: String, gm_address: String) -> Router {
    Router::new()
        .route("/", get(panel))
        .route("/api/status", get(status))
        .route("/api/players", get(players))
        .route("/api/collection", get(collection))
        .route("/api/teleport_locations", get(teleport_locations))
        .route("/api/teleport", post(teleport))
        .route(
            "/api/gacha_banners",
            get(gacha_banners).post(set_gacha_banner),
        )
        .route(
            "/api/boss_rush_seasons",
            get(boss_rush_seasons).post(set_boss_rush_season),
        )
        .route("/api/run_gm_cmd", post(run_command))
        .with_state(AppState {
            token: Arc::from(token),
            gm_address: Arc::from(gm_address),
        })
}

async fn teleport_locations(
    State(state): State<AppState>,
    Query(query): Query<CollectionQuery>,
) -> Response {
    forward(
        &state,
        &query.token,
        GmRequest::TeleportLocations {
            player_uid: query.player_uid,
        },
    )
    .await
}

async fn teleport(State(state): State<AppState>, Json(body): Json<TeleportBody>) -> Response {
    let request = match (body.anchor_id, body.task_id) {
        (Some(anchor_id), None) => GmRequest::Teleport {
            player_uid: body.player_uid,
            anchor_id,
        },
        (None, Some(task_id)) => GmRequest::TeleportMission {
            player_uid: body.player_uid,
            task_id,
        },
        _ => return (StatusCode::BAD_REQUEST, "choose one teleport destination").into_response(),
    };
    forward(&state, &body.token, request).await
}

async fn gacha_banners(State(state): State<AppState>, Query(query): Query<TokenQuery>) -> Response {
    forward(&state, &query.token, GmRequest::GachaBanners).await
}

async fn set_gacha_banner(State(state): State<AppState>, Json(body): Json<BannerBody>) -> Response {
    forward(
        &state,
        &body.token,
        GmRequest::SetGachaBanner {
            gacha_id: body.gacha_id,
            enabled: body.enabled,
        },
    )
    .await
}

async fn boss_rush_seasons(
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Response {
    forward(&state, &query.token, GmRequest::BossRushSeasons).await
}

async fn set_boss_rush_season(
    State(state): State<AppState>,
    Json(body): Json<BossRushBody>,
) -> Response {
    forward(
        &state,
        &body.token,
        GmRequest::SetBossRushSeason {
            event_id: body.event_id,
            enabled: body.enabled,
        },
    )
    .await
}

async fn panel(State(state): State<AppState>) -> Html<String> {
    let token = serde_json::to_string(&*state.token).unwrap();
    Html(PANEL_HTML.replace("__MUIP_TOKEN__", &token))
}

async fn status(State(state): State<AppState>, Query(query): Query<TokenQuery>) -> Response {
    forward(&state, &query.token, GmRequest::Status).await
}

async fn players(State(state): State<AppState>, Query(query): Query<TokenQuery>) -> Response {
    forward(&state, &query.token, GmRequest::ListPlayers).await
}

async fn collection(
    State(state): State<AppState>,
    Query(query): Query<CollectionQuery>,
) -> Response {
    forward(
        &state,
        &query.token,
        GmRequest::Collection {
            player_uid: query.player_uid,
        },
    )
    .await
}

async fn run_command(State(state): State<AppState>, Json(body): Json<CommandBody>) -> Response {
    let request = GmRequest::Execute {
        player_uid: body.player_uid,
        command: body.command,
    };
    forward(&state, &body.token, request).await
}

async fn forward(state: &AppState, token: &str, request: GmRequest) -> Response {
    if token != &*state.token {
        return (
            StatusCode::UNAUTHORIZED,
            Json(GmResponse::error(401, "invalid MUIP token")),
        )
            .into_response();
    }

    match send_gm(&state.gm_address, request).await {
        Ok(response) => {
            let status = if response.retcode == 0 {
                StatusCode::OK
            } else {
                StatusCode::from_u16(response.retcode as u16)
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            (status, Json(response)).into_response()
        }
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(GmResponse::error(
                503,
                format!("GM bridge unavailable: {error}"),
            )),
        )
            .into_response(),
    }
}

async fn send_gm(address: &str, request: GmRequest) -> anyhow::Result<GmResponse> {
    let mut stream = TcpStream::connect(address).await?;
    let mut payload = serde_json::to_vec(&request)?;
    payload.push(b'\n');
    stream.write_all(&payload).await?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response).await?;
    Ok(serde_json::from_str(response.trim())?)
}
