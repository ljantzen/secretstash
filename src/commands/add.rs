use anyhow::{Result, anyhow};
use std::io::{self, Read, Write};

use crate::{cli::ItemType, crypto, db::Db, session};

pub fn add(
    item_type: ItemType,
    shortname: &str,
    edit: bool,
    from_stdin: bool,
    tags: &[String],
    text: Option<&str>,
    db_path: &std::path::Path,
) -> Result<()> {
    if edit && from_stdin {
        return Err(anyhow!("Cannot use --edit and --stdin together"));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path)?;

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

    let type_str = item_type.to_string();
    let (enc, nonce) = crypto::encrypt(&key, content.as_bytes())?;
    let item_id = db.insert_item(shortname, &type_str, &enc, &nonce)?;

    let mut seen = std::collections::HashSet::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() || !seen.insert(tag) {
            continue;
        }
        let (tag_enc, tag_nonce) = crypto::encrypt(&key, tag.as_bytes())?;
        db.add_tag(item_id, &tag_enc, &tag_nonce)?;
    }

    println!("Added '{}' ({}).", shortname, type_str);
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
