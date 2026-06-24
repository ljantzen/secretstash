use anyhow::{Result, anyhow};
use std::io::{self, BufRead, Write};

use crate::{db::Db, session};

pub fn purge(shortname: &str, force: bool, db_path: &std::path::Path) -> Result<()> {
    let key = session::load_key()?;
    let db = Db::open(db_path, &key)?;

    if !db.item_exists(shortname)? {
        return Err(anyhow!("Item '{shortname}' not found"));
    }

    if !force {
        print!("Delete '{shortname}' and all its history? [y/N] ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;

        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    db.delete_item(shortname)?;
    println!("Purged '{shortname}'.");
    Ok(())
}
