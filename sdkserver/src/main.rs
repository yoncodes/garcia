use tracing::info;

mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::init_tracing();
    let config = common::load_config()?;
    common::init_config(config.clone());
    let listen = std::env::var("GARCIA_SDK_LISTEN").unwrap_or(config.sdk.listen);
    let capture_upstream = std::env::var("GARCIA_CAPTURE_SDK_UPSTREAM").ok();
    let database = database::connect_to(&config.database.path).await?;
    database::run_migrations(&database).await?;
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    info!(
        listen,
        capture_upstream = capture_upstream.as_deref().unwrap_or("disabled"),
        database = %config.database.path.display(),
        "SDK server listening"
    );
    axum::serve(
        listener,
        handlers::router(database).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}
