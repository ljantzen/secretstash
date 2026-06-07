use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

/// Returns true if the session key was stored in the system keychain.
pub fn save_key(key: &[u8; 32], timeout_minutes: u64) -> Result<bool> {
    let expiry = if timeout_minutes == 0 {
        u64::MAX
    } else {
        now_secs() + timeout_minutes * 60
    };
    let content = encode_session(expiry, timeout_minutes, key);

    let used_keychain = crate::keychain::save(&content);

    let path = crate::config::session_path()?;
    write_session_file(&path, content.as_bytes())?;

    Ok(used_keychain)
}

pub fn load_key() -> Result<Zeroizing<[u8; 32]>> {
    // Try keychain first; clear it and fall through on any error.
    if let Some(content) = crate::keychain::load() {
        match parse_session(content.trim()) {
            Ok((key, timeout_minutes)) => {
                let _ = refresh_session(&key, timeout_minutes);
                return Ok(key);
            }
            Err(_) => {
                crate::keychain::clear();
            }
        }
    }

    // Fall back to session file.
    let path = crate::config::session_path()?;
    let content = fs::read_to_string(&path)
        .map_err(|_| anyhow!("Not authenticated. Run 'stash auth login' first."))?;
    let (key, timeout_minutes) = parse_session(content.trim())?;
    let _ = refresh_session(&key, timeout_minutes);
    Ok(key)
}

pub fn clear_key() -> Result<()> {
    crate::keychain::clear();
    let path = crate::config::session_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn refresh_session(key: &[u8; 32], timeout_minutes: u64) -> Result<()> {
    if timeout_minutes == 0 {
        return Ok(());
    }
    let expiry = now_secs() + timeout_minutes * 60;
    let content = encode_session(expiry, timeout_minutes, key);
    crate::keychain::save(&content);
    let path = crate::config::session_path()?;
    write_session_file(&path, content.as_bytes())
}

fn encode_session(expiry: u64, timeout_minutes: u64, key: &[u8; 32]) -> Zeroizing<String> {
    Zeroizing::new(format!(
        "{}\n{}\n{}",
        expiry,
        timeout_minutes,
        B64.encode(key)
    ))
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

fn parse_session(content: &str) -> Result<(Zeroizing<[u8; 32]>, u64)> {
    let corrupt = || anyhow!("Corrupt session file. Run 'stash auth login' again.");
    let mut lines = content.lines();
    let expiry: u64 = lines
        .next()
        .ok_or_else(corrupt)?
        .parse()
        .map_err(|_| corrupt())?;
    let timeout_minutes: u64 = lines
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
    Ok((key, timeout_minutes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(expiry_offset_secs: i64, timeout_minutes: u64, key: &[u8; 32]) -> String {
        let expiry = (now_secs() as i64 + expiry_offset_secs) as u64;
        format!("{}\n{}\n{}", expiry, timeout_minutes, B64.encode(key))
    }

    #[test]
    fn valid_session_parsed() {
        let key = [0x42u8; 32];
        let content = make_session(900, 15, &key);
        assert_eq!(*parse_session(&content).unwrap().0, key);
    }

    #[test]
    fn expired_session_rejected() {
        let key = [0x42u8; 32];
        let content = make_session(-1, 15, &key);
        let err = parse_session(&content).unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn missing_timeout_line_rejected() {
        let expiry = now_secs() + 900;
        let key = [0x42u8; 32];
        let content = format!("{}\n{}", expiry, B64.encode(key));
        let err = parse_session(&content).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn missing_key_line_rejected() {
        let expiry = now_secs() + 900;
        let err = parse_session(&format!("{}\n15", expiry)).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn bad_base64_rejected() {
        let expiry = now_secs() + 900;
        let content = format!("{}\n15\nnot!base64!!!", expiry);
        assert!(parse_session(&content).is_err());
    }

    #[test]
    fn wrong_key_length_rejected() {
        let expiry = now_secs() + 900;
        let content = format!("{}\n15\n{}", expiry, B64.encode([0u8; 16]));
        let err = parse_session(&content).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn non_numeric_expiry_rejected() {
        let err = parse_session("not-a-number\n15\nYWJj").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("corrupt"));
    }

    #[test]
    fn zero_timeout_stores_max_expiry_and_never_expires() {
        let key = [0x11u8; 32];
        let content = format!("{}\n0\n{}", u64::MAX, B64.encode(key));
        assert_eq!(*parse_session(&content).unwrap().0, key);
    }
}
