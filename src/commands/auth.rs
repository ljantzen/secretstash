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

    session::save_key(&key, session_timeout_minutes)?;
    if session_timeout_minutes == 0 {
        println!("Logged in. Session does not expire.");
    } else {
        println!(
            "Logged in. Session expires in {} minute(s).",
            session_timeout_minutes
        );
    }
    Ok(())
}

pub fn logout() -> Result<()> {
    session::clear_key()?;
    println!("Logged out.");
    Ok(())
}
