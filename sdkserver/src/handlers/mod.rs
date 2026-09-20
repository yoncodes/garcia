use std::{net::SocketAddr, time::Instant};

use axum::{
    Router,
    extract::{ConnectInfo, Request},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};

mod open_notices;
mod quick_sdk_login;

pub fn router(database: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/open_notices", get(open_notices::list))
        .route("/internal_notices", get(open_notices::list))
        .route("/quickSDKlogin", post(quick_sdk_login::login))
        .layer(middleware::from_fn(log_request))
        .with_state(database)
}

async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let client = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|peer| peer.0.to_string())
        .unwrap_or_else(|| "unknown".into());
    let started = Instant::now();
    let response = next.run(request).await;

    tracing::info!(
        %client,
        %method,
        %path,
        status = response.status().as_u16(),
        elapsed_ms = started.elapsed().as_millis() as u64,
        "SDK request completed"
    );
    response
}
