use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "stash",
    about = "Encrypted notes, URLs, and secrets manager",
    version
)]
pub struct Cli {
    /// Use this database file instead of the default
    #[arg(long, value_name = "FILE", global = true)]
    pub db: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Add a new item
    Add {
        /// Item type
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        item_type: ItemType,
        /// Short name (identifier)
        #[arg(short = 's', long)]
        shortname: String,
        /// Open $EDITOR to compose content
        #[arg(short = 'e', long)]
        edit: bool,
        /// Read content from stdin
        #[arg(long)]
        stdin: bool,
        /// Tag(s) to attach (repeatable: --tag work --tag personal)
        #[arg(short = 'g', long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Preferred browser for opening this URL (url items only)
        #[arg(short = 'b', long)]
        browser: Option<String>,
        /// Content text
        text: Option<String>,
    },
    /// Show an item's content
    Show {
        /// Show metadata (type, timestamps) in addition to content
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Copy content to clipboard instead of printing
        #[arg(short = 'c', long)]
        copy: bool,
        shortname: String,
    },
    /// Show version history of an item
    History { shortname: String },
    /// Edit an item in $EDITOR
    Edit { shortname: String },
    /// Open a URL item in the browser
    Web {
        /// Open in private/incognito mode
        #[arg(short = 'p', long)]
        private: bool,
        /// Browser binary to use (overrides config `browser`)
        #[arg(short = 'b', long)]
        browser: Option<String>,
        shortname: String,
    },
    /// Delete an item and its entire history
    Purge {
        /// Skip confirmation prompt
        #[arg(short = 'f', long)]
        force: bool,
        shortname: String,
    },
    /// List all items, optionally filtered by tag(s) and/or type
    List {
        /// Show items that have ANY of the specified tags (repeatable)
        #[arg(short = 'g', long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Filter by item type
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        item_type: Option<ItemType>,
    },
    /// Add tags to an existing item
    Tag {
        shortname: String,
        /// Tags to add (at least one required)
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Remove tags from an item
    Untag {
        shortname: String,
        /// Tags to remove (at least one required)
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Search across all item content and/or tags (case-insensitive)
    Find {
        /// Filter by tag
        #[arg(short = 'g', long = "tag", value_name = "TAG")]
        tag: Option<String>,
        /// Filter by item type
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        item_type: Option<ItemType>,
        /// Search term (required unless --tag or --type is used)
        query: Option<String>,
    },
    /// Rename an item
    Rename { shortname: String, new_name: String },
    /// Restore an item to a previous version
    Restore {
        shortname: String,
        /// Version number to restore (default: most recent archived version)
        #[arg(long, value_name = "N")]
        version: Option<i64>,
    },
    /// Copy an item to a new shortname
    Copy { shortname: String, dest: String },
    /// Set or clear the stored browser preference for a URL item
    Browser {
        shortname: String,
        /// Browser binary to use (e.g. firefox, chromium)
        browser: Option<String>,
        /// Remove the stored browser preference
        #[arg(long)]
        clear: bool,
    },
    /// Migrate an existing vault from the old field-level-encrypted format
    /// to whole-database SQLCipher encryption
    Migrate,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Authenticate with master password
    Login,
    /// Clear the current session
    Logout,
    /// Change the master password and re-encrypt the vault
    Reset,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ItemType {
    Url,
    Note,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Url => write!(f, "url"),
            ItemType::Note => write!(f, "note"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    // ── auth ──────────────────────────────────────────────────────────────

    #[test]
    fn auth_login() {
        let cli = parse(&["stash", "auth", "login"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Auth {
                action: AuthAction::Login
            }
        ));
    }

    #[test]
    fn auth_logout() {
        let cli = parse(&["stash", "auth", "logout"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Auth {
                action: AuthAction::Logout
            }
        ));
    }

    // ── add ───────────────────────────────────────────────────────────────

    #[test]
    fn add_inline_text() {
        let cli = parse(&["stash", "add", "-t", "note", "-s", "k", "hello"]).unwrap();
        if let Commands::Add {
            item_type,
            shortname,
            edit,
            stdin,
            text,
            ..
        } = cli.command
        {
            assert!(matches!(item_type, ItemType::Note));
            assert_eq!(shortname, "k");
            assert!(!edit);
            assert!(!stdin);
            assert_eq!(text.as_deref(), Some("hello"));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_long_flags() {
        let cli = parse(&[
            "stash",
            "add",
            "--type",
            "url",
            "--shortname",
            "gh",
            "https://x.com",
        ])
        .unwrap();
        if let Commands::Add {
            item_type,
            shortname,
            ..
        } = cli.command
        {
            assert!(matches!(item_type, ItemType::Url));
            assert_eq!(shortname, "gh");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_rejects_secret_type() {
        assert!(parse(&["stash", "add", "-t", "secret", "-s", "k", "v"]).is_err());
    }

    #[test]
    fn add_edit_flag() {
        let cli = parse(&["stash", "add", "-t", "note", "-s", "k", "-e"]).unwrap();
        if let Commands::Add {
            edit, stdin, text, ..
        } = cli.command
        {
            assert!(edit);
            assert!(!stdin);
            assert!(text.is_none());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_stdin_flag() {
        let cli = parse(&["stash", "add", "-t", "note", "-s", "k", "--stdin"]).unwrap();
        if let Commands::Add { edit, stdin, .. } = cli.command {
            assert!(!edit);
            assert!(stdin);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_requires_type() {
        assert!(parse(&["stash", "add", "-s", "k", "text"]).is_err());
    }

    #[test]
    fn add_requires_shortname() {
        assert!(parse(&["stash", "add", "-t", "note", "text"]).is_err());
    }

    #[test]
    fn add_rejects_invalid_type() {
        assert!(parse(&["stash", "add", "-t", "image", "-s", "k"]).is_err());
    }

    // ── show ──────────────────────────────────────────────────────────────

    #[test]
    fn show_default_not_verbose() {
        let cli = parse(&["stash", "show", "k"]).unwrap();
        if let Commands::Show {
            shortname, verbose, ..
        } = cli.command
        {
            assert_eq!(shortname, "k");
            assert!(!verbose);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn show_verbose_short() {
        let cli = parse(&["stash", "show", "-v", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::Show { verbose: true, .. }));
    }

    #[test]
    fn show_verbose_long() {
        let cli = parse(&["stash", "show", "--verbose", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::Show { verbose: true, .. }));
    }

    // ── web ───────────────────────────────────────────────────────────────

    #[test]
    fn web_no_private() {
        let cli = parse(&["stash", "web", "myurl"]).unwrap();
        if let Commands::Web {
            private, shortname, ..
        } = cli.command
        {
            assert!(!private);
            assert_eq!(shortname, "myurl");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn web_private_short() {
        let cli = parse(&["stash", "web", "-p", "myurl"]).unwrap();
        assert!(matches!(cli.command, Commands::Web { private: true, .. }));
    }

    #[test]
    fn web_private_long() {
        let cli = parse(&["stash", "web", "--private", "myurl"]).unwrap();
        assert!(matches!(cli.command, Commands::Web { private: true, .. }));
    }

    // ── list ──────────────────────────────────────────────────────────────

    #[test]
    fn list_no_filter() {
        let cli = parse(&["stash", "list"]).unwrap();
        if let Commands::List { tags, .. } = cli.command {
            assert!(tags.is_empty());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn list_single_tag() {
        let cli = parse(&["stash", "list", "--tag", "work"]).unwrap();
        if let Commands::List { tags, .. } = cli.command {
            assert_eq!(tags, ["work"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn list_multiple_tags() {
        let cli = parse(&["stash", "list", "-g", "work", "-g", "personal"]).unwrap();
        if let Commands::List { tags, .. } = cli.command {
            assert_eq!(tags, ["work", "personal"]);
        } else {
            panic!("wrong variant");
        }
    }

    // ── other subcommands ─────────────────────────────────────────────────

    #[test]
    fn parse_history() {
        let cli = parse(&["stash", "history", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::History { shortname } if shortname == "k"));
    }

    #[test]
    fn parse_edit() {
        let cli = parse(&["stash", "edit", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::Edit { shortname } if shortname == "k"));
    }

    #[test]
    fn parse_purge() {
        let cli = parse(&["stash", "purge", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::Purge { shortname, .. } if shortname == "k"));
    }

    #[test]
    fn add_with_tags() {
        let cli = parse(&[
            "stash", "add", "-t", "note", "-s", "k", "--tag", "work", "--tag", "personal", "text",
        ])
        .unwrap();
        if let Commands::Add { tags, .. } = cli.command {
            assert_eq!(tags, ["work", "personal"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn parse_tag() {
        let cli = parse(&["stash", "tag", "myitem", "work", "personal"]).unwrap();
        if let Commands::Tag { shortname, tags } = cli.command {
            assert_eq!(shortname, "myitem");
            assert_eq!(tags, ["work", "personal"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn tag_requires_at_least_one_tag() {
        assert!(parse(&["stash", "tag", "myitem"]).is_err());
    }

    #[test]
    fn parse_untag() {
        let cli = parse(&["stash", "untag", "myitem", "work"]).unwrap();
        if let Commands::Untag { shortname, tags } = cli.command {
            assert_eq!(shortname, "myitem");
            assert_eq!(tags, ["work"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn untag_requires_at_least_one_tag() {
        assert!(parse(&["stash", "untag", "myitem"]).is_err());
    }

    #[test]
    fn find_query_only() {
        let cli = parse(&["stash", "find", "search term"]).unwrap();
        if let Commands::Find { query, tag, .. } = cli.command {
            assert_eq!(query.as_deref(), Some("search term"));
            assert!(tag.is_none());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn find_tag_only() {
        let cli = parse(&["stash", "find", "--tag", "work"]).unwrap();
        if let Commands::Find { query, tag, .. } = cli.command {
            assert!(query.is_none());
            assert_eq!(tag.as_deref(), Some("work"));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn find_query_and_tag() {
        let cli = parse(&["stash", "find", "--tag", "work", "term"]).unwrap();
        if let Commands::Find { query, tag, .. } = cli.command {
            assert_eq!(query.as_deref(), Some("term"));
            assert_eq!(tag.as_deref(), Some("work"));
        } else {
            panic!("wrong variant");
        }
    }

    // ── --db global flag ─────────────────────────────────────────────────

    #[test]
    fn db_flag_before_subcommand() {
        let cli = parse(&["stash", "--db", "/tmp/test.db", "list"]).unwrap();
        assert_eq!(
            cli.db.as_deref(),
            Some(std::path::Path::new("/tmp/test.db"))
        );
    }

    #[test]
    fn db_flag_after_subcommand() {
        let cli = parse(&["stash", "list", "--db", "/tmp/test.db"]).unwrap();
        assert_eq!(
            cli.db.as_deref(),
            Some(std::path::Path::new("/tmp/test.db"))
        );
    }

    #[test]
    fn db_flag_absent() {
        let cli = parse(&["stash", "list"]).unwrap();
        assert!(cli.db.is_none());
    }

    // ── browser ───────────────────────────────────────────────────────────

    #[test]
    fn browser_set() {
        let cli = parse(&["stash", "browser", "myurl", "firefox"]).unwrap();
        if let Commands::Browser {
            shortname,
            browser,
            clear,
        } = cli.command
        {
            assert_eq!(shortname, "myurl");
            assert_eq!(browser.as_deref(), Some("firefox"));
            assert!(!clear);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_clear() {
        let cli = parse(&["stash", "browser", "myurl", "--clear"]).unwrap();
        if let Commands::Browser {
            shortname,
            browser,
            clear,
        } = cli.command
        {
            assert_eq!(shortname, "myurl");
            assert!(browser.is_none());
            assert!(clear);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_no_args_is_ok_at_parse_level() {
        let cli = parse(&["stash", "browser", "myurl"]).unwrap();
        assert!(matches!(cli.command, Commands::Browser { .. }));
    }

    // ── migrate ───────────────────────────────────────────────────────────

    #[test]
    fn parse_migrate() {
        let cli = parse(&["stash", "migrate"]).unwrap();
        assert!(matches!(cli.command, Commands::Migrate));
    }

    // ── ItemType display ──────────────────────────────────────────────────

    #[test]
    fn item_type_display() {
        assert_eq!(ItemType::Url.to_string(), "url");
        assert_eq!(ItemType::Note.to_string(), "note");
    }
}
