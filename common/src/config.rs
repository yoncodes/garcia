use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const CONFIG_TEMPLATE: &str = include_str!("../Config.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub sdk: ServiceSettings,
    pub hotpatch: HotpatchSettings,
    #[serde(default)]
    pub packetcapture: PacketCaptureSettings,
    #[serde(default)]
    pub muip: MuipSettings,
    pub account_defaults: AccountDefaults,
    pub gameplay: GameplaySettings,
    pub paths: PathSettings,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub listen: String,
    pub assets_url: String,
    pub zone_offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSettings {
    pub listen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotpatchSettings {
    pub listen: String,
    pub upstream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketCaptureSettings {
    pub listen: String,
    pub upstream: String,
    pub output: PathBuf,
}

impl Default for PacketCaptureSettings {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:8888".into(),
            upstream: "43.166.151.130:8888".into(),
            output: "data/proxy".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuipSettings {
    pub enabled: bool,
    pub listen: String,
    pub token: String,
    pub gm_listen: String,
}

impl Default for MuipSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            listen: "127.0.0.1:18890".into(),
            token: "garcia".into(),
            gm_listen: "127.0.0.1:18891".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDefaults {
    #[serde(default)]
    pub nickname: String,
    pub level: i32,
    pub gameplay_id: i32,
    pub active_formation: i32,
    pub formation_key: i32,
    pub appear_skill_key: i32,
    pub permanent_profile_end_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    #[serde(default = "default_dungeon_player_exp_per_stamina")]
    pub dungeon_player_exp_per_stamina: i32,
    pub question_url: String,
}

fn default_dungeon_player_exp_per_stamina() -> i32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSettings {
    pub game_tables: PathBuf,
    pub assets: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

impl ServerConfig {
    pub fn load_or_create(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, CONFIG_TEMPLATE)?;
        }

        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn resolve_paths(&mut self, config_dir: &Path) {
        if self.paths.game_tables.is_relative() {
            self.paths.game_tables = config_dir.join(&self.paths.game_tables);
        }
        if self.paths.assets.is_relative() {
            self.paths.assets = config_dir.join(&self.paths.assets);
        }
        if self.packetcapture.output.is_relative() {
            self.packetcapture.output = config_dir.join(&self.packetcapture.output);
        }
        if self.database.path.is_relative() {
            self.database.path = config_dir.join(&self.database.path);
        }
    }

    pub fn validate_paths(&self) -> anyhow::Result<()> {
        if !self.paths.game_tables.is_dir() {
            anyhow::bail!(
                "game tables directory not found: {}",
                self.paths.game_tables.display()
            );
        }
        if let Some(parent) = self.database.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(&self.paths.assets)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_or_create_writes_default_config() {
        let directory = std::env::temp_dir().join(format!("garcia-config-{}", std::process::id()));
        let path = directory.join("config.toml");
        let config = ServerConfig::load_or_create(&path).unwrap();

        assert!(path.is_file());
        assert_eq!(config.server.listen, "0.0.0.0:8888");
        assert_eq!(config.gameplay.dungeon_player_exp_per_stamina, 10);
        assert_eq!(config.packetcapture.listen, "0.0.0.0:8888");
        assert_eq!(config.packetcapture.upstream, "43.166.151.130:8888");
        assert_eq!(config.packetcapture.output, PathBuf::from("data/proxy"));
        assert_eq!(
            config.gameplay.question_url,
            "https://www.wjx.cn/vm/OteiTGi.aspx#"
        );
        assert_eq!(config.paths.game_tables, PathBuf::from("data/tables"));
        assert_eq!(config.database.path, PathBuf::from("data/garcia.db"));

        std::fs::remove_dir_all(directory).unwrap();
    }
}
