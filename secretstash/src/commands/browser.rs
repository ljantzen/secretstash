use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn set_browser_all(
    browser: Option<&str>,
    clear: bool,
    private: Option<bool>,
    db_path: &std::path::Path,
) -> Result<()> {
    if browser.is_some() && clear {
        return Err(anyhow!("Cannot use --clear together with a browser name"));
    }
    if browser.is_none() && !clear && private.is_none() {
        return Err(anyhow!(
            "Provide a browser name, --clear, --private, or --no-private"
        ));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let shortnames: Vec<String> = db
        .list_items()?
        .into_iter()
        .filter(|item| item.item_type == "url")
        .map(|item| item.shortname)
        .collect();

    for shortname in &shortnames {
        if browser.is_some() || clear {
            db.set_browser(shortname, browser)?;
        }
        if let Some(p) = private {
            db.set_private(shortname, if p { Some(true) } else { None })?;
        }
    }

    if browser.is_some() || clear {
        if clear {
            println!(
                "Cleared browser preference for {} URL item(s).",
                shortnames.len()
            );
        } else {
            println!(
                "Set browser to '{}' for {} URL item(s).",
                browser.unwrap(),
                shortnames.len()
            );
        }
    }
    if let Some(p) = private {
        if p {
            println!(
                "Set {} URL item(s) to always open in private mode.",
                shortnames.len()
            );
        } else {
            println!(
                "Cleared private-mode preference for {} URL item(s).",
                shortnames.len()
            );
        }
    }

    Ok(())
}

pub fn set_browser(
    shortname: &str,
    browser: Option<&str>,
    clear: bool,
    private: Option<bool>,
    db_path: &std::path::Path,
) -> Result<()> {
    if browser.is_some() && clear {
        return Err(anyhow!("Cannot use --clear together with a browser name"));
    }
    if browser.is_none() && !clear && private.is_none() {
        return Err(anyhow!(
            "Provide a browser name, --clear, --private, or --no-private"
        ));
    }

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

    if browser.is_some() || clear {
        db.set_browser(shortname, browser)?;
        if clear {
            println!("Cleared browser preference for '{shortname}'.");
        } else {
            println!("Set browser for '{}' to '{}'.", shortname, browser.unwrap());
        }
    }

    if let Some(p) = private {
        db.set_private(shortname, if p { Some(true) } else { None })?;
        if p {
            println!("Set '{shortname}' to always open in private mode.");
        } else {
            println!("Cleared private-mode preference for '{shortname}'.");
        }
    }

    Ok(())
}
