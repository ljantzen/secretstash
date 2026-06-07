use anyhow::{Result, anyhow};
use std::io::{self, BufRead, Write};

use crate::{db::Db, session};

pub fn purge(shortname: &str, db_path: &std::path::Path) -> Result<()> {
    session::load_key()?;
    let db = Db::open(db_path)?;

    if !db.item_exists(shortname)? {
        return Err(anyhow!("Item '{}' not found", shortname));
    }

    print!("Delete '{}' and all its history? [y/N] ", shortname);
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;

    if line.trim().eq_ignore_ascii_case("y") {
        db.delete_item(shortname)?;
        println!("Purged '{}'.", shortname);
    } else {
        println!("Aborted.");
    }

    Ok(())
}
