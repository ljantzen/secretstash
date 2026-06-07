use anyhow::{Result, anyhow};

use super::show::fmt_ts;
use crate::{db::Db, session};

pub fn restore(shortname: &str, version: Option<i64>, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let entry = match version {
        Some(v) => db
            .get_history_version(item.id, v)?
            .ok_or_else(|| anyhow!("Version {} not found for '{}'", v, shortname))?,
        None => db
            .get_latest_history(item.id)?
            .ok_or_else(|| anyhow!("No history to restore for '{}'", shortname))?,
    };

    // Verify the key can decrypt the history entry before committing
    let _ = crate::crypto::decrypt(&key, &entry.content_enc, &entry.nonce)
        .map_err(|_| anyhow!("Could not decrypt history entry (wrong key or corrupted data)"))?;

    db.replace_content(
        item.id,
        shortname,
        &item.content_enc,
        &item.nonce,
        &entry.content_enc,
        &entry.nonce,
    )?;

    println!(
        "Restored '{}' to v{} ({}).",
        shortname,
        entry.version,
        fmt_ts(&entry.created_at)
    );
    Ok(())
}
