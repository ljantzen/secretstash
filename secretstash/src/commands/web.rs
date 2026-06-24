use anyhow::{Result, anyhow};

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

pub fn web(
    shortname: &str,
    private: bool,
    cli_browser: Option<&str>,
    cfg_browser: Option<&str>,
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
        .map(normalize_browser);

    match (browser, private) {
        (Some(b), false) => open_with(b, &url),
        (Some(b), true) => open_private_with(b, &url),
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

fn open_private_with(browser: &str, url: &str) -> Result<()> {
    let flag = PRIVATE_FLAGS
        .iter()
        .find(|(name, _)| *name == browser)
        .map(|(_, flag)| *flag)
        .ok_or_else(|| {
            anyhow!(
                "Unknown private-mode flag for '{browser}'. \
                 Known browsers: firefox (--private-window), \
                 google-chrome / chromium / brave-browser / vivaldi (--incognito)."
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
}
