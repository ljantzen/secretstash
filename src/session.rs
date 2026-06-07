use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn load_key() -> Result<[u8; 32]> {
    let path = crate::config::session_path()?;
    let content = fs::read_to_string(&path)
        .map_err(|_| anyhow!("Not authenticated. Run 'stash auth login' first."))?;
    let bytes = B64.decode(content.trim())?;
    if bytes.len() != 32 {
        return Err(anyhow!(
            "Corrupt session file. Run 'stash auth login' again."
        ));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

pub fn save_key(key: &[u8; 32]) -> Result<()> {
    let path = crate::config::session_path()?;
    fs::write(&path, B64.encode(key))?;
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

pub fn clear_key() -> Result<()> {
    let path = crate::config::session_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
