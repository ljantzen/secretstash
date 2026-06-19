use std::io::{self, Read};
use std::path::Path;

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::{db::Db, session};

#[derive(Deserialize)]
struct ImportFile {
    version: u32,
    items: Vec<ImportItem>,
}

#[derive(Deserialize)]
struct ImportItem {
    shortname: String,
    #[serde(rename = "type")]
    item_type: String,
    content: String,
    #[serde(default)]
    tags: Vec<String>,
    browser: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(default)]
    history: Option<Vec<ImportHistoryEntry>>,
}

#[derive(Deserialize)]
struct ImportHistoryEntry {
    version: i64,
    content: String,
    created_at: String,
}

pub fn import(overwrite: bool, file: Option<&Path>, db_path: &Path) -> Result<()> {
    let json = match file {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let (imported, skipped, failed) = load_from_str(&db, &json, overwrite)?;

    println!("Imported {} item(s).", imported);
    if skipped > 0 {
        println!(
            "Skipped {} item(s) that already exist. Use --overwrite to replace them.",
            skipped
        );
    }
    if failed > 0 {
        println!(
            "{} item(s) could not be imported (see warnings above).",
            failed
        );
    }

    Ok(())
}

fn load_from_str(db: &Db, json: &str, overwrite: bool) -> Result<(usize, usize, usize)> {
    let import_file: ImportFile =
        serde_json::from_str(json).map_err(|e| anyhow!("Invalid export file: {}", e))?;

    if import_file.version != 1 {
        eprintln!(
            "Warning: export file version is {}, expected 1. Proceeding anyway.",
            import_file.version
        );
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for item in import_file.items {
        if item.item_type != "url" && item.item_type != "note" {
            eprintln!(
                "Skipping '{}': unknown type '{}'.",
                item.shortname, item.item_type
            );
            failed += 1;
            continue;
        }

        let exists = db.item_exists(&item.shortname)?;
        if exists && !overwrite {
            skipped += 1;
            continue;
        }
        if exists {
            db.delete_item(&item.shortname)?;
        }

        let item_id = db.insert_item_full(
            &item.shortname,
            &item.item_type,
            &item.content,
            item.browser.as_deref(),
            &item.created_at,
            &item.updated_at,
        )?;

        for tag in &item.tags {
            let tag = tag.trim();
            if !tag.is_empty() {
                db.add_tag(item_id, tag)?;
            }
        }

        if let Some(history) = item.history {
            for entry in history {
                db.insert_history_entry(item_id, &entry.content, entry.version, &entry.created_at)?;
            }
        }

        imported += 1;
    }

    Ok((imported, skipped, failed))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "version": 1,
        "exported_at": "2026-01-01T00:00:00Z",
        "items": [
            {
                "shortname": "gh",
                "type": "url",
                "content": "https://github.com",
                "tags": ["work"],
                "browser": "firefox",
                "created_at": "2025-01-01T00:00:00Z",
                "updated_at": "2025-06-01T00:00:00Z"
            },
            {
                "shortname": "note1",
                "type": "note",
                "content": "hello world",
                "tags": [],
                "created_at": "2025-02-01T00:00:00Z",
                "updated_at": "2025-02-01T00:00:00Z",
                "history": [
                    {"version": 1, "content": "hello", "created_at": "2025-01-01T00:00:00Z"}
                ]
            }
        ]
    }"#;

    #[test]
    fn basic_import() {
        let db = Db::open_in_memory().unwrap();
        let (imported, skipped, failed) = load_from_str(&db, SAMPLE, false).unwrap();
        assert_eq!(imported, 2);
        assert_eq!(skipped, 0);
        assert_eq!(failed, 0);

        let gh = db.get_item("gh").unwrap().unwrap();
        assert_eq!(gh.content, "https://github.com");
        assert_eq!(gh.item_type, "url");
        assert_eq!(gh.browser.as_deref(), Some("firefox"));
        assert_eq!(gh.created_at, "2025-01-01T00:00:00Z");
        assert_eq!(gh.updated_at, "2025-06-01T00:00:00Z");

        let tags = db.get_tags(gh.id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, "work");
    }

    #[test]
    fn history_is_imported() {
        let db = Db::open_in_memory().unwrap();
        load_from_str(&db, SAMPLE, false).unwrap();

        let note = db.get_item("note1").unwrap().unwrap();
        let history = db.get_history(note.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "hello");
        assert_eq!(history[0].version, 1);
        assert_eq!(history[0].created_at, "2025-01-01T00:00:00Z");
    }

    #[test]
    fn skip_existing_by_default() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("gh", "note", "original", None).unwrap();

        let (imported, skipped, _) = load_from_str(&db, SAMPLE, false).unwrap();
        assert_eq!(imported, 1);
        assert_eq!(skipped, 1);

        // original content must be untouched
        assert_eq!(db.get_item("gh").unwrap().unwrap().content, "original");
    }

    #[test]
    fn overwrite_replaces_existing() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("gh", "note", "original", None).unwrap();

        let (imported, skipped, _) = load_from_str(&db, SAMPLE, true).unwrap();
        assert_eq!(imported, 2);
        assert_eq!(skipped, 0);

        assert_eq!(
            db.get_item("gh").unwrap().unwrap().content,
            "https://github.com"
        );
    }

    #[test]
    fn unknown_type_counted_as_failed() {
        let json = r#"{
            "version": 1,
            "exported_at": "2026-01-01T00:00:00Z",
            "items": [
                {
                    "shortname": "x",
                    "type": "secret",
                    "content": "oops",
                    "tags": [],
                    "created_at": "2025-01-01T00:00:00Z",
                    "updated_at": "2025-01-01T00:00:00Z"
                }
            ]
        }"#;
        let db = Db::open_in_memory().unwrap();
        let (imported, _, failed) = load_from_str(&db, json, false).unwrap();
        assert_eq!(imported, 0);
        assert_eq!(failed, 1);
    }

    #[test]
    fn invalid_json_returns_error() {
        let db = Db::open_in_memory().unwrap();
        assert!(load_from_str(&db, "not json", false).is_err());
    }

    #[test]
    fn timestamps_preserved() {
        let db = Db::open_in_memory().unwrap();
        load_from_str(&db, SAMPLE, false).unwrap();

        let note = db.get_item("note1").unwrap().unwrap();
        assert_eq!(note.created_at, "2025-02-01T00:00:00Z");
        assert_eq!(note.updated_at, "2025-02-01T00:00:00Z");
    }
}
