use anyhow::{Result, anyhow};
use std::io::{self, Read, Write};
use std::path::PathBuf;

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
    private: Option<bool>,
    db_path: &std::path::Path,
) -> Result<()> {
    if edit && from_stdin {
        return Err(anyhow!("Cannot use --edit and --stdin together"));
    }
    if browser.is_some() && item_type != "url" {
        return Err(anyhow!("--browser is only valid for URL items"));
    }
    if private.is_some() && item_type != "url" {
        return Err(anyhow!(
            "--private/--no-private is only valid for URL items"
        ));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    if db.item_exists(shortname)? {
        return Err(anyhow!(
            "Item '{shortname}' already exists. Use 'stash edit {shortname}' to update it."
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

    let resolved_browser = browser
        .map(crate::commands::web::resolve_browser)
        .transpose()?;
    let item_id = db.insert_item(
        shortname,
        item_type,
        &content,
        title,
        resolved_browser.as_deref(),
    )?;

    if let Some(p) = private {
        db.set_private(shortname, if p { Some(true) } else { None })?;
    }

    let mut seen = std::collections::HashSet::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() || !seen.insert(tag) {
            continue;
        }
        db.add_tag(item_id, tag)?;
    }

    println!("Added '{shortname}' ({item_type}).");
    Ok(())
}

pub fn open_in_editor(initial: &str) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let mut tmp = tempfile::Builder::new()
        .prefix(".stash-edit-")
        .tempfile_in(secure_tmp_dir())?;
    tmp.write_all(initial.as_bytes())?;
    tmp.flush()?;

    let status = std::process::Command::new(&editor)
        .arg(tmp.path())
        .status()
        .map_err(|e| anyhow!("Failed to launch '{editor}': {e}"))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status"));
    }

    Ok(std::fs::read_to_string(tmp.path())?)
}

/// Prefer a tmpfs-backed, per-user directory for the editor scratch file so
/// plaintext secrets don't linger on persistent storage after the file is
/// unlinked. Falls back to the regular temp dir when unavailable.
fn secure_tmp_dir() -> PathBuf {
    resolve_secure_tmp_dir(std::env::var("XDG_RUNTIME_DIR").ok().as_deref())
}

/// Pure resolution logic, separated from env-var access so it's testable
/// without mutating process-wide environment state.
fn resolve_secure_tmp_dir(xdg_runtime_dir: Option<&str>) -> PathBuf {
    if let Some(dir) = xdg_runtime_dir {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path;
        }
    }
    std::env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_secure_tmp_dir_prefers_existing_xdg_runtime_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve_secure_tmp_dir(Some(dir.path().to_str().unwrap())),
            dir.path()
        );
    }

    #[test]
    fn resolve_secure_tmp_dir_falls_back_when_unset() {
        assert_eq!(resolve_secure_tmp_dir(None), std::env::temp_dir());
    }

    #[test]
    fn resolve_secure_tmp_dir_falls_back_when_not_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let bogus = dir.path().join("does-not-exist");
        assert_eq!(
            resolve_secure_tmp_dir(Some(bogus.to_str().unwrap())),
            std::env::temp_dir()
        );
    }
}
