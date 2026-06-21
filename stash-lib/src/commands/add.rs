use anyhow::{Result, anyhow};
use std::io::{self, Read, Write};

use crate::{db::Db, session};

#[allow(clippy::too_many_arguments)]
pub fn add(
    item_type: &str,
    shortname: &str,
    edit: bool,
    from_stdin: bool,
    tags: &[String],
    text: Option<&str>,
    title: Option<&str>,
    browser: Option<&str>,
    db_path: &std::path::Path,
) -> Result<()> {
    if edit && from_stdin {
        return Err(anyhow!("Cannot use --edit and --stdin together"));
    }
    if browser.is_some() && item_type != "url" {
        return Err(anyhow!("--browser is only valid for URL items"));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    if db.item_exists(shortname)? {
        return Err(anyhow!(
            "Item '{}' already exists. Use 'stash edit {}' to update it.",
            shortname,
            shortname
        ));
    }

    let content = if edit {
        open_in_editor(text.unwrap_or(""))?
    } else if from_stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else if let Some(t) = text {
        t.to_string()
    } else {
        return Err(anyhow!(
            "No content provided. Use --edit, --stdin, or pass text as an argument."
        ));
    };

    if content.trim().is_empty() {
        return Err(anyhow!("Content cannot be empty"));
    }

    let item_id = db.insert_item(shortname, item_type, &content, title, browser)?;

    let mut seen = std::collections::HashSet::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() || !seen.insert(tag) {
            continue;
        }
        db.add_tag(item_id, tag)?;
    }

    println!("Added '{}' ({}).", shortname, item_type);
    Ok(())
}

pub fn open_in_editor(initial: &str) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(initial.as_bytes())?;
    tmp.flush()?;

    let status = std::process::Command::new(&editor)
        .arg(tmp.path())
        .status()
        .map_err(|e| anyhow!("Failed to launch '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status"));
    }

    Ok(std::fs::read_to_string(tmp.path())?)
}
