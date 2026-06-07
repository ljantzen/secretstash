mod cli;
mod commands;
mod config;
mod crypto;
mod db;
mod session;

use anyhow::Result;
use clap::Parser;
use cli::{AuthAction, Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { action } => match action {
            AuthAction::Login => commands::auth::login(),
            AuthAction::Logout => commands::auth::logout(),
        },
        Commands::Add {
            item_type,
            shortname,
            edit,
            stdin,
            tags,
            text,
        } => commands::add::add(item_type, &shortname, edit, stdin, &tags, text.as_deref()),
        Commands::Show { verbose, shortname } => commands::show::show(&shortname, verbose),
        Commands::History { shortname } => commands::history::history(&shortname),
        Commands::Edit { shortname } => commands::edit::edit(&shortname),
        Commands::Web { private, shortname } => commands::web::web(&shortname, private),
        Commands::Purge { shortname } => commands::purge::purge(&shortname),
        Commands::Tag { shortname, tags } => commands::tag::add_tags(&shortname, &tags),
        Commands::Untag { shortname, tags } => commands::tag::remove_tags(&shortname, &tags),
        Commands::Find { query, tag } => commands::find::find(query.as_deref(), tag.as_deref()),
    }
}
