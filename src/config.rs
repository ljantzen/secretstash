use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

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

/// Returns the salt file path that corresponds to a given database path.
/// The salt file sits alongside the database with a `.salt` extension.
pub fn salt_path_for_db(db_path: &Path) -> PathBuf {
    db_path.with_extension("salt")
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

/// Write the Argon2id salt to a file with mode 0600 on Unix.
/// The file is atomically replaced (remove + create_new) to avoid
/// a window where it exists with permissive permissions.
#[cfg(unix)]
pub fn write_salt_file(path: &Path, salt: &str) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    writeln!(f, "{}", salt)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn write_salt_file(path: &Path, salt: &str) -> Result<()> {
    std::fs::write(path, format!("{}\n", salt))?;
    Ok(())
}

/// Pre-create `path` as an empty file with mode 0600 if it does not yet exist,
/// so that a program which would otherwise create it world-readable (e.g.
/// SQLite creating a fresh database) never gets the chance. No-op on non-Unix
/// and when the file already exists. Best effort: errors are ignored.
#[cfg(unix)]
pub fn precreate_private(path: &Path) {
    use std::os::unix::fs::OpenOptionsExt;

    if !path.exists() {
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path);
    }
}

#[cfg(not(unix))]
pub fn precreate_private(_path: &Path) {}

/// Tighten permissions to 0600 on a database file and its WAL/SHM sidecars,
/// which SQLite may create with the process umask. Best effort: missing files
/// and errors are ignored. No-op on non-Unix.
#[cfg(unix)]
pub fn restrict_db_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    for suffix in ["", "-wal", "-shm"] {
        let mut p = path.as_os_str().to_os_string();
        p.push(suffix);
        let p = PathBuf::from(p);
        if p.exists() {
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
    }
}

#[cfg(not(unix))]
pub fn restrict_db_permissions(_path: &Path) {}

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

    #[test]
    fn salt_path_for_db_replaces_extension() {
        let db = Path::new("/home/user/.local/share/stash/stash.db");
        assert_eq!(
            salt_path_for_db(db),
            PathBuf::from("/home/user/.local/share/stash/stash.salt")
        );
    }
}
