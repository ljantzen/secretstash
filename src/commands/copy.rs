use anyhow::{Result, anyhow};

use crate::{crypto, db::Db, session};

pub fn copy(source: &str, dest: &str, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(source)?
        .ok_or_else(|| anyhow!("Item '{}' not found", source))?;

    if db.item_exists(dest)? {
        return Err(anyhow!("Item '{}' already exists", dest));
    }

    // Re-encrypt with a fresh nonce
    let content = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let (enc, nonce) = crypto::encrypt(&key, content.as_slice())?;
    let new_id = db.insert_item(dest, &item.item_type, &enc, &nonce, item.browser.as_deref())?;

    // Copy tags, each with a fresh nonce
    for tag in db.get_tags(item.id)? {
        let tag_bytes = crypto::decrypt(&key, &tag.tag_enc, &tag.nonce)?;
        let (tag_enc, tag_nonce) = crypto::encrypt(&key, tag_bytes.as_slice())?;
        db.add_tag(new_id, &tag_enc, &tag_nonce)?;
    }

    println!("Copied '{}' to '{}'.", source, dest);
    Ok(())
}
