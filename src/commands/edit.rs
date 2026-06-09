use anyhow::{Result, anyhow};

use super::add::open_in_editor;
use crate::{db::Db, session};

pub fn edit(shortname: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let new_content = open_in_editor(&item.content)?;

    if new_content == item.content {
        println!("No changes made.");
        return Ok(());
    }

    if new_content.trim().is_empty() {
        return Err(anyhow!("Content cannot be empty"));
    }

    db.replace_content(item.id, shortname, &item.content, &new_content)?;

    println!("Updated '{}'.", shortname);
    Ok(())
}
