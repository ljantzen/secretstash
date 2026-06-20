use anyhow::{Result, anyhow};

use crate::{clipboard::Clipboard, db::Db, session};

pub fn show(
    shortname: &str,
    verbose: bool,
    copy: bool,
    clear_after_secs: u64,
    db_path: &std::path::Path,
) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    if copy {
        let cb = Clipboard::copy(item.content.trim_end())?;
        println!("Copied '{}' to clipboard.", shortname);
        if clear_after_secs > 0 {
            cb.schedule_clear(clear_after_secs);
            println!(
                "Clipboard will be cleared in {} second(s).",
                clear_after_secs
            );
        }
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

pub fn fmt_ts(ts: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|_| ts.to_string())
}
