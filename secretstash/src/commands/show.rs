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
        .ok_or_else(|| anyhow!("Item '{shortname}' not found"))?;

    if copy {
        let cb = Clipboard::copy(item.content.trim_end())?;
        println!("Copied '{shortname}' to clipboard.");
        if clear_after_secs > 0 {
            cb.schedule_clear(clear_after_secs);
            println!("Clipboard will be cleared in {clear_after_secs} second(s).");
        }
        return Ok(());
    }

    let mut tags: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();
    tags.sort();

    if verbose {
        println!("shortname : {}", item.shortname);
        println!("type      : {}", item.item_type);
        if let Some(t) = &item.title {
            println!("title     : {t}");
        }
        if let Some(b) = &item.browser {
            println!("browser   : {b}");
        }
        if item.private == Some(true) {
            println!("private   : yes");
        }
        if !tags.is_empty() {
            println!("tags      : {}", tags.join(", "));
        }
        println!("created   : {}", fmt_ts(&item.created_at));
        println!("updated   : {}", fmt_ts(&item.updated_at));
        println!();
    } else if let Some(t) = &item.title {
        println!("{t}");
    }
    println!("{}", item.content.trim_end());
    if !verbose && !tags.is_empty() {
        println!();
        println!("tags: {}", tags.join(", "));
    }
    Ok(())
}

#[must_use]
pub fn fmt_ts(ts: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(ts).map_or_else(
        |_| ts.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M UTC").to_string(),
    )
}
