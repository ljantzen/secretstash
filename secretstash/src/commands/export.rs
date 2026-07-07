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
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<ExportHistoryEntry>>,
}

#[derive(Serialize)]
struct ExportHistoryEntry {
    version: i64,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
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
                    title: h.title,
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
            private: item.private,
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

    if let Some(path) = output {
        write_export_file(path, json.as_bytes())?;
        eprintln!("Exported to {}", path.display());
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        out.write_all(json.as_bytes())?;
        out.write_all(b"\n")?;
    }

    Ok(())
}

// Write the export file with 0600 permissions on Unix so plaintext secrets
// don't land on disk world/group-readable under the process umask. The
// existing file (if any) is removed first so there is no window in which it
// exists with permissive permissions.
#[cfg(unix)]
fn write_export_file(path: &Path, json: &[u8]) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(json)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_export_file(path: &Path, json: &[u8]) -> Result<()> {
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn export_file_has_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        write_export_file(&path, b"{}").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn write_export_file_writes_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        write_export_file(&path, b"{\"version\":1}").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"version\":1}");
    }

    #[test]
    fn write_export_file_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");
        std::fs::write(&path, b"old content").unwrap();

        write_export_file(&path, b"new content").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new content");
    }
}
