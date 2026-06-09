use anyhow::{Result, anyhow};

use crate::{db::Db, session};

pub fn set_browser(
    shortname: &str,
    browser: Option<&str>,
    clear: bool,
    db_path: &std::path::Path,
) -> Result<()> {
    if browser.is_some() && clear {
        return Err(anyhow!("Cannot use --clear together with a browser name"));
    }
    if browser.is_none() && !clear {
        return Err(anyhow!(
            "Provide a browser name or pass --clear to remove the stored preference"
        ));
    }

    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    let item = db
        .get_item(shortname)?
        .ok_or_else(|| anyhow!("Item '{}' not found", shortname))?;

    if item.item_type != "url" {
        return Err(anyhow!(
            "'{}' is type '{}', not a URL",
            shortname,
            item.item_type
        ));
    }

    db.set_browser(shortname, browser)?;

    if clear {
        println!("Cleared browser preference for '{}'.", shortname);
    } else {
        println!("Set browser for '{}' to '{}'.", shortname, browser.unwrap());
    }
    Ok(())
}
