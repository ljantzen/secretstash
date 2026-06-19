use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::{db::Db, session};

#[derive(Serialize)]
struct ExportFile {
    version: u32,
    exported_at: String,
    items: Vec<ExportItem>,
}

#[derive(Serialize)]
struct ExportItem {
    shortname: String,
    #[serde(rename = "type")]
    item_type: String,
    content: String,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    browser: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<ExportHistoryEntry>>,
}

#[derive(Serialize)]
struct ExportHistoryEntry {
    version: i64,
    content: String,
    created_at: String,
}

pub fn export(include_history: bool, output: Option<&Path>, db_path: &Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let mut items: Vec<ExportItem> = Vec::new();
    for item in db.list_items()? {
        let mut tags: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();
        tags.sort();

        let history = if include_history {
            let entries = db
                .get_history(item.id)?
                .into_iter()
                .map(|h| ExportHistoryEntry {
                    version: h.version,
                    content: h.content,
                    created_at: h.created_at,
                })
                .collect();
            Some(entries)
        } else {
            None
        };

        items.push(ExportItem {
            shortname: item.shortname,
            item_type: item.item_type,
            content: item.content,
            tags,
            browser: item.browser,
            created_at: item.created_at,
            updated_at: item.updated_at,
            history,
        });
    }

    let file = ExportFile {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        items,
    };

    let json = serde_json::to_string_pretty(&file)?;

    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            eprintln!("Exported to {}", path.display());
        }
        None => {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            out.write_all(json.as_bytes())?;
            out.write_all(b"\n")?;
        }
    }

    Ok(())
}
