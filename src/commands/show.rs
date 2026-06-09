use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn show(shortname: &str, verbose: bool, copy: bool, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    if copy {
        copy_to_clipboard(item.content.trim_end())?;
        println!("Copied '{}' to clipboard.", shortname);
        return Ok(());
    }

    let mut tags: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();
    tags.sort();

    if verbose {
        println!("shortname : {}", item.shortname);
        println!("type      : {}", item.item_type);
        if let Some(b) = &item.browser {
            println!("browser   : {}", b);
        }
        if !tags.is_empty() {
            println!("tags      : {}", tags.join(", "));
        }
        println!("created   : {}", fmt_ts(&item.created_at));
        println!("updated   : {}", fmt_ts(&item.updated_at));
        println!();
    }
    println!("{}", item.content.trim_end());
    if !verbose && !tags.is_empty() {
        println!();
        println!("tags: {}", tags.join(", "));
    }
    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let candidates: &[(&str, &[&str])] = &[
        ("pbcopy", &[]),
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
        ("clip.exe", &[]),
    ];

    for (cmd, args) in candidates {
        match Command::new(cmd).args(*args).stdin(Stdio::piped()).spawn() {
            Ok(mut child) => {
                child.stdin.take().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Err(anyhow!(
        "No clipboard command found. \
         Install pbcopy (macOS), wl-clipboard (Wayland), xclip/xsel (X11), \
         or use Windows where clip.exe is built in."
    ))
}

pub fn fmt_ts(ts: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|_| ts.to_string())
}
