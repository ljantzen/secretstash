use anyhow::{Result, anyhow};
use std::path::PathBuf;

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub db: Option<PathBuf>,
    pub session_timeout_minutes: Option<u64>,
    pub browser: Option<String>,
}

pub fn data_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Cannot determine local data directory"))?
        .join("stash");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("stash.db"))
}

pub fn session_path() -> Result<PathBuf> {
    Ok(data_dir()?.join(".session"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow!("Cannot determine config directory"))?
        .join("stash")
        .join("stash.toml"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path)?;
    toml::from_str(&text).map_err(|e| anyhow!("Invalid config {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_toml_parses_to_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(cfg.db.is_none());
    }

    #[test]
    fn db_field_parsed() {
        let cfg: Config = toml::from_str(r#"db = "/tmp/test.db""#).unwrap();
        assert_eq!(
            cfg.db.as_deref(),
            Some(std::path::Path::new("/tmp/test.db"))
        );
    }

    #[test]
    fn session_timeout_parsed() {
        let cfg: Config = toml::from_str("session_timeout_minutes = 60").unwrap();
        assert_eq!(cfg.session_timeout_minutes, Some(60));
    }

    #[test]
    fn unknown_field_rejected() {
        assert!(toml::from_str::<Config>("unknown_key = true").is_err());
    }
}
