use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn copy(source: &str, dest: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(source)?
        .ok_or_else(|| anyhow!("Item '{}' not found", source))?;

    if db.item_exists(dest)? {
        return Err(anyhow!("Item '{}' already exists", dest));
    }

    let new_id = db.insert_item(
        dest,
        &item.item_type,
        &item.content,
        item.title.as_deref(),
        item.browser.as_deref(),
    )?;

    for tag in db.get_tags(item.id)? {
        db.add_tag(new_id, &tag.tag)?;
    }

    println!("Copied '{}' to '{}'.", source, dest);
    Ok(())
}
