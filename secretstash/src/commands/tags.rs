use anyhow::Result;

use crate::{db::Db, session};

pub fn tags(db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;
    let all_tags = db.list_all_tags()?;

    if all_tags.is_empty() {
        println!("No tags in vault.");
        return Ok(());
    }

    let tag_w = all_tags
        .iter()
        .map(|(t, _)| t.len())
        .max()
        .unwrap_or(3)
        .max("TAG".len());

    println!("{:<tag_w$}  ITEMS", "TAG");
    println!("{}", "─".repeat(tag_w + 2 + 5));
    for (tag, count) in &all_tags {
        println!("{:<tag_w$}  {}", tag, count);
    }
    println!();
    println!("{} tag(s).", all_tags.len());
    Ok(())
}
