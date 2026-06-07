use anyhow::{Result, anyhow};

use super::tag::decrypt_tags;
use crate::{crypto, db::Db, session};

pub fn show(shortname: &str, verbose: bool, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    let content = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let text = String::from_utf8(content.to_vec())?;

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

pub fn fmt_ts(ts: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|_| ts.to_string())
}
