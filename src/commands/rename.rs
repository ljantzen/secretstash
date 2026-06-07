use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn rename(old: &str, new: &str, db_path: &std::path::Path) -> Result<()> {
    session::load_key()?;
    let db = Db::open(db_path)?;

    if db.get_item(new)?.is_some() {
        return Err(anyhow!("Item '{}' already exists", new));
    }

    db.rename_item(old, new)?;
    println!("Renamed '{}' to '{}'.", old, new);
    Ok(())
}
