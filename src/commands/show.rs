use anyhow::{Result, anyhow};

use super::tag::decrypt_tags;
use crate::{crypto, db::Db, session};

pub fn show(shortname: &str, verbose: bool, copy: bool, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let content = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let text = String::from_utf8(content.to_vec())?;

    if copy {
        copy_to_clipboard(text.trim_end())?;
        println!("Copied '{}' to clipboard.", shortname);
        return Ok(());
    }

    let mut tags = decrypt_tags(&db, &key, item.id)?;
    tags.sort();

    if verbose {
        println!("shortname : {}", item.shortname);
        println!("type      : {}", item.item_type);
        if !tags.is_empty() {
            println!("tags      : {}", tags.join(", "));
        }
        println!("created   : {}", fmt_ts(&item.created_at));
        println!("updated   : {}", fmt_ts(&item.updated_at));
        println!();
    }
    println!("{}", text.trim_end());
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
        ("pbcopy", &[]),                         // macOS
        ("wl-copy", &[]),                        // Wayland
        ("xclip", &["-selection", "clipboard"]), // X11
        ("xsel", &["--clipboard", "--input"]),   // X11 alt
        ("clip.exe", &[]),                       // Windows (also WSL)
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
