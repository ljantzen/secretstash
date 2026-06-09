use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn add_tags(shortname: &str, new_tags: &[String], db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let existing: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();

    let mut added = 0;
    for tag in new_tags {
        let tag = tag.trim();
        if tag.is_empty() || existing.contains(&tag.to_string()) {
            continue;
        }
        db.add_tag(item.id, tag)?;
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
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let raw_tags = db.get_tags(item.id)?;
    let mut removed = 0;

    for tag_to_remove in remove {
        for raw in &raw_tags {
            if raw.tag == *tag_to_remove {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None).unwrap();
        db.add_tag(id, "work").unwrap();
        db.add_tag(id, "personal").unwrap();
        let tags: Vec<String> = db
            .get_tags(id)
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert_eq!(tags, vec!["work", "personal"]);
    }

    #[test]
    fn tags_empty() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None).unwrap();
        let tags: Vec<String> = db
            .get_tags(id)
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert!(tags.is_empty());
    }
}
