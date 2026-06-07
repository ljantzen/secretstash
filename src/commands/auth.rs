use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};

use crate::{crypto, db::Db, session};

const CANARY: &[u8] = b"stash-auth-canary-v1";
const MIN_PASSWORD_LEN: usize = 12;

pub fn login(db_path: &std::path::Path, session_timeout_minutes: u64) -> Result<()> {
    let db = Db::open(db_path)?;

    let is_new_vault = db.get_meta("salt")?.is_none();

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
        db.set_meta("salt", &s)?;
        s
    } else {
        db.get_meta("salt")?.unwrap()
    };

    let key = crypto::derive_key(&password, &salt)?;

    if is_new_vault {
        let (enc, nonce) = crypto::encrypt(&key, CANARY)?;
        db.set_meta("canary_enc", &B64.encode(&enc))?;
        db.set_meta("canary_nonce", &B64.encode(&nonce))?;
        println!("New vault created.");
    } else {
        let enc = B64.decode(
            db.get_meta("canary_enc")?
                .ok_or_else(|| anyhow!("Vault metadata corrupted"))?,
        )?;
        let nonce = B64.decode(
            db.get_meta("canary_nonce")?
                .ok_or_else(|| anyhow!("Vault metadata corrupted"))?,
        )?;
        let plaintext =
            crypto::decrypt(&key, &enc, &nonce).map_err(|_| anyhow!("Incorrect password"))?;
        if plaintext.as_slice() != CANARY {
            return Err(anyhow!("Incorrect password"));
        }
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

pub fn logout() -> Result<()> {
    session::clear_key()?;
    println!("Logged out.");
    Ok(())
}

pub fn reset(db_path: &std::path::Path) -> Result<()> {
    let db = Db::open(db_path)?;

    let old_salt = db
        .get_meta("salt")?
        .ok_or_else(|| anyhow!("No vault found — run 'stash auth login' first"))?;
    let old_password = rpassword::prompt_password("Current master password: ")?;
    let old_key = crypto::derive_key(&old_password, &old_salt)?;

    let canary_enc = B64.decode(
        db.get_meta("canary_enc")?
            .ok_or_else(|| anyhow!("Vault metadata corrupted"))?,
    )?;
    let canary_nonce = B64.decode(
        db.get_meta("canary_nonce")?
            .ok_or_else(|| anyhow!("Vault metadata corrupted"))?,
    )?;
    let plaintext = crypto::decrypt(&old_key, &canary_enc, &canary_nonce)
        .map_err(|_| anyhow!("Incorrect password"))?;
    if plaintext.as_slice() != CANARY {
        return Err(anyhow!("Incorrect password"));
    }

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

    let items = db.list_items()?;
    let mut item_updates: Vec<(i64, Vec<u8>, Vec<u8>)> = Vec::new();
    let mut history_updates: Vec<(i64, i64, Vec<u8>, Vec<u8>)> = Vec::new();
    let mut tag_updates: Vec<(i64, Vec<u8>, Vec<u8>)> = Vec::new();

    for item in &items {
        let pt = crypto::decrypt(&old_key, &item.content_enc, &item.nonce)
            .map_err(|e| anyhow!("Failed to decrypt '{}': {}", item.shortname, e))?;
        let (enc, nonce) = crypto::encrypt(&new_key, &pt)?;
        item_updates.push((item.id, enc, nonce));

        for entry in db.get_history(item.id)? {
            let pt = crypto::decrypt(&old_key, &entry.content_enc, &entry.nonce).map_err(|e| {
                anyhow!("Failed to decrypt history for '{}': {}", item.shortname, e)
            })?;
            let (enc, nonce) = crypto::encrypt(&new_key, &pt)?;
            history_updates.push((item.id, entry.version, enc, nonce));
        }

        for tag in db.get_tags(item.id)? {
            let pt = crypto::decrypt(&old_key, &tag.tag_enc, &tag.nonce)
                .map_err(|e| anyhow!("Failed to decrypt tag for '{}': {}", item.shortname, e))?;
            let (enc, nonce) = crypto::encrypt(&new_key, &pt)?;
            tag_updates.push((tag.id, enc, nonce));
        }
    }

    let (new_canary_enc, new_canary_nonce) = crypto::encrypt(&new_key, CANARY)?;

    db.reencrypt_all(
        &new_salt,
        &B64.encode(&new_canary_enc),
        &B64.encode(&new_canary_nonce),
        &item_updates,
        &history_updates,
        &tag_updates,
    )?;

    session::clear_key()?;
    println!("Master password changed. Run 'stash auth login' to start a new session.");
    Ok(())
}
