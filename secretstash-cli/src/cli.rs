use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
pub use clap_complete::Shell;

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
        /// Item type: url or note
        item_type: ItemType,
        /// Short name (identifier)
        shortname: String,
        /// Open $EDITOR to compose content
        #[arg(short = 'e', long)]
        edit: bool,
        /// Read content from stdin
        #[arg(long)]
        stdin: bool,
        /// Human-readable title
        #[arg(short = 't', long, value_name = "TITLE")]
        title: Option<String>,
        /// Tag(s) to attach (repeatable: --tag work --tag personal)
        #[arg(short = 'g', long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Preferred browser for opening this URL (url items only)
        #[arg(short = 'b', long)]
        browser: Option<String>,
        /// Always open this URL in private/incognito mode (url items only)
        #[arg(short = 'p', long, conflicts_with = "no_private")]
        private: bool,
        /// Clear the stored private-mode preference (url items only)
        #[arg(short = 'n', long, conflicts_with = "private")]
        no_private: bool,
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
        /// Clear clipboard after this many seconds (requires --copy; overrides config)
        #[arg(long, value_name = "SECONDS")]
        clear_after: Option<u64>,
        shortname: String,
    },
    /// Show version history of an item
    History { shortname: String },
    /// Edit an item in $EDITOR
    Edit {
        shortname: String,
        /// Set a new title (updates title only, does not open editor)
        #[arg(short = 't', long, value_name = "TITLE")]
        title: Option<String>,
    },
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
    /// Search items by content, title, tag, and/or type
    Search {
        /// Text or regex to match against content and titles (required unless --tag or --type is given)
        pattern: Option<String>,
        /// Treat pattern as a regular expression (case-sensitive; use (?i) for case-insensitive)
        #[arg(short = 'r', long)]
        regex: bool,
        /// Also search archived versions (shown as name:vN)
        #[arg(short = 'H', long)]
        include_history: bool,
        /// Filter by tag (repeatable; matches items with ANY of the given tags)
        #[arg(short = 'g', long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Filter by item type
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        item_type: Option<ItemType>,
    },
    /// List all tags used in the vault with item counts
    Tags,
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
    /// Set or clear the stored browser preference for one or more URL items
    Browser {
        /// Browser binary to use (e.g. firefox, chromium)
        browser: String,
        /// URL item name(s) to configure; omit when using --all
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        shortnames: Vec<String>,
        /// Apply to all URL items in the vault
        #[arg(long)]
        all: bool,
        /// Remove the stored browser preference
        #[arg(long)]
        clear: bool,
        /// Always open this URL in private/incognito mode
        #[arg(short = 'p', long, conflicts_with = "no_private")]
        private: bool,
        /// Clear the stored private-mode preference
        #[arg(short = 'n', long, conflicts_with = "private")]
        no_private: bool,
    },
    /// Import items from a JSON export file (reads stdin if FILE is omitted)
    Import {
        /// Path to the export file
        file: Option<PathBuf>,
        /// Replace existing items instead of skipping them
        #[arg(long)]
        overwrite: bool,
    },
    /// Export all vault items to JSON
    Export {
        /// Write output to this file instead of stdout
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Include full version history for each item
        #[arg(long)]
        include_history: bool,
    },
    /// Migrate an existing vault from the old field-level-encrypted format
    /// to whole-database `SQLCipher` encryption
    Migrate,
    /// Print a shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Authenticate with master password
    Login {
        /// Session timeout in minutes (0 = never expire); overrides config
        #[arg(long, value_name = "MINUTES")]
        timeout: Option<u64>,
    },
    /// Show current session status
    Status,
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
    fn auth_login_no_timeout() {
        let cli = parse(&["stash", "auth", "login"]).unwrap();
        if let Commands::Auth {
            action: AuthAction::Login { timeout },
        } = cli.command
        {
            assert!(timeout.is_none());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn auth_login_with_timeout() {
        let cli = parse(&["stash", "auth", "login", "--timeout", "60"]).unwrap();
        if let Commands::Auth {
            action: AuthAction::Login { timeout },
        } = cli.command
        {
            assert_eq!(timeout, Some(60));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn auth_login_timeout_zero_disables_expiry() {
        let cli = parse(&["stash", "auth", "login", "--timeout", "0"]).unwrap();
        if let Commands::Auth {
            action: AuthAction::Login { timeout },
        } = cli.command
        {
            assert_eq!(timeout, Some(0));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn auth_status() {
        let cli = parse(&["stash", "auth", "status"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Auth {
                action: AuthAction::Status
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
        let cli = parse(&["stash", "add", "note", "k", "hello"]).unwrap();
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
    fn add_url_positional() {
        let cli = parse(&["stash", "add", "url", "gh", "https://x.com"]).unwrap();
        if let Commands::Add {
            item_type,
            shortname,
            text,
            ..
        } = cli.command
        {
            assert!(matches!(item_type, ItemType::Url));
            assert_eq!(shortname, "gh");
            assert_eq!(text.as_deref(), Some("https://x.com"));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_rejects_invalid_type() {
        assert!(parse(&["stash", "add", "secret", "k", "v"]).is_err());
        assert!(parse(&["stash", "add", "image", "k"]).is_err());
    }

    #[test]
    fn add_edit_flag() {
        let cli = parse(&["stash", "add", "note", "k", "-e"]).unwrap();
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
        let cli = parse(&["stash", "add", "note", "k", "--stdin"]).unwrap();
        if let Commands::Add { edit, stdin, .. } = cli.command {
            assert!(!edit);
            assert!(stdin);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_requires_type_and_shortname() {
        assert!(parse(&["stash", "add"]).is_err());
        assert!(parse(&["stash", "add", "note"]).is_err());
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
        assert!(matches!(cli.command, Commands::Edit { shortname, .. } if shortname == "k"));
    }

    #[test]
    fn parse_purge() {
        let cli = parse(&["stash", "purge", "k"]).unwrap();
        assert!(matches!(cli.command, Commands::Purge { shortname, .. } if shortname == "k"));
    }

    #[test]
    fn add_with_tags() {
        let cli = parse(&[
            "stash", "add", "note", "k", "--tag", "work", "--tag", "personal", "text",
        ])
        .unwrap();
        if let Commands::Add { tags, .. } = cli.command {
            assert_eq!(tags, ["work", "personal"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_browser_flag() {
        let cli = parse(&["stash", "add", "url", "k", "--browser", "firefox"]).unwrap();
        if let Commands::Add { browser, .. } = cli.command {
            assert_eq!(browser.as_deref(), Some("firefox"));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_browser_short_flag() {
        let cli = parse(&["stash", "add", "url", "k", "-b", "chromium"]).unwrap();
        if let Commands::Add { browser, .. } = cli.command {
            assert_eq!(browser.as_deref(), Some("chromium"));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_private_flag() {
        let cli = parse(&["stash", "add", "url", "k", "--private"]).unwrap();
        if let Commands::Add {
            private,
            no_private,
            ..
        } = cli.command
        {
            assert!(private);
            assert!(!no_private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_private_short_flag() {
        let cli = parse(&["stash", "add", "url", "k", "-p"]).unwrap();
        if let Commands::Add { private, .. } = cli.command {
            assert!(private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_no_private_flag() {
        let cli = parse(&["stash", "add", "url", "k", "--no-private"]).unwrap();
        if let Commands::Add {
            private,
            no_private,
            ..
        } = cli.command
        {
            assert!(!private);
            assert!(no_private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_no_private_short_flag() {
        let cli = parse(&["stash", "add", "url", "k", "-n"]).unwrap();
        if let Commands::Add { no_private, .. } = cli.command {
            assert!(no_private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn add_private_and_no_private_conflict() {
        assert!(parse(&["stash", "add", "url", "k", "--private", "--no-private"]).is_err());
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
    fn search_pattern_only() {
        let cli = parse(&["stash", "search", "search term"]).unwrap();
        if let Commands::Search { pattern, tags, .. } = cli.command {
            assert_eq!(pattern.as_deref(), Some("search term"));
            assert!(tags.is_empty());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn search_tag_only() {
        let cli = parse(&["stash", "search", "--tag", "work"]).unwrap();
        if let Commands::Search { pattern, tags, .. } = cli.command {
            assert!(pattern.is_none());
            assert_eq!(tags, ["work"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn search_multiple_tags() {
        let cli = parse(&["stash", "search", "--tag", "work", "--tag", "personal"]).unwrap();
        if let Commands::Search { pattern, tags, .. } = cli.command {
            assert!(pattern.is_none());
            assert_eq!(tags, ["work", "personal"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn search_pattern_and_tag() {
        let cli = parse(&["stash", "search", "--tag", "work", "term"]).unwrap();
        if let Commands::Search { pattern, tags, .. } = cli.command {
            assert_eq!(pattern.as_deref(), Some("term"));
            assert_eq!(tags, ["work"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn search_regex_flag() {
        let cli = parse(&["stash", "search", "--regex", r"\d+"]).unwrap();
        if let Commands::Search { pattern, regex, .. } = cli.command {
            assert_eq!(pattern.as_deref(), Some(r"\d+"));
            assert!(regex);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn search_include_history_flag() {
        let cli = parse(&["stash", "search", "-H", "term"]).unwrap();
        if let Commands::Search {
            include_history, ..
        } = cli.command
        {
            assert!(include_history);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn parse_tags() {
        let cli = parse(&["stash", "tags"]).unwrap();
        assert!(matches!(cli.command, Commands::Tags));
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
        let cli = parse(&["stash", "browser", "firefox", "myurl"]).unwrap();
        if let Commands::Browser {
            browser,
            shortnames,
            clear,
            ..
        } = cli.command
        {
            assert_eq!(browser, "firefox");
            assert_eq!(shortnames, ["myurl"]);
            assert!(!clear);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_set_multiple() {
        let cli = parse(&["stash", "browser", "firefox", "url1", "url2", "url3"]).unwrap();
        if let Commands::Browser {
            browser,
            shortnames,
            ..
        } = cli.command
        {
            assert_eq!(browser, "firefox");
            assert_eq!(shortnames, ["url1", "url2", "url3"]);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_clear() {
        let cli = parse(&["stash", "browser", "firefox", "myurl", "--clear"]).unwrap();
        if let Commands::Browser {
            shortnames, clear, ..
        } = cli.command
        {
            assert_eq!(shortnames, ["myurl"]);
            assert!(clear);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_private_flag() {
        let cli = parse(&["stash", "browser", "firefox", "myurl", "--private"]).unwrap();
        if let Commands::Browser {
            shortnames,
            private,
            no_private,
            ..
        } = cli.command
        {
            assert_eq!(shortnames, ["myurl"]);
            assert!(private);
            assert!(!no_private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_no_private_flag() {
        let cli = parse(&["stash", "browser", "firefox", "myurl", "--no-private"]).unwrap();
        if let Commands::Browser {
            private,
            no_private,
            ..
        } = cli.command
        {
            assert!(!private);
            assert!(no_private);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_requires_at_least_one_shortname() {
        assert!(parse(&["stash", "browser", "firefox"]).is_err());
    }

    #[test]
    fn browser_all_flag() {
        let cli = parse(&["stash", "browser", "firefox", "--all"]).unwrap();
        if let Commands::Browser {
            browser,
            all,
            shortnames,
            ..
        } = cli.command
        {
            assert_eq!(browser, "firefox");
            assert!(all);
            assert!(shortnames.is_empty());
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn browser_all_conflicts_with_shortnames() {
        assert!(parse(&["stash", "browser", "firefox", "url1", "--all"]).is_err());
    }

    // ── import ────────────────────────────────────────────────────────────

    #[test]
    fn import_defaults() {
        let cli = parse(&["stash", "import"]).unwrap();
        if let Commands::Import { file, overwrite } = cli.command {
            assert!(file.is_none());
            assert!(!overwrite);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn import_with_file() {
        let cli = parse(&["stash", "import", "/tmp/vault.json"]).unwrap();
        if let Commands::Import { file, .. } = cli.command {
            assert_eq!(
                file.as_deref(),
                Some(std::path::Path::new("/tmp/vault.json"))
            );
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn import_overwrite() {
        let cli = parse(&["stash", "import", "--overwrite", "/tmp/vault.json"]).unwrap();
        if let Commands::Import { overwrite, .. } = cli.command {
            assert!(overwrite);
        } else {
            panic!("wrong variant");
        }
    }

    // ── export ────────────────────────────────────────────────────────────

    #[test]
    fn export_defaults() {
        let cli = parse(&["stash", "export"]).unwrap();
        if let Commands::Export {
            output,
            include_history,
        } = cli.command
        {
            assert!(output.is_none());
            assert!(!include_history);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn export_with_output_file() {
        let cli = parse(&["stash", "export", "-o", "/tmp/vault.json"]).unwrap();
        if let Commands::Export { output, .. } = cli.command {
            assert_eq!(
                output.as_deref(),
                Some(std::path::Path::new("/tmp/vault.json"))
            );
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn export_include_history() {
        let cli = parse(&["stash", "export", "--include-history"]).unwrap();
        if let Commands::Export {
            include_history, ..
        } = cli.command
        {
            assert!(include_history);
        } else {
            panic!("wrong variant");
        }
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
