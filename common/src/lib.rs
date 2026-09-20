use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub mod config;
pub mod network;
pub mod time;

static CONFIG: OnceLock<config::ServerConfig> = OnceLock::new();

pub fn init_config(config: config::ServerConfig) {
    CONFIG.get_or_init(|| config);
}

pub fn config() -> &'static config::ServerConfig {
    CONFIG.get().expect("config not initialized")
}

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    let _ = tracing_subscriber::fmt().try_init();
}

pub fn load_config() -> anyhow::Result<config::ServerConfig> {
    let path = config_path();
    let mut config = config::ServerConfig::load_or_create(&path)?;
    config.resolve_paths(path.parent().unwrap_or_else(|| Path::new(".")));
    config.validate_paths()?;
    Ok(config)
}

fn config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("GARCIA_CONFIG") {
        return path.into();
    }

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(path) = workspace_config(&current_dir) {
        return path;
    }
    if let Some(path) = current_dir
        .ancestors()
        .map(|directory| directory.join("config.toml"))
        .find(|path| path.is_file())
    {
        return path;
    }

    if let Some(path) = std::env::current_exe().ok().and_then(|exe| {
        exe.ancestors()
            .map(|directory| directory.join("config.toml"))
            .find(|path| path.is_file())
    }) {
        return path;
    }

    current_dir.join("config.toml")
}

fn workspace_config(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|directory| {
        let config = directory.join("config.toml");
        (directory.join("Cargo.toml").is_file() && config.is_file()).then_some(config)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_config_ignores_stale_build_directory_config() {
        let root = std::env::temp_dir().join(format!("garcia-workspace-{}", std::process::id()));
        let release = root.join("target/release");
        std::fs::create_dir_all(&release).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[workspace]").unwrap();
        std::fs::write(root.join("config.toml"), "root").unwrap();
        std::fs::write(release.join("config.toml"), "stale").unwrap();

        assert_eq!(workspace_config(&release), Some(root.join("config.toml")));

        std::fs::remove_dir_all(root).unwrap();
    }
}
