mod cli;
mod commands;
mod config;
mod crypto;
mod db;
mod keychain;
mod session;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use cli::{AuthAction, Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let cfg = config::load_config()?;

    let db_path: PathBuf = if let Some(p) = cli.db {
        p
    } else if let Ok(p) = std::env::var("STASH_DB") {
        PathBuf::from(p)
    } else if let Some(p) = cfg.db {
        p
    } else {
        config::db_path()?
    };

    match cli.command {
        Commands::Auth { action } => match action {
            AuthAction::Login => {
                let timeout = cfg.session_timeout_minutes.unwrap_or(15);
                commands::auth::login(&db_path, timeout)
            }
            AuthAction::Logout => commands::auth::logout(),
        },
        Commands::Add {
            item_type,
            shortname,
            edit,
            stdin,
            tags,
            browser,
            text,
        } => commands::add::add(
            item_type,
            &shortname,
            edit,
            stdin,
            &tags,
            text.as_deref(),
            browser.as_deref(),
            &db_path,
        ),
        Commands::Show {
            verbose,
            copy,
            shortname,
        } => commands::show::show(&shortname, verbose, copy, &db_path),
        Commands::History { shortname } => commands::history::history(&shortname, &db_path),
        Commands::Edit { shortname } => commands::edit::edit(&shortname, &db_path),
        Commands::Web {
            private,
            browser,
            shortname,
        } => commands::web::web(
            &shortname,
            private,
            browser.as_deref(),
            cfg.browser.as_deref(),
            &db_path,
        ),
        Commands::Purge { force, shortname } => commands::purge::purge(&shortname, force, &db_path),
        Commands::List { tags, item_type } => commands::list::list(
            &tags,
            item_type.as_ref().map(|t| t.to_string()).as_deref(),
            &db_path,
        ),
        Commands::Tag { shortname, tags } => commands::tag::add_tags(&shortname, &tags, &db_path),
        Commands::Untag { shortname, tags } => {
            commands::tag::remove_tags(&shortname, &tags, &db_path)
        }
        Commands::Find {
            query,
            tag,
            item_type,
        } => commands::find::find(
            query.as_deref(),
            tag.as_deref(),
            item_type.as_ref().map(|t| t.to_string()).as_deref(),
            &db_path,
        ),
        Commands::Rename {
            shortname,
            new_name,
        } => commands::rename::rename(&shortname, &new_name, &db_path),
        Commands::Restore { shortname, version } => {
            commands::restore::restore(&shortname, version, &db_path)
        }
        Commands::Copy { shortname, dest } => commands::copy::copy(&shortname, &dest, &db_path),
    }
}
