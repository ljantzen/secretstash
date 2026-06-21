use anyhow::{Result, anyhow};

use crate::{config, crypto, db::Db, session};

const MIN_PASSWORD_LEN: usize = 12;

pub fn login(db_path: &std::path::Path, session_timeout_minutes: u64) -> Result<()> {
    let salt_path = config::salt_path_for_db(db_path);
    let is_new_vault = !salt_path.exists();

    if is_new_vault && db_path.exists() {
        return Err(anyhow!(
            "This vault was created with an older version of stash. \
             Run 'stash migrate' to convert it to the new encrypted format."
        ));
    }

    let password = if is_new_vault {
        let pw = rpassword::prompt_password("Create master password: ")?;
        if pw.len() < MIN_PASSWORD_LEN {
            return Err(anyhow!(
                "Master password must be at least {} characters",
                MIN_PASSWORD_LEN
            ));
        }
        let confirm = rpassword::prompt_password("Confirm master password: ")?;
        if pw != confirm {
            return Err(anyhow!("Passwords do not match"));
        }
        pw
    } else {
        rpassword::prompt_password("Master password: ")?
    };

    let salt = if is_new_vault {
        let s = crypto::generate_salt();
        config::write_salt_file(&salt_path, &s)?;
        s
    } else {
        std::fs::read_to_string(&salt_path)?.trim().to_string()
    };

    let key = crypto::derive_key(&password, &salt)?;

    // Opening the DB verifies the key implicitly: wrong key → error on first query.
    let _db = Db::open(db_path, &key).map_err(|_| anyhow!("Incorrect password"))?;

    if is_new_vault {
        println!("New vault created.");
    }

    let used_keychain = session::save_key(&key, session_timeout_minutes)?;
    let storage = if used_keychain { " (keychain)" } else { "" };
    if session_timeout_minutes == 0 {
        println!("Logged in. Session does not expire.{}", storage);
    } else {
        println!(
            "Logged in. Session expires in {} minute(s).{}",
            session_timeout_minutes, storage
        );
    }
    Ok(())
}

pub fn status() -> Result<()> {
    match crate::session::get_status() {
        Err(e) => {
            println!("{}", e);
        }
        Ok(info) => {
            println!("Logged in.");
            if info.expiry_secs == u64::MAX {
                println!("Session does not expire (timeout: 0).");
            } else {
                let remaining_secs = info.expiry_secs.saturating_sub(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                println!(
                    "{} (timeout: {} minute(s)).",
                    format_remaining(remaining_secs),
                    info.timeout_minutes
                );
            }
            let storage = if info.via_keychain {
                "keychain"
            } else {
                "session file"
            };
            println!("Stored in {}.", storage);
        }
    }
    Ok(())
}

fn format_remaining(secs: u64) -> String {
    if secs < 60 {
        return "Expires in less than a minute.".to_string();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("Expires in {} minute(s).", mins);
    }
    let hours = mins / 60;
    let mins_rem = mins % 60;
    if mins_rem == 0 {
        format!("Expires in {} hour(s).", hours)
    } else {
        format!("Expires in {} hour(s) {} minute(s).", hours, mins_rem)
    }
}

pub fn logout() -> Result<()> {
    session::clear_key()?;
    println!("Logged out.");
    Ok(())
}

pub fn reset(db_path: &std::path::Path) -> Result<()> {
    let salt_path = config::salt_path_for_db(db_path);

    let old_salt = std::fs::read_to_string(&salt_path)
        .map(|s| s.trim().to_string())
        .map_err(|_| anyhow!("No vault found — run 'stash auth login' first"))?;

    let old_password = rpassword::prompt_password("Current master password: ")?;
    let old_key = crypto::derive_key(&old_password, &old_salt)?;

    let db = Db::open(db_path, &old_key).map_err(|_| anyhow!("Incorrect password"))?;

    let new_password = rpassword::prompt_password("New master password: ")?;
    if new_password.len() < MIN_PASSWORD_LEN {
        return Err(anyhow!(
            "Master password must be at least {} characters",
            MIN_PASSWORD_LEN
        ));
    }
    let confirm = rpassword::prompt_password("Confirm new master password: ")?;
    if new_password != confirm {
        return Err(anyhow!("Passwords do not match"));
    }

    let new_salt = crypto::generate_salt();
    let new_key = crypto::derive_key(&new_password, &new_salt)?;

    db.rekey(&new_key)?;
    config::write_salt_file(&salt_path, &new_salt)?;

    session::clear_key()?;
    println!("Master password changed. Run 'stash auth login' to start a new session.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn format_remaining_sub_minute() {
        assert_eq!(format_remaining(30), "Expires in less than a minute.");
        assert_eq!(format_remaining(0), "Expires in less than a minute.");
    }

    #[test]
    fn format_remaining_minutes() {
        assert_eq!(format_remaining(60), "Expires in 1 minute(s).");
        assert_eq!(format_remaining(2700), "Expires in 45 minute(s).");
    }

    #[test]
    fn format_remaining_exact_hours() {
        assert_eq!(format_remaining(3600), "Expires in 1 hour(s).");
        assert_eq!(format_remaining(7200), "Expires in 2 hour(s).");
    }

    #[test]
    fn format_remaining_hours_and_minutes() {
        assert_eq!(format_remaining(3660), "Expires in 1 hour(s) 1 minute(s).");
        assert_eq!(format_remaining(5400), "Expires in 1 hour(s) 30 minute(s).");
    }

    /// `login()` must reject a vault that has a DB file but no salt file
    /// (old field-level-encrypted format) BEFORE attempting any TTY prompt.
    /// It must also leave no salt file behind after rejecting.
    #[test]
    fn login_detects_old_format_vault_before_any_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");

        // Create a plain SQLite file (old format) — no salt file.
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE items (id INTEGER PRIMARY KEY);")
            .unwrap();
        drop(conn);

        let err = login(&db_path, 0).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("stash migrate"),
            "expected 'stash migrate' hint, got: {msg}"
        );
        assert!(
            !config::salt_path_for_db(&db_path).exists(),
            "salt file must not be created when rejecting an old-format vault"
        );
    }
}
