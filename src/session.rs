use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

pub fn load_key() -> Result<Zeroizing<[u8; 32]>> {
    let path = crate::config::session_path()?;
    let content = fs::read_to_string(&path)
        .map_err(|_| anyhow!("Not authenticated. Run 'stash auth login' first."))?;
    parse_session(content.trim())
}

pub fn save_key(key: &[u8; 32], timeout_minutes: u64) -> Result<()> {
    let path = crate::config::session_path()?;
    let expiry = if timeout_minutes == 0 {
        u64::MAX
    } else {
        now_secs() + timeout_minutes * 60
    };
    let content = Zeroizing::new(format!("{}\n{}", expiry, B64.encode(key)));
    write_session_file(&path, content.as_bytes())
}

pub fn clear_key() -> Result<()> {
    let path = crate::config::session_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// On Unix: delete any existing file then create with mode 0600 before writing,
// so there is no window in which the file exists with permissive permissions.
#[cfg(unix)]
fn write_session_file(path: &std::path::Path, content: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    if path.exists() {
        fs::remove_file(path)?;
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(content)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_session_file(path: &std::path::Path, content: &[u8]) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn parse_session(content: &str) -> Result<Zeroizing<[u8; 32]>> {
    let corrupt = || anyhow!("Corrupt session file. Run 'stash auth login' again.");
    let mut lines = content.lines();
    let expiry: u64 = lines
        .next()
        .ok_or_else(corrupt)?
        .parse()
        .map_err(|_| corrupt())?;
    let key_b64 = lines.next().ok_or_else(corrupt)?;

    if now_secs() > expiry {
        let _ = crate::config::session_path().map(fs::remove_file);
        return Err(anyhow!("Session expired. Run 'stash auth login' again."));
    }

    let bytes = Zeroizing::new(B64.decode(key_b64).map_err(|_| corrupt())?);
    if bytes.len() != 32 {
        return Err(corrupt());
    }
    let mut key = Zeroizing::new([0u8; 32]);
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
        assert_eq!(*parse_session(&content).unwrap(), key);
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

    #[test]
    fn zero_timeout_stores_max_expiry_and_never_expires() {
        // timeout_minutes=0 means no timeout: expiry is stored as u64::MAX
        let key = [0x11u8; 32];
        let content = format!("{}\n{}", u64::MAX, B64.encode(key));
        assert_eq!(*parse_session(&content).unwrap(), key);
    }
}
