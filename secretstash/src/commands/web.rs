use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::{db::Db, session};

const PRIVATE_FLAGS: &[(&str, &str)] = &[
    ("firefox", "--private-window"),
    ("google-chrome", "--incognito"),
    ("chromium", "--incognito"),
    ("chromium-browser", "--incognito"),
    ("brave-browser", "--incognito"),
    ("vivaldi", "--incognito"),
    ("vivaldi-stable", "--incognito"),
];

/// Canonical browser names used for prefix resolution.
/// Includes aliases (`chrome`) so they participate in prefix matching.
const KNOWN_BROWSERS: &[&str] = &[
    "firefox",
    "google-chrome",
    "chrome",
    "chromium",
    "chromium-browser",
    "brave-browser",
    "vivaldi",
    "vivaldi-stable",
];

/// Resolve a browser name to its canonical form.
///
/// 1. Exact match → normalize (e.g. `chrome` → `google-chrome`).
/// 2. Unique prefix match among known browsers → normalize that match.
/// 3. Ambiguous prefix → error listing the candidates.
/// 4. No match at all → pass through unchanged (unknown/custom browser).
pub fn resolve_browser(input: &str) -> Result<String> {
    if KNOWN_BROWSERS.contains(&input) {
        return Ok(normalize_browser(input).to_string());
    }
    let matches: Vec<&str> = KNOWN_BROWSERS
        .iter()
        .copied()
        .filter(|b| b.starts_with(input))
        .collect();
    match matches.len() {
        0 => Ok(input.to_string()),
        1 => Ok(normalize_browser(matches[0]).to_string()),
        _ => Err(anyhow!(
            "ambiguous browser '{}': matches {}",
            input,
            matches.join(", ")
        )),
    }
}

pub fn web(
    shortname: &str,
    private: bool,
    cli_browser: Option<&str>,
    cfg_browser: Option<&str>,
    cfg_browser_flags: &HashMap<String, String>,
    db_path: &std::path::Path,
) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{shortname}' not found"))?;

    if item.item_type != "url" {
        return Err(anyhow!(
            "'{}' is type '{}', not a URL",
            shortname,
            item.item_type
        ));
    }

    let url = item.content.trim().to_string();
    let private = private || item.private.unwrap_or(false);

    let browser = cli_browser
        .or(item.browser.as_deref())
        .or(cfg_browser)
        .map(resolve_browser)
        .transpose()?;

    match (browser, private) {
        (Some(b), false) => open_with(&b, &url),
        (Some(b), true) => open_private_with(&b, &url, cfg_browser_flags),
        (None, false) => {
            open::that(&url)?;
            println!("Opened '{shortname}' in browser.");
            Ok(())
        }
        (None, true) => open_private_discover(&url),
    }
}

fn open_with(browser: &str, url: &str) -> Result<()> {
    std::process::Command::new(browser)
        .arg(url)
        .spawn()
        .map_err(|e| browser_err(browser, e))?;
    println!("Opened in {browser}.");
    Ok(())
}

fn open_private_with(
    browser: &str,
    url: &str,
    cfg_browser_flags: &HashMap<String, String>,
) -> Result<()> {
    let flag = PRIVATE_FLAGS
        .iter()
        .find(|(name, _)| *name == browser)
        .map(|(_, flag)| *flag)
        .or_else(|| cfg_browser_flags.get(browser).map(String::as_str))
        .ok_or_else(|| {
            anyhow!(
                "Unknown private-mode flag for '{browser}'. \
                 Known browsers: firefox (--private-window), \
                 google-chrome / chromium / brave-browser / vivaldi (--incognito). \
                 Add a [browser_flags] entry in stash.toml to support other browsers."
            )
        })?;

    std::process::Command::new(browser)
        .arg(flag)
        .arg(url)
        .spawn()
        .map_err(|e| browser_err(browser, e))?;
    println!("Opened in {browser} (private mode).");
    Ok(())
}

fn open_private_discover(url: &str) -> Result<()> {
    for (browser, flag) in PRIVATE_FLAGS {
        match std::process::Command::new(browser)
            .arg(flag)
            .arg(url)
            .spawn()
        {
            Ok(_) => {
                println!("Opened in {browser} (private mode).");
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Err(anyhow!(
        "No supported browser found for private mode. \
         Tried: firefox, google-chrome, chromium, chromium-browser, brave-browser, vivaldi. \
         Set `browser` in stash.toml or pass --browser."
    ))
}

fn normalize_browser(browser: &str) -> &str {
    match browser {
        "chrome" => "google-chrome",
        other => other,
    }
}

fn browser_err(browser: &str, e: std::io::Error) -> anyhow::Error {
    if e.kind() == std::io::ErrorKind::NotFound {
        anyhow!("Browser '{browser}' not found")
    } else {
        e.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_normalizes_to_google_chrome() {
        assert_eq!(normalize_browser("chrome"), "google-chrome");
    }

    #[test]
    fn known_browsers_pass_through_unchanged() {
        for (name, _) in PRIVATE_FLAGS {
            assert_eq!(normalize_browser(name), *name);
        }
    }

    #[test]
    fn unknown_browser_passes_through() {
        assert_eq!(normalize_browser("opera"), "opera");
    }

    #[test]
    fn resolve_exact_match() {
        assert_eq!(resolve_browser("firefox").unwrap(), "firefox");
        assert_eq!(resolve_browser("chromium").unwrap(), "chromium");
        assert_eq!(resolve_browser("vivaldi").unwrap(), "vivaldi");
    }

    #[test]
    fn resolve_alias_chrome() {
        assert_eq!(resolve_browser("chrome").unwrap(), "google-chrome");
    }

    #[test]
    fn resolve_unique_prefix() {
        assert_eq!(resolve_browser("fi").unwrap(), "firefox");
        assert_eq!(resolve_browser("fire").unwrap(), "firefox");
        assert_eq!(resolve_browser("br").unwrap(), "brave-browser");
        assert_eq!(resolve_browser("go").unwrap(), "google-chrome");
        assert_eq!(resolve_browser("chromium-b").unwrap(), "chromium-browser");
        assert_eq!(resolve_browser("vivaldi-").unwrap(), "vivaldi-stable");
    }

    #[test]
    fn resolve_ambiguous_prefix_errors() {
        assert!(resolve_browser("vi").is_err()); // vivaldi, vivaldi-stable
        assert!(resolve_browser("ch").is_err()); // chrome, chromium, chromium-browser
    }

    #[test]
    fn resolve_unknown_browser_passes_through() {
        assert_eq!(resolve_browser("opera").unwrap(), "opera");
        assert_eq!(resolve_browser("safari").unwrap(), "safari");
    }
}
