# stash

[![CI](https://github.com/ljantzen/stash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/stash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A command-line manager for notes and URLs — stored in an encrypted local database.

Each item is identified by a short name and versioned: every edit is archived so you can review the full history at any time.

## Features

- **Two item types** — `note`, `url`
- **Encrypted at rest** — ChaCha20-Poly1305 field-level encryption; Argon2id key derivation
- **Tags** — attach multiple encrypted tags to any item; filter and search by tag
- **Version history** — every edit is archived; nothing is silently overwritten
- **Editor integration** — `$EDITOR` opens for composing or editing any item
- **Browser integration** — open URL items directly, with optional private/incognito mode; store a per-item preferred browser
- **Self-contained** — bundles SQLite; no external database or service required

## Disclaimer

Although I believe that the cryptographic primitives implemented in this program are sound, they have not undergone a
security review. The encryption provided will provide protection from snooping family members, and probably colleagues
(depending on where you work). But, if youir notes and urls need protection from serious and determined hackers maybe
you should store them elsewhere.  Consider yourself notified. 

## Installation

### Pre-built binaries

Download the latest release for your platform from the [releases page](https://github.com/ljantzen/stash/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 (glibc) | `stash-linux-x86_64.tar.gz` |
| Linux x86_64 (musl / static) | `stash-linux-x86_64-musl.tar.gz` |
| macOS x86_64 (Intel) | `stash-macos-x86_64.tar.gz` |
| macOS aarch64 (Apple Silicon) | `stash-macos-aarch64.tar.gz` |
| Windows x86_64 | `stash-windows-x86_64.zip` |

Extract and place the `stash` binary somewhere on your `$PATH`.

### From source

Requires Rust 1.75+.

```sh
cargo install --path .
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
| `~/.local/share/stash/stash.db` | Encrypted SQLite database |
| `~/.local/share/stash/.session` | Cached session key (mode 0600) |

On macOS the base directory is `~/Library/Application Support/stash/`.

## Quick start

```sh
# First run: creates a new vault and prompts for a master password
stash auth login

# Add items
stash add --type note  --shortname todo  "Buy milk"
stash add --type url   --shortname gh    "https://github.com"
stash add --type url   --shortname work  "https://example.com" --browser firefox

# Add items with tags
stash add --type note --shortname todo --tag work --tag personal "Buy milk"

# Compose a longer note in your editor
stash add --type note --shortname journal --edit

# Read from stdin
echo "some text" | stash add --type note --shortname pipe --stdin

# Show an item
stash show todo                 # content only (tags printed if present)
stash show todo --verbose       # content + metadata (type, timestamps, tags)
stash show todo --copy          # copy content to clipboard (no terminal output)

# Edit an item (opens $EDITOR; old version is archived automatically)
stash edit todo

# View version history
stash history todo

# Open a URL in the browser
stash web gh
stash web gh --private          # incognito / private mode
stash web gh --browser firefox  # use a specific browser (overrides config)

# List all items
stash list

# List items filtered by tag (OR logic — shows items with any matching tag)
stash list --tag work
stash list --tag work --tag personal

# List items filtered by type
stash list --type url

# Manage tags on existing items
stash tag todo urgent
stash untag todo personal

# Search across all item content
stash find "search term"

# Search within a specific tag
stash find --tag work "meeting"

# List all items with a given tag (no content search)
stash find --tag work

# Filter by type (combinable with --tag and query)
stash find --type url
stash find --type url "github"

# Change or clear the stored browser preference for a URL item
stash browser gh firefox
stash browser gh --clear

# Delete an item and all its history
stash purge todo
stash purge todo --force        # skip confirmation (useful in scripts)

# Rename an item
stash rename todo todo-done

# Copy an item to a new shortname
stash copy aws-key aws-key-backup

# Restore an item to its previous version (undo last edit)
stash restore todo

# Restore to a specific version
stash restore todo --version 2

# End the session
stash auth logout

# Change the master password (re-encrypts the entire vault)
stash auth reset
```

## Command reference

### `stash auth login`

Authenticates against the vault. On first run, creates a new vault and prompts
you to set a master password (minimum 12 characters). On subsequent runs, prompts
for the existing password.

The derived key is cached for up to `session_timeout_minutes` minutes (default
15; 0 = no timeout). The timeout is a **sliding window** — every stash command
resets the clock, so the session only expires after that many minutes of
inactivity. On macOS the key is stored in **Keychain**; on Linux it is stored
in the **Secret Service** (GNOME Keyring / KWallet) if `secret-tool` is
available. In both cases the session file (`~/.local/share/stash/.session`,
mode 0600) is kept as a fallback. The login message notes `(keychain)` when the
system keychain was used.

### `stash auth logout`

Removes the cached session key. The vault database is not affected.

### `stash auth reset`

Changes the master password. Prompts for the current password (to verify it),
then prompts for the new password (minimum 12 characters, with confirmation).
All item content, version history, and tags are re-encrypted under the new key
in a single atomic transaction — the vault is never left in a partial state.
The current session is cleared on success; run `stash auth login` afterward.

### `stash add`

```
stash add -t/--type <url|note> -s/--shortname <name> [options] [TEXT]
```

| Option | Description |
|--------|-------------|
| `-t`, `--type` | Item type: `url` or `note` (required) |
| `-s`, `--shortname` | Identifier used in all other commands (required) |
| `-e`, `--edit` | Open `$EDITOR` to compose content |
| `--stdin` | Read content from standard input |
| `-g`, `--tag <TAG>` | Attach a tag (repeatable: `--tag work --tag personal`) |
| `-b`, `--browser <BROWSER>` | Store a preferred browser for this URL item (url items only) |
| `TEXT` | Inline content as a positional argument |

Exactly one of `--edit`, `--stdin`, or positional `TEXT` must be supplied.

### `stash show <shortname>`

Prints the item's content. If the item has tags they are shown on a `tags:` line
after the content. Add `--verbose` / `-v` to also show the type, timestamps, and
tags in a metadata header. Add `--copy` / `-c` to copy the content to the
clipboard instead of printing it (requires `pbcopy` on macOS, `wl-copy` on
Wayland, `xclip` or `xsel` on X11, or `clip.exe` on Windows/WSL).

### `stash edit <shortname>`

Opens the item in `$EDITOR`. If the content changes, the previous version is
automatically saved to history before the update is written.

### `stash history <shortname>`

Shows all archived versions followed by the current content, each labelled with
its version number and timestamp.

### `stash web [-p] [-b <browser>] <shortname>`

Opens a `url`-type item in the browser. Pass `-p` / `--private` to open in
private/incognito mode. Pass `-b` / `--browser` to specify a browser binary
(e.g., `firefox`, `google-chrome`); this overrides the `browser` field in
`stash.toml`. Without a specified browser, the system default is used (or, for
`--private`, tries Firefox, Chrome, Chromium, Brave, and Vivaldi in order).

Browser resolution order: `--browser` flag > per-item stored browser (`stash browser`) > `browser` in `stash.toml` > system default.

Private-mode flags are known for: `firefox` (`--private-window`),
`google-chrome` / `chrome`, `chromium`, `chromium-browser`, `brave-browser`, `vivaldi`, `vivaldi-stable` (`--incognito`).
`chrome` is accepted as an alias for `google-chrome`.

### `stash browser <shortname> [<browser> | --clear]`

Sets or clears the preferred browser stored with a `url`-type item. This
preference is used by `stash web` when no `--browser` flag is given.

```sh
stash browser gh firefox      # set stored browser
stash browser gh --clear      # remove stored browser preference
```

### `stash list`

```
stash list [-g/--tag <TAG>]... [-t/--type <TYPE>]
```

Lists all items in a table showing name, type, and tags. Pass one or more
`-g`/`--tag` options to show only items that have **any** of the specified tags
(OR logic). Pass `-t`/`--type` to restrict to a single type (`url` or `note`). Both
filters can be combined.

### `stash tag <shortname> <TAG>...`

Adds one or more tags to an existing item. Duplicate tags are silently ignored.

### `stash untag <shortname> <TAG>...`

Removes one or more tags from an existing item. Tags not present on the item are
silently ignored.

### `stash find`

```
stash find [--tag <TAG>] [--type <TYPE>] [QUERY]
```

Searches items by content, tag, and/or type (case-insensitive). At least one of
`QUERY`, `--tag`, or `--type` is required.

| Option | Description |
|--------|-------------|
| `QUERY` | Text to search for in item content |
| `-g`, `--tag <TAG>` | Restrict results to items that have this tag |
| `-t`, `--type <TYPE>` | Restrict results to `url` or `note` |

Results show the item name, type, and a snippet of matching content.

### `stash purge <shortname>`

Deletes an item and its entire history after a confirmation prompt. Pass
`--force` / `-f` to skip the prompt (useful in scripts).

### `stash rename <shortname> <new-name>`

Renames an item. Fails if `<new-name>` is already in use.

### `stash copy <shortname> <dest>`

Copies an item (content, type, and tags) to a new shortname. History is not
copied. Each field is re-encrypted with a fresh nonce.

### `stash restore <shortname>`

```
stash restore <shortname> [--version <N>]
```

Restores an item to a previous version. Without `--version`, restores to the
most recently archived version (undo last edit). The current content is
archived before the restore, so the full history is preserved.

## Security notes

- The database file is **not** encrypted as a whole; only the content fields are.
  Metadata (shortnames, types, timestamps) is stored in plaintext.
- **Tags are encrypted** — each tag is encrypted individually with its own random
  nonce, so tag values are never stored in plaintext.
- The session file stores the raw 32-byte key in base64. It is written with
  mode 0600 and lives only as long as you stay logged in.
- `$EDITOR` opens items in a temporary file. Some editors create swap or backup
  files in the same directory.

## License

MIT — see [LICENSE](LICENSE).
