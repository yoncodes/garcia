use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum::{Router, routing::get};
use tokio::fs;
use tracing::info;

mod cache;
mod web;

#[derive(Clone)]
struct AppState {
    assets: PathBuf,
    upstream: String,
    client: reqwest::Client,
    refresh: Arc<tokio::sync::Mutex<()>>,
    version: Arc<RwLock<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::init_tracing();
    let config = common::load_config()?;
    common::init_config(config.clone());
    let listen = std::env::var("GARCIA_HOTPATCH_LISTEN").unwrap_or(config.hotpatch.listen);
    let assets = std::env::var_os("GARCIA_ASSETS")
        .map(PathBuf::from)
        .unwrap_or(config.paths.assets);
    let upstream = std::env::var("GARCIA_HOTPATCH_UPSTREAM")
        .unwrap_or(config.hotpatch.upstream)
        .trim_end_matches('/')
        .to_owned();
    fs::create_dir_all(&assets).await?;
    cache::remove_stale_downloads(&assets)?;
    let version = cache::latest_cached_version(&assets).unwrap_or_else(|| "unversioned".into());

    let state = AppState {
        assets: assets.canonicalize()?,
        upstream,
        client: reqwest::Client::builder().build()?,
        refresh: Arc::new(tokio::sync::Mutex::new(())),
        version: Arc::new(RwLock::new(version)),
    };
    let app = Router::new()
        .route("/prod/en/Android/{*path}", get(web::serve_asset))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    info!(listen, assets = %state.assets.display(), upstream = %state.upstream, "hotpatch server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
