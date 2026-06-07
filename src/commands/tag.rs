use anyhow::{Result, anyhow};

use crate::{crypto, db::Db, session};

pub fn add_tags(shortname: &str, new_tags: &[String], db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    // Decrypt existing tags so we can deduplicate
    let existing_tags = decrypt_tags(&db, &key, item.id)?;

    let mut added = 0;
    for tag in new_tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }
        if existing_tags.contains(&tag.to_string()) {
            continue;
        }
        let (enc, nonce) = crypto::encrypt(&key, tag.as_bytes())?;
        db.add_tag(item.id, &enc, &nonce)?;
        added += 1;
    }

    match added {
        0 => println!(
            "No new tags added (all already present on '{}').",
            shortname
        ),
        n => println!("Added {} tag(s) to '{}'.", n, shortname),
    }
    Ok(())
}

pub fn remove_tags(shortname: &str, remove: &[String], db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let raw_tags = db.get_tags(item.id)?;
    let mut removed = 0;

    for tag_to_remove in remove {
        for raw in &raw_tags {
            let decrypted = crypto::decrypt(&key, &raw.tag_enc, &raw.nonce)?;
            if String::from_utf8(decrypted)? == *tag_to_remove {
                db.delete_tag(raw.id)?;
                removed += 1;
                break;
            }
        }
    }

    match removed {
        0 => println!("No matching tags found on '{}'.", shortname),
        n => println!("Removed {} tag(s) from '{}'.", n, shortname),
    }
    Ok(())
}

pub fn decrypt_tags(db: &Db, key: &[u8; 32], item_id: i64) -> Result<Vec<String>> {
    db.get_tags(item_id)?
        .into_iter()
        .map(|t| {
            let b = crypto::decrypt(key, &t.tag_enc, &t.nonce)?;
            Ok(String::from_utf8(b)?)
        })
        .collect()
}
