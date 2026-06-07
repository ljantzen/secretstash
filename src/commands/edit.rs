use anyhow::{Result, anyhow};

use super::add::open_in_editor;
use crate::{config, crypto, db::Db, session};

pub fn edit(shortname: &str) -> Result<()> {
    let key = session::load_key()?;
    let db_path = config::db_path()?;
    let db = Db::open(&db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let current_bytes = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let current_text = String::from_utf8(current_bytes)?;

    let new_text = open_in_editor(&current_text)?;

    if new_text == current_text {
        println!("No changes made.");
        return Ok(());
    }

    if new_text.trim().is_empty() {
        return Err(anyhow!("Content cannot be empty"));
    }

    // Archive current version before overwriting
    db.add_history(item.id, &item.content_enc, &item.nonce)?;

    let (enc, nonce) = crypto::encrypt(&key, new_text.as_bytes())?;
    db.update_item(shortname, &enc, &nonce)?;

    println!("Updated '{}'.", shortname);
    Ok(())
}
