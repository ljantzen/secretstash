use anyhow::{Result, anyhow};

use super::show::fmt_ts;
use crate::{db::Db, session};

pub fn history(shortname: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let entries = db.get_history(item.id)?;

    println!("History for '{}' ({}):", item.shortname, item.item_type);

    for entry in &entries {
        println!();
        println!("─── v{} ({}) ───", entry.version, fmt_ts(&entry.created_at));
        println!("{}", entry.content.trim_end());
    }

    println!();
    println!("─── current ({}) ───", fmt_ts(&item.updated_at));
    println!("{}", item.content.trim_end());
    Ok(())
}
