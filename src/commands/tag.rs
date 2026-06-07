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
            if String::from_utf8(decrypted.to_vec())? == *tag_to_remove {
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
            Ok(String::from_utf8(b.to_vec())?)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_db() -> (NamedTempFile, Db) {
        let f = NamedTempFile::new().unwrap();
        let db = Db::open(f.path()).unwrap();
        (f, db)
    }

    #[test]
    fn decrypt_tags_empty() {
        let key = [0x42u8; 32];
        let (_f, db) = make_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        assert!(decrypt_tags(&db, &key, id).unwrap().is_empty());
    }

    #[test]
    fn decrypt_tags_roundtrip() {
        let key = [0x42u8; 32];
        let (_f, db) = make_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        for tag in ["work", "personal"] {
            let (enc, nonce) = crypto::encrypt(&key, tag.as_bytes()).unwrap();
            db.add_tag(id, &enc, &nonce).unwrap();
        }
        let tags = decrypt_tags(&db, &key, id).unwrap();
        assert_eq!(tags, vec!["work", "personal"]);
    }

    #[test]
    fn decrypt_tags_wrong_key_fails() {
        let key = [0x42u8; 32];
        let wrong_key = [0x99u8; 32];
        let (_f, db) = make_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        let (enc, nonce) = crypto::encrypt(&key, b"work").unwrap();
        db.add_tag(id, &enc, &nonce).unwrap();
        assert!(decrypt_tags(&db, &wrong_key, id).is_err());
    }
}
