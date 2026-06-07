use anyhow::{Result, anyhow};

use super::show::fmt_ts;
use crate::{crypto, db::Db, session};

pub fn history(shortname: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let entries = db.get_history(item.id)?;

    println!("History for '{}' ({}):", item.shortname, item.item_type);

    for entry in &entries {
        let bytes = crypto::decrypt(&key, &entry.content_enc, &entry.nonce)?;
        let text = String::from_utf8(bytes.to_vec())?;
        println!();
        println!("─── v{} ({}) ───", entry.version, fmt_ts(&entry.created_at));
        println!("{}", text.trim_end());
    }

    let bytes = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let current = String::from_utf8(bytes.to_vec())?;
    println!();
    println!("─── current ({}) ───", fmt_ts(&item.updated_at));
    println!("{}", current.trim_end());
    Ok(())
}
