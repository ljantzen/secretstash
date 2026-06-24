# stash

[![CI](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A command-line manager for notes and URLs — stored in an encrypted local database.

Each item is identified by a short name and versioned: every edit is archived so you can review the full history at any time.

## Features

- **Two item types** — `note`, `url`
- **Encrypted at rest** — whole-database SQLCipher encryption (AES-256-CBC); Argon2id key derivation
- **Titles** — optional human-readable title on every item, tracked through version history
- **Tags** — attach multiple tags to any item; filter and search by tag
- **Version history** — every edit is archived; nothing is silently overwritten
- **Editor integration** — `$EDITOR` opens for composing or editing any item
- **Browser integration** — open URL items directly, with optional private/incognito mode; store a per-item preferred browser and a per-item always-private preference
- **Export / import** — dump the vault to a portable JSON file and restore it to any vault
- **Self-contained** — bundles SQLite; no external database or service required

## Disclaimer

Although I believe that the cryptographic primitives implemented in this program are sound, they have not undergone a
security review. The encryption provided will provide protection from snooping family members, and probably colleagues
(depending on where you work). But, if your notes and urls need protection from serious and determined hackers maybe
you should store them elsewhere. Also review the `Security notes` near the end of this document. Consider yourself notified.

## Installation

### Pre-built binaries

Download the latest release for your platform from the [releases page](https://github.com/ljantzen/secretstash/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 (glibc) | `secretstash-linux-x86_64.tar.gz` |
| Linux x86_64 (musl / static) | `secretstash-linux-x86_64-musl.tar.gz` |
| macOS aarch64 (Apple Silicon) | `secretstash-macos-aarch64.tar.gz` |
| Windows x86_64 | `secretstash-windows-x86_64.zip` |

Extract and place the `stash` binary somewhere on your `$PATH`.

### From source

Requires Rust 1.85+.

```sh
cargo install --path secretstash-cli
```

## Configuration

### Config file

stash reads `~/.config/stash/stash.toml` on startup (macOS:
`~/Library/Application Support/stash/stash.toml`). Create the directory and
file if it does not exist.

```toml
# ~/.config/stash/stash.toml
db = "/mnt/usb/stash.db"
session_timeout_minutes = 60   # default: 15; set to 0 to disable timeout
clipboard_clear_seconds = 30   # clear clipboard N seconds after --copy; default: 0 (disabled)
browser = "firefox"            # preferred browser for `stash web`
```

### Alternate database

The database path is resolved in this order (first match wins):

| Source | Example |
|--------|---------|
| `--db` flag | `stash --db /tmp/test.db list` |
| `STASH_DB` env var | `export STASH_DB=/mnt/usb/stash.db` |
| `db` in `stash.toml` | `db = "/mnt/usb/stash.db"` |
| Default | `~/.local/share/stash/stash.db` |

## Data storage

| File | Purpose |
|------|---------|
| `~/.local/share/stash/stash.db` | SQLCipher-encrypted database |
| `~/.local/share/stash/stash.salt` | Argon2id salt for key derivation (mode 0600) |
| `~/.local/share/stash/.session` | Cached session key (mode 0600) |

On macOS the base directory is `~/Library/Application Support/stash/`.

## Quick start

```sh
# First run: creates a new vault and prompts for a master password
stash auth login

# Login with a password from stdin (useful in scripts)
echo "mypassword" | stash auth login

# Login with a custom session timeout (overrides config for this session)
stash auth login --timeout 60   # expire after 60 minutes of inactivity
stash auth login --timeout 0    # never expire

# Add items: stash add <type> <name> [content]
stash add note todo  "Buy milk"
stash add url  gh    "https://github.com"
stash add url  work  "https://example.com" --browser firefox

# Add an item with a title
stash add url gh "https://github.com" --title "GitHub"

# Add items with tags
stash add note todo "Buy milk" --tag work --tag personal

# Compose a longer note in your editor
stash add note journal --edit

# Read from stdin
echo "some text" | stash add note pipe --stdin

# Show an item
stash show gh                   # prints title (if set) then content
stash show gh --verbose         # content + metadata (type, title, timestamps, tags)
stash show gh --copy            # copy content to clipboard (no terminal output)

# Edit an item (opens $EDITOR; old version is archived automatically)
stash edit todo

# Update just the title, without opening an editor
stash edit gh --title "GitHub Home"

# View version history (includes title per version)
stash history gh

# Open a URL in the browser
stash web gh
stash web gh --private          # incognito / private mode
stash web gh --browser firefox  # use a specific browser (overrides config)

# Store a permanent per-item private-mode preference
stash browser gh --private      # stash web gh will always open in private mode
stash browser gh --no-private   # clear that preference

# List all items (TITLE column shown when any item has a title)
stash list

# List items filtered by tag (OR logic — shows items with any matching tag)
stash list --tag work
stash list --tag work --tag personal

# List items filtered by type
stash list --type url

# Manage tags on existing items
stash tag todo urgent
stash untag todo personal

# List all tags in the vault with item counts
stash tags

# Search item content and titles (case-insensitive substring)
stash search "github"

# Search using a regular expression (case-sensitive)
stash search --regex '\d{4}-\d{2}-\d{2}'

# Case-insensitive regex (use (?i) inline flag)
stash search --regex '(?i)github'

# Also search archived versions (shown as name:vN)
stash search --include-history "old value"
stash search --regex --include-history '(?i)todo'

# Filter by tag (repeatable; OR logic — shows items with any matching tag)
stash search --tag work
stash search --tag work --tag personal

# Filter by type
stash search --type url

# Combine: pattern + tag + type
stash search --tag work --type url "meeting"

# Change or clear the stored browser preference for a URL item
stash browser gh firefox
stash browser gh --clear

# Delete an item and all its history
stash purge todo
stash purge todo --force        # skip confirmation (useful in scripts)

# Rename an item
stash rename todo todo-done

# Copy an item to a new shortname (copies content, title, type, and tags)
stash copy aws-key aws-key-backup

# Restore an item to its previous version (undo last edit; restores title too)
stash restore todo

# Restore to a specific version
stash restore todo --version 2

# End the session
stash auth logout

# Change the master password (re-keys the database in place)
stash auth reset

# Migrate an existing vault from the old field-level-encrypted format
stash migrate

# Export the entire vault to a JSON file
stash export -o vault.json

# Export including full version history for each item
stash export --include-history -o vault-with-history.json

# Import from a JSON export (skips items that already exist)
stash import vault.json

# Import and overwrite items that already exist
stash import --overwrite vault.json

# Pipe between vaults (e.g. copy to a different database)
stash export --include-history | stash import --db /path/to/other.db
```

## Command reference

### `stash auth login`

```
stash auth login [--timeout <MINUTES>]
```

Authenticates against the vault. On first run, creates a new vault and prompts
you to set a master password (minimum 12 characters). On subsequent runs, prompts
for the existing password.

If stdin is not a terminal (i.e., data is piped in), the first line is read as
the password and the confirmation prompt is skipped. This enables non-interactive
use in scripts:

```sh
echo "mypassword" | stash auth login
```

The derived key is cached for up to `session_timeout_minutes` minutes (default
15; 0 = no timeout). The timeout is a **sliding window** — every stash command
resets the clock, so the session only expires after that many minutes of
inactivity. Pass `--timeout` to override the config value for this login only:

```sh
stash auth login --timeout 60   # expire after 60 minutes of inactivity
stash auth login --timeout 0    # never expire
```

Timeout resolution order: `--timeout` flag > `session_timeout_minutes` in `stash.toml` > default (15 minutes).

On macOS the key is stored in **Keychain**; on Linux it is stored
in the **Secret Service** (GNOME Keyring / KWallet) if `secret-tool` is
available. In both cases the session file (`~/.local/share/stash/.session`,
mode 0600) is kept as a fallback. The login message notes `(keychain)` when the
system keychain was used.

### `stash auth logout`

Removes the cached session key. The vault database is not affected.

### `stash auth reset`

Changes the master password. Prompts for the current password (to verify it),
then prompts for the new password (minimum 12 characters, with confirmation).
The database is re-keyed in place via SQLCipher's `PRAGMA rekey` — a single
atomic operation that re-encrypts every page of the database file.
The current session is cleared on success; run `stash auth login` afterward.

### `stash add`

```
stash add <url|note> <name> [TEXT] [options]
```

| Argument / Option | Description |
|-------------------|-------------|
| `url\|note` | Item type (required, positional) |
| `<name>` | Short identifier used in all other commands (required, positional) |
| `TEXT` | Inline content (optional positional) |
| `-t`, `--title <TITLE>` | Human-readable title |
| `-e`, `--edit` | Open `$EDITOR` to compose content |
| `--stdin` | Read content from standard input |
| `-g`, `--tag <TAG>` | Attach a tag (repeatable: `--tag work --tag personal`) |
| `-b`, `--browser <BROWSER>` | Store a preferred browser for this URL item (url items only) |

Exactly one of `TEXT`, `--edit`, or `--stdin` must be supplied.

### `stash show <shortname>`

Prints the item's content. If the item has a title it is printed as a header
line above the content. If the item has tags they are shown on a `tags:` line
after the content. Add `--verbose` / `-v` to show all metadata (type, title,
browser preference, private flag, timestamps, and tags) in a header block.

Add `--copy` / `-c` to copy the content to the clipboard instead of printing it
(requires `pbcopy` on macOS, `wl-copy` on Wayland, `xclip` or `xsel` on X11,
or `clip.exe` on Windows/WSL).

Pass `--clear-after <SECONDS>` to automatically clear the clipboard after the
given number of seconds. This overrides `clipboard_clear_seconds` in `stash.toml`
for a single invocation:

```sh
stash show mypassword --copy                  # copy; no automatic clear
stash show mypassword --copy --clear-after 30 # copy; clear after 30 s
```

Clear timeout resolution order: `--clear-after` flag > `clipboard_clear_seconds`
in `stash.toml` > 0 (disabled).

### `stash edit <shortname>`

```
stash edit <shortname> [-t/--title <TITLE>]
```

Without `--title`, opens the item in `$EDITOR`. If the content changes, the
previous version is automatically saved to history before the update is written.

With `--title <TITLE>`, updates the title only without opening an editor. The
current content and old title are archived to history as part of the update.

### `stash history <shortname>`

Shows all archived versions followed by the current content, each labelled with
its version number and timestamp. If a version had a title set, it is shown
alongside the content for that version.

### `stash web [-p] [-b <browser>] <shortname>`

Opens a `url`-type item in the browser. Pass `-p` / `--private` to open in
private/incognito mode. Pass `-b` / `--browser` to specify a browser binary
(e.g., `firefox`, `google-chrome`); this overrides the `browser` field in
`stash.toml`. Without a specified browser, the system default is used (or, for
`--private`, tries Firefox, Chrome, Chromium, Brave, and Vivaldi in order).

If the item has a stored private-mode preference (set via `stash browser
--private`), private mode is activated automatically even without `-p`.

Browser resolution order: `--browser` flag > per-item stored browser (`stash browser`) > `browser` in `stash.toml` > system default.

Private mode resolution order: `-p` flag > per-item stored private preference > off.

Private-mode flags are known for: `firefox` (`--private-window`),
`google-chrome` / `chrome`, `chromium`, `chromium-browser`, `brave-browser`, `vivaldi`, `vivaldi-stable` (`--incognito`).
`chrome` is accepted as an alias for `google-chrome`.

### `stash browser <shortname> [<browser>] [--clear] [--private | --no-private]`

Sets or clears browser preferences stored with a `url`-type item.

| Option | Description |
|--------|-------------|
| `<browser>` | Browser binary to store (e.g. `firefox`) |
| `--clear` | Remove the stored browser preference |
| `--private` | Always open this URL in private/incognito mode |
| `--no-private` | Clear the stored private-mode preference |

Options can be combined: `stash browser gh firefox --private` sets both at once.
`--private` and `--no-private` are mutually exclusive.

```sh
stash browser gh firefox          # set stored browser
stash browser gh --clear          # remove stored browser preference
stash browser gh --private        # always open in private mode
stash browser gh --no-private     # clear private-mode preference
stash browser gh firefox --private  # set browser and private mode together
```

### `stash list`

```
stash list [-g/--tag <TAG>]... [-t/--type <TYPE>]
```

Lists all items in a table showing name, type, and tags. When at least one item
has a title, a TITLE column is added to the table. Pass one or more
`-g`/`--tag` options to show only items that have **any** of the specified tags
(OR logic). Pass `-t`/`--type` to restrict to a single type (`url` or `note`). Both
filters can be combined.

### `stash tag <shortname> <TAG>...`

Adds one or more tags to an existing item. Duplicate tags are silently ignored.

### `stash untag <shortname> <TAG>...`

Removes one or more tags from an existing item. Tags not present on the item are
silently ignored.

### `stash tags`

Lists every tag in the vault, sorted alphabetically, with a count of how many
items carry each tag:

```
TAG        ITEMS
──────────────────
personal   3
work       5

2 tag(s).
```

### `stash search`

```
stash search [PATTERN] [--regex] [--include-history] [--tag <TAG>]... [--type <TYPE>]
```

Searches items by content, title, tag, and/or type. At least one of `PATTERN`,
`--tag`, or `--type` is required.

| Option | Description |
|--------|-------------|
| `PATTERN` | Text or regex to match against content and titles |
| `-r`, `--regex` | Treat `PATTERN` as a regular expression |
| `-H`, `--include-history` | Also search archived versions (shown as `name:vN`) |
| `-g`, `--tag <TAG>` | Restrict to items with this tag (repeatable; OR logic) |
| `-t`, `--type <TYPE>` | Restrict results to `url` or `note` |

Without `--regex`, matching is a case-insensitive substring search. With
`--regex`, the pattern is case-sensitive; prefix with `(?i)` for
case-insensitive regex.

```sh
stash search "api key"
stash search --regex '\d{4}-\d{2}-\d{2}'
stash search --regex '(?i)secret'
stash search --tag work
stash search --tag work --tag personal
stash search --type url "github"
stash search --include-history "old password"
stash search --regex --include-history '(?i)todo'
```

### `stash purge <shortname>`

Deletes an item and its entire history after a confirmation prompt. Pass
`--force` / `-f` to skip the prompt (useful in scripts).

### `stash rename <shortname> <new-name>`

Renames an item. Fails if `<new-name>` is already in use.

### `stash copy <shortname> <dest>`

Copies an item (content, title, type, and tags) to a new shortname. History is
not copied.

### `stash migrate`

Converts a vault from the old field-level-encrypted plain SQLite format to
the current whole-database SQLCipher format. Run this once after upgrading from
v1.0.x.

The same master password is used — no password change is required. The migration
writes a new encrypted database to a sibling file first, then renames it into
place atomically. The original file is not modified until the rename succeeds,
so a crash mid-migration leaves the old vault intact.

After migration, run `stash auth login` to start a new session.

### `stash export`

```
stash export [-o/--output <FILE>] [--include-history]
```

Exports all vault items to JSON. Output goes to stdout by default so it can be
piped or redirected; use `-o`/`--output` to write directly to a file.

| Option | Description |
|--------|-------------|
| `-o`, `--output <FILE>` | Write to this file instead of stdout |
| `--include-history` | Include full version history for each item |

The JSON format is versioned (`"version": 1`) and includes the shortname, type,
content, title, tags, browser preference, private flag, and timestamps for every
item. History entries include their title at the time of archiving.

### `stash import`

```
stash import [FILE] [--overwrite]
```

Imports items from a JSON export file. Reads from stdin if `FILE` is omitted,
making it composable with `stash export` via a pipe.

| Option | Description |
|--------|-------------|
| `FILE` | Path to the export file (omit to read from stdin) |
| `--overwrite` | Replace existing items instead of skipping them |

By default, items whose shortname already exists in the vault are skipped and
counted in the summary. Pass `--overwrite` to delete the existing item (and its
history) and replace it with the imported version. History entries present in the
export file are restored in either case.

Prints a summary on completion: items imported, items skipped, items that failed
(e.g. unknown type).

### `stash restore <shortname>`

```
stash restore <shortname> [--version <N>]
```

Restores an item to a previous version. Without `--version`, restores to the
most recently archived version (undo last edit). Both content and title are
restored from the history entry. The current content and title are archived
before the restore, so the full history is preserved.

### `stash completions <SHELL>`

Prints a shell completion script to stdout. Supported shells: `bash`, `zsh`,
`fish`, `powershell`, `elvish`.

Source the output in your shell's startup file to get tab-completion for all
subcommands, flags, and arguments:

```sh
# bash — add to ~/.bashrc
source <(stash completions bash)

# zsh — add to ~/.zshrc
source <(stash completions zsh)

# fish — save to the completions directory
stash completions fish > ~/.config/fish/completions/stash.fish
```

## Excluding stash from shell history

When you pass content inline (`stash add note pw "s3cr3t"`), that content lands
in your shell history. The cleanest remedies are to use `--edit` or `--stdin`
instead; if you still want inline content, you can suppress history recording
at the shell level.

### bash

Add to `~/.bashrc`:

```bash
# Ignore any command that starts with "stash"
HISTIGNORE="stash *:stash:$HISTIGNORE"
```

Alternatively, enable space-prefix suppression and lead every sensitive stash
command with a space:

```bash
HISTCONTROL=ignorespace   # or ignoreboth (combines with ignoredups)
#  ↓ leading space — not recorded
 stash add note pw "s3cr3t"
```

### zsh

Add to `~/.zshrc`:

```zsh
# Option 1: ignore stash via a hook (works in all zsh versions)
zshaddhistory() { [[ $1 != stash\ * && $1 != stash$'\n' ]] }

# Option 2: space-prefix suppression (simpler, same caveat as bash)
setopt HIST_IGNORE_SPACE
#  ↓ leading space — not recorded
 stash add note pw "s3cr3t"
```

### fish

Fish has no pattern-based ignore list. Use `--stdin` or `--edit` to keep
secrets out of the command line entirely. If you do pass inline content and
want to remove the entry afterward:

```fish
history delete -- "stash add note pw s3cr3t"
```

## Security notes

- The **entire database file** is encrypted with SQLCipher (AES-256-CBC with
  HMAC-SHA512 page authentication). Shortnames, types, titles, tags, content, and
  timestamps are all opaque to anyone without the key.
- The **salt file** (`stash.salt`) is stored in plaintext alongside the database.
  It contains the Argon2id salt used to derive the encryption key from your
  master password. Losing it means losing access to the vault.
- The **session file** stores the raw 32-byte derived key in base64. It is
  written with mode 0600 and removed on `stash auth logout`.
- `$EDITOR` opens items in a temporary file. Some editors create swap or backup
  files in the same directory.

## License

MIT — see [LICENSE](../LICENSE).
