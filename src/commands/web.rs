use anyhow::{Result, anyhow};

use crate::{crypto, db::Db, session};

pub fn web(shortname: &str, private: bool, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path)?;

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

    let content = crypto::decrypt(&key, &item.content_enc, &item.nonce)?;
    let url = String::from_utf8(content)?.trim().to_string();

    if private {
        open_private(&url)
    } else {
        open::that(&url)?;
        println!("Opened '{}' in browser.", shortname);
        Ok(())
    }
}

fn open_private(url: &str) -> Result<()> {
    let candidates: &[(&str, &[&str])] = &[
        ("firefox", &["--private-window"]),
        ("google-chrome", &["--incognito"]),
        ("chromium", &["--incognito"]),
        ("chromium-browser", &["--incognito"]),
        ("brave-browser", &["--incognito"]),
    ];

    for (browser, flags) in candidates {
        match std::process::Command::new(browser)
            .args(*flags)
            .arg(url)
            .spawn()
        {
            Ok(_) => {
                println!("Opened in {} (private mode).", browser);
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Err(anyhow!(
        "No supported browser found for private mode. \
         Tried: firefox, google-chrome, chromium, chromium-browser, brave-browser"
    ))
}
