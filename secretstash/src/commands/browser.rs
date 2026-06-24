use anyhow::{Result, anyhow};

use crate::{db::Db, session};

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
