use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn rename(old: &str, new: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    if db.get_item(new)?.is_some() {
        return Err(anyhow!("Item '{new}' already exists"));
    }

    db.rename_item(old, new)?;
    println!("Renamed '{old}' to '{new}'.");
    Ok(())
}
