use anyhow::{Result, anyhow};

use super::add::open_in_editor;
use crate::{crypto, db::Db, session};

pub fn edit(shortname: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let current_bytes = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let current_text = String::from_utf8(current_bytes.to_vec())?;

    let new_text = open_in_editor(&current_text)?;

    if new_text == current_text {
        println!("No changes made.");
        return Ok(());
    }

    if new_text.trim().is_empty() {
        return Err(anyhow!("Content cannot be empty"));
    }

    let (enc, nonce) = crypto::encrypt(&key, new_text.as_bytes())?;
    db.replace_content(
        item.id,
        shortname,
        &item.content_enc,
        &item.nonce,
        &enc,
        &nonce,
    )?;

    println!("Updated '{}'.", shortname);
    Ok(())
}
