mod cli;

use std::path::PathBuf;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{AuthAction, Cli, Commands};
use secretstash::{commands, config};

#[allow(clippy::too_many_lines)]
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
            AuthAction::Login { timeout } => {
                let timeout = timeout.or(cfg.session_timeout_minutes).unwrap_or(15);
                commands::auth::login(&db_path, timeout)
            }
            AuthAction::Status => commands::auth::status(),
            AuthAction::Logout => commands::auth::logout(),
            AuthAction::Reset => commands::auth::reset(&db_path),
        },
        Commands::Add {
            item_type,
            shortname,
            edit,
            stdin,
            title,
            tags,
            browser,
            private,
            no_private,
            text,
        } => {
            let private_pref = if private {
                Some(true)
            } else if no_private {
                Some(false)
            } else {
                None
            };
            commands::add::add(
                &item_type.to_string(),
                &shortname,
                edit,
                stdin,
                &tags,
                text.as_deref(),
                title.as_deref(),
                browser.as_deref(),
                private_pref,
                &db_path,
            )
        }
        Commands::Show {
            verbose,
            copy,
            clear_after,
            shortname,
        } => {
            let clear_secs = clear_after.or(cfg.clipboard_clear_seconds).unwrap_or(0);
            commands::show::show(&shortname, verbose, copy, clear_secs, &db_path)
        }
        Commands::History { shortname } => commands::history::history(&shortname, &db_path),
        Commands::Edit { shortname, title } => {
            commands::edit::edit(&shortname, title.as_deref(), &db_path)
        }
        Commands::Web {
            private,
            browser,
            shortname,
        } => commands::web::web(
            &shortname,
            private,
            browser.as_deref(),
            cfg.browser.as_deref(),
            cfg.private,
            &cfg.browser_flags,
            &db_path,
        ),
        Commands::Purge { force, shortname } => commands::purge::purge(&shortname, force, &db_path),
        Commands::List { tags, item_type } => commands::list::list(
            &tags,
            item_type
                .as_ref()
                .map(std::string::ToString::to_string)
                .as_deref(),
            &db_path,
        ),
        Commands::Tag { shortname, tags } => commands::tag::add_tags(&shortname, &tags, &db_path),
        Commands::Untag { shortname, tags } => {
            commands::tag::remove_tags(&shortname, &tags, &db_path)
        }
        Commands::Search {
            pattern,
            regex,
            include_history,
            tags,
            item_type,
        } => commands::search::search(
            pattern.as_deref(),
            regex,
            include_history,
            &tags,
            item_type
                .as_ref()
                .map(std::string::ToString::to_string)
                .as_deref(),
            &db_path,
        ),
        Commands::Tags => commands::tags::tags(&db_path),
        Commands::Rename {
            shortname,
            new_name,
        } => commands::rename::rename(&shortname, &new_name, &db_path),
        Commands::Restore { shortname, version } => {
            commands::restore::restore(&shortname, version, &db_path)
        }
        Commands::Copy { shortname, dest } => commands::copy::copy(&shortname, &dest, &db_path),
        Commands::Browser {
            browser,
            shortnames,
            all,
            clear,
            private,
            no_private,
        } => {
            let private_pref = if private {
                Some(true)
            } else if no_private {
                Some(false)
            } else {
                None
            };
            let browser_opt = if clear { None } else { Some(browser.as_str()) };
            if all {
                commands::browser::set_browser_all(browser_opt, clear, private_pref, &db_path)
            } else {
                for shortname in &shortnames {
                    commands::browser::set_browser(
                        shortname,
                        browser_opt,
                        clear,
                        private_pref,
                        &db_path,
                    )?;
                }
                Ok(())
            }
        }
        Commands::Import { file, overwrite } => {
            commands::import::import(overwrite, file.as_deref(), &db_path)
        }
        Commands::Export {
            output,
            include_history,
        } => commands::export::export(include_history, output.as_deref(), &db_path),
        Commands::Migrate => commands::migrate::migrate(&db_path),
        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "stash", &mut std::io::stdout());
            Ok(())
        }
    }
}
