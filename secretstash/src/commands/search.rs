use anyhow::{Result, anyhow};
use regex::Regex;

use crate::{db::Db, session};

pub fn search(
    pattern: Option<&str>,
    use_regex: bool,
    include_history: bool,
    tag_filter: Option<&str>,
    type_filter: Option<&str>,
    db_path: &std::path::Path,
) -> Result<()> {
    if pattern.is_none() && tag_filter.is_none() && type_filter.is_none() {
        return Err(anyhow!(
            "Provide a search pattern, --tag <TAG>, --type <TYPE>, or a combination."
        ));
    }
    if use_regex && pattern.is_none() {
        return Err(anyhow!("--regex requires a pattern."));
    }

    let matcher = pattern.map(|p| Matcher::build(p, use_regex)).transpose()?;
    let tag_lc = tag_filter.map(|t| t.to_lowercase());

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;
    let items = db.list_items()?;

    if items.is_empty() {
        println!("Vault is empty.");
        return Ok(());
    }

    // (display label, item type, snippet)
    let mut results: Vec<(String, String, String)> = Vec::new();

    for item in &items {
        if let Some(t) = type_filter
            && item.item_type != t
        {
            continue;
        }

        if let Some(ref tf) = tag_lc {
            let tags = db.get_tags(item.id)?;
            if !tags.iter().any(|t| t.tag.to_lowercase() == *tf) {
                continue;
            }
        }

        if let Some(ref m) = matcher {
            let content_snip = m.find_snippet(&item.content);
            let title_snip = item
                .title
                .as_deref()
                .and_then(|t| m.find_snippet(t).map(|_| preview(&item.content, 80)));
            match content_snip.or(title_snip) {
                Some(snip) => results.push((item.shortname.clone(), item.item_type.clone(), snip)),
                None => continue,
            }
        } else {
            results.push((
                item.shortname.clone(),
                item.item_type.clone(),
                preview(&item.content, 80),
            ));
        }

        if include_history {
            let entries = db.get_history(item.id)?;
            for entry in &entries {
                if let Some(ref m) = matcher {
                    if let Some(snip) = m.find_snippet(&entry.content) {
                        let label = format!("{}:v{}", item.shortname, entry.version);
                        results.push((label, item.item_type.clone(), snip));
                    }
                } else {
                    let label = format!("{}:v{}", item.shortname, entry.version);
                    results.push((label, item.item_type.clone(), preview(&entry.content, 80)));
                }
            }
        }
    }

    if results.is_empty() {
        println!("No matches.");
        return Ok(());
    }

    let label_w = results.iter().map(|(l, _, _)| l.len()).max().unwrap_or(4);
    for (label, item_type, snip) in &results {
        println!("{:<label_w$}  [{:<6}]  {}", label, item_type, snip);
    }

    println!();
    println!("{} result(s).", results.len());
    Ok(())
}

enum Matcher {
    Literal(String),
    Regex(Regex),
}

impl Matcher {
    fn build(pattern: &str, use_regex: bool) -> Result<Self> {
        if use_regex {
            let re = Regex::new(pattern).map_err(|e| anyhow!("Invalid regex: {e}"))?;
            Ok(Self::Regex(re))
        } else {
            Ok(Self::Literal(pattern.to_lowercase()))
        }
    }

    fn find_snippet(&self, text: &str) -> Option<String> {
        const CTX: usize = 40;
        match self {
            Self::Literal(q) => {
                let text_lc = text.to_lowercase();
                let byte_pos = text_lc.find(q.as_str())?;
                let char_start = text[..byte_pos].chars().count();
                let char_end = char_start + q.chars().count();
                Some(snippet_at_chars(text, char_start, char_end, CTX))
            }
            Self::Regex(re) => {
                let m = re.find(text)?;
                let char_start = text[..m.start()].chars().count();
                let char_end = text[..m.end()].chars().count();
                Some(snippet_at_chars(text, char_start, char_end, CTX))
            }
        }
    }
}

fn snippet_at_chars(text: &str, match_start: usize, match_end: usize, context: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = match_start.saturating_sub(context);
    let end = (match_end + context).min(chars.len());
    let prefix = if start > 0 { "…" } else { "" };
    let suffix = if end < chars.len() { "…" } else { "" };
    let raw: String = chars[start..end].iter().collect();
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{prefix}{normalized}{suffix}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_match_found() {
        let m = Matcher::build("hello", false).unwrap();
        assert!(m.find_snippet("say hello world").is_some());
    }

    #[test]
    fn literal_match_case_insensitive() {
        let m = Matcher::build("HELLO", false).unwrap();
        assert!(m.find_snippet("say hello world").is_some());
    }

    #[test]
    fn literal_no_match() {
        let m = Matcher::build("xyz", false).unwrap();
        assert!(m.find_snippet("hello world").is_none());
    }

    #[test]
    fn regex_match_found() {
        let m = Matcher::build(r"\d{4}", true).unwrap();
        assert!(m.find_snippet("code: 1234").is_some());
    }

    #[test]
    fn regex_no_match() {
        let m = Matcher::build(r"\d{4}", true).unwrap();
        assert!(m.find_snippet("no digits here").is_none());
    }

    #[test]
    fn regex_invalid_pattern_errors() {
        assert!(Matcher::build(r"[unclosed", true).is_err());
    }

    #[test]
    fn regex_case_sensitive_by_default() {
        let m = Matcher::build("HELLO", true).unwrap();
        assert!(m.find_snippet("say hello world").is_none());
    }

    #[test]
    fn regex_inline_case_flag() {
        let m = Matcher::build("(?i)HELLO", true).unwrap();
        assert!(m.find_snippet("say hello world").is_some());
    }

    #[test]
    fn snippet_context_adds_ellipsis() {
        let text = format!("{}TARGET{}", "a".repeat(50), "b".repeat(50));
        let m = Matcher::build("TARGET", false).unwrap();
        let snip = m.find_snippet(&text).unwrap();
        assert!(snip.starts_with('…'));
        assert!(snip.ends_with('…'));
        assert!(snip.contains("TARGET"));
    }

    #[test]
    fn snippet_no_ellipsis_for_short_text() {
        let m = Matcher::build("hi", false).unwrap();
        let snip = m.find_snippet("hi").unwrap();
        assert!(!snip.starts_with('…'));
        assert!(!snip.ends_with('…'));
    }

    #[test]
    fn snippet_normalizes_whitespace() {
        let m = Matcher::build("bar", false).unwrap();
        let snip = m.find_snippet("foo   bar\nbaz").unwrap();
        assert!(!snip.contains('\n'));
        assert!(!snip.contains("   "));
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
}
