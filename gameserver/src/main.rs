mod gm;
mod handlers;
mod net;

pub use logic;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::init_tracing();
    let config = common::load_config()?;
    common::init_config(config.clone());
    let address = std::env::var("GARCIA_LISTEN").unwrap_or_else(|_| config.server.listen.clone());
    let database = database::connect_to(&config.database.path).await?;
    database::run_migrations(&database).await?;
    tracing::info!(path = %config.database.path.display(), "database ready");
    tracing::info!(path = %config.paths.game_tables.display(), "loading game tables");
    let tables = configs::GameTables::load(&config.paths.game_tables)?;
    let state = std::sync::Arc::new(net::app::AppState::new(database, tables));
    if config.muip.enabled {
        let gm_address = config.muip.gm_listen.clone();
        let gm_state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = gm::run(&gm_address, gm_state).await {
                tracing::error!(%error, "MUIP GM bridge stopped");
            }
        });
    }
    net::udp::run(&address, state).await?;
    Ok(())
}
