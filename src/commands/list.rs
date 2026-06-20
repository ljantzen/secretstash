use anyhow::Result;

use crate::{db::Db, session};

pub fn list(
    tag_filters: &[String],
    type_filter: Option<&str>,
    db_path: &std::path::Path,
) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;
    let items = db.list_items()?;

    if items.is_empty() {
        println!("Vault is empty.");
        return Ok(());
    }

    let filters_lc: Vec<String> = tag_filters.iter().map(|t| t.to_lowercase()).collect();

    let mut rows: Vec<(String, String, Option<String>, Vec<String>)> = Vec::new();
    for item in items {
        if let Some(t) = type_filter
            && item.item_type != t
        {
            continue;
        }

        let mut tags: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();
        tags.sort();

        if !filters_lc.is_empty() {
            let tags_lc: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
            if !filters_lc.iter().any(|f| tags_lc.contains(f)) {
                continue;
            }
        }

        rows.push((item.shortname, item.item_type, item.title, tags));
    }

    if rows.is_empty() {
        println!(
            "No items tagged with: {}.",
            tag_filters
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(());
    }

    let name_w = rows
        .iter()
        .map(|(n, _, _, _)| n.len())
        .max()
        .unwrap_or(4)
        .max("NAME".len());
    let type_w = "note".len();
    let title_w = rows
        .iter()
        .filter_map(|(_, _, t, _)| t.as_deref().map(|s| s.len()))
        .max()
        .unwrap_or(0)
        .max("TITLE".len());

    let has_titles = rows.iter().any(|(_, _, t, _)| t.is_some());

    if has_titles {
        println!(
            "{:<name_w$}  {:<type_w$}  {:<title_w$}  TAGS",
            "NAME", "TYPE", "TITLE"
        );
        println!("{}", "─".repeat(name_w + 2 + type_w + 2 + title_w + 2 + 20));
    } else {
        println!("{:<name_w$}  {:<type_w$}  TAGS", "NAME", "TYPE");
        println!("{}", "─".repeat(name_w + 2 + type_w + 2 + 20));
    }

    for (name, item_type, title, tags) in &rows {
        if has_titles {
            println!(
                "{:<name_w$}  {:<type_w$}  {:<title_w$}  {}",
                name,
                item_type,
                title.as_deref().unwrap_or(""),
                tags.join(", ")
            );
        } else {
            println!(
                "{:<name_w$}  {:<type_w$}  {}",
                name,
                item_type,
                tags.join(", ")
            );
        }
    }

    println!();
    println!("{} item(s).", rows.len());
    Ok(())
}
