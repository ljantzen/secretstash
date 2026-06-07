use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn load_key() -> Result<[u8; 32]> {
    let path = crate::config::session_path()?;
    let content = fs::read_to_string(&path)
        .map_err(|_| anyhow!("Not authenticated. Run 'stash auth login' first."))?;
    parse_session(content.trim())
}

pub fn save_key(key: &[u8; 32], timeout_minutes: u64) -> Result<()> {
    let path = crate::config::session_path()?;
    let expiry = now_secs() + timeout_minutes * 60;
    fs::write(&path, format!("{}\n{}", expiry, B64.encode(key)))?;
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn parse_session(content: &str) -> Result<[u8; 32]> {
    let corrupt = || anyhow!("Corrupt session file. Run 'stash auth login' again.");
    let mut lines = content.lines();
    let expiry: u64 = lines
        .next()
        .ok_or_else(corrupt)?
        .parse()
        .map_err(|_| corrupt())?;
    let key_b64 = lines.next().ok_or_else(corrupt)?;

    if now_secs() > expiry {
        // Clean up expired session
        let _ = crate::config::session_path().map(fs::remove_file);
        return Err(anyhow!("Session expired. Run 'stash auth login' again."));
    }

    let bytes = B64.decode(key_b64).map_err(|_| corrupt())?;
    if bytes.len() != 32 {
        return Err(corrupt());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(expiry_offset_secs: i64, key: &[u8; 32]) -> String {
        let expiry = (now_secs() as i64 + expiry_offset_secs) as u64;
        format!("{}\n{}", expiry, B64.encode(key))
    }

    #[test]
    fn valid_session_parsed() {
        let key = [0x42u8; 32];
        let content = make_session(900, &key);
        assert_eq!(parse_session(&content).unwrap(), key);
    }

    #[test]
    fn expired_session_rejected() {
        let key = [0x42u8; 32];
        let content = make_session(-1, &key);
        let err = parse_session(&content).unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn missing_key_line_rejected() {
        let expiry = now_secs() + 900;
        let err = parse_session(&expiry.to_string()).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn bad_base64_rejected() {
        let expiry = now_secs() + 900;
        let content = format!("{}\nnot!base64!!!", expiry);
        assert!(parse_session(&content).is_err());
    }

    #[test]
    fn wrong_key_length_rejected() {
        let expiry = now_secs() + 900;
        let content = format!("{}\n{}", expiry, B64.encode([0u8; 16]));
        let err = parse_session(&content).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn non_numeric_expiry_rejected() {
        let err = parse_session("not-a-number\nYWJj").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }
}
