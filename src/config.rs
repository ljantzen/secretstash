use anyhow::{Result, anyhow};
use std::path::PathBuf;

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
