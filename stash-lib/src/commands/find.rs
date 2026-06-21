use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn find(
    query: Option<&str>,
    tag_filter: Option<&str>,
    type_filter: Option<&str>,
    db_path: &std::path::Path,
) -> Result<()> {
    if query.is_none() && tag_filter.is_none() && type_filter.is_none() {
        return Err(anyhow!(
            "Provide a search term, --tag <TAG>, --type <TYPE>, or a combination."
        ));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;
    let items = db.list_items()?;

    if items.is_empty() {
        println!("Vault is empty.");
        return Ok(());
    }

    let query_lc = query.map(|q| q.to_lowercase());
    let tag_lc = tag_filter.map(|t| t.to_lowercase());
    let mut results: Vec<(String, String, String)> = Vec::new();

    for item in items {
        if let Some(t) = type_filter
            && item.item_type != t
        {
            continue;
        }

        if let Some(ref q) = query_lc {
            let content_match = item.content.to_lowercase().contains(q.as_str());
            let title_match = item
                .title
                .as_deref()
                .is_some_and(|t| t.to_lowercase().contains(q.as_str()));
            if !content_match && !title_match {
                continue;
            }
        }

        if let Some(ref tf) = tag_lc {
            let tags: Vec<String> = db.get_tags(item.id)?.into_iter().map(|t| t.tag).collect();
            if !tags.iter().any(|t| t.to_lowercase() == *tf) {
                continue;
            }
        }

        let snip = match query {
            Some(q) => snippet(&item.content, q, 40),
            None => preview(&item.content, 80),
        };
        results.push((item.shortname, item.item_type, snip));
    }

    if results.is_empty() {
        println!("No matches.");
        return Ok(());
    }

    let name_width = results.iter().map(|(n, _, _)| n.len()).max().unwrap_or(4);

    for (name, item_type, snip) in &results {
        println!("{:<name_width$}  [{:<6}]  {}", name, item_type, snip);
    }

    println!();
    println!("{} result(s).", results.len());
    Ok(())
}

fn preview(text: &str, max_chars: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = normalized.chars().collect();
    if chars.len() <= max_chars {
        normalized
    } else {
        format!("{}…", chars[..max_chars].iter().collect::<String>())
    }
}

fn snippet(text: &str, query: &str, context_chars: usize) -> String {
    let text_lc = text.to_lowercase();
    let query_lc = query.to_lowercase();

    let Some(byte_pos) = text_lc.find(&query_lc) else {
        let preview: String = text.chars().take(context_chars * 2).collect();
        return format!("{}…", preview.trim());
    };

    let char_pos = text[..byte_pos].chars().count();
    let query_char_len = query_lc.chars().count();
    let chars: Vec<char> = text.chars().collect();

    let start = char_pos.saturating_sub(context_chars);
    let end = (char_pos + query_char_len + context_chars).min(chars.len());

    let prefix = if start > 0 { "…" } else { "" };
    let suffix = if end < chars.len() { "…" } else { "" };

    let raw: String = chars[start..end].iter().collect();
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");

    format!("{}{}{}", prefix, normalized, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_match_at_start() {
        let s = snippet("hello world", "hello", 5);
        assert!(s.contains("hello"));
        assert!(!s.starts_with('…'));
    }

    #[test]
    fn snippet_match_at_end() {
        let s = snippet("hello world", "world", 5);
        assert!(s.contains("world"));
        assert!(!s.ends_with('…'));
    }

    #[test]
    fn snippet_match_in_middle_adds_ellipsis() {
        let text = format!("{}TARGET{}", "a".repeat(50), "b".repeat(50));
        let s = snippet(&text, "TARGET", 5);
        assert!(s.starts_with('…'));
        assert!(s.ends_with('…'));
        assert!(s.contains("TARGET"));
    }

    #[test]
    fn snippet_is_case_insensitive() {
        let s = snippet("Hello World", "world", 5);
        assert!(s.to_lowercase().contains("world"));
    }

    #[test]
    fn snippet_normalizes_whitespace() {
        let s = snippet("foo   bar\nbaz", "bar", 10);
        assert!(!s.contains('\n'));
        assert!(!s.contains("   "));
    }

    #[test]
    fn snippet_full_short_text_has_no_ellipsis() {
        let s = snippet("hi", "hi", 10);
        assert!(!s.starts_with('…'));
        assert!(!s.ends_with('…'));
    }

    #[test]
    fn preview_short_text_unchanged() {
        assert_eq!(preview("hello world", 20), "hello world");
    }

    #[test]
    fn preview_long_text_truncated() {
        let s = preview(&"word ".repeat(30), 20);
        assert!(s.ends_with('…'));
        assert!(s.chars().count() <= 21);
    }

    #[test]
    fn preview_normalizes_whitespace() {
        let s = preview("foo\n\nbar  baz", 100);
        assert!(!s.contains('\n'));
    }
}
