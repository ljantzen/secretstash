# stash

[![CI](https://github.com/ljantzen/stash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/stash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A command-line manager for notes, URLs, and secrets — stored in an encrypted local database.

Each item is identified by a short name and versioned: every edit is archived so you can review the full history at any time.

## Features

- **Three item types** — `note`, `url`, `secret`
- **Encrypted at rest** — ChaCha20-Poly1305 field-level encryption; Argon2id key derivation
- **Version history** — every edit is archived; nothing is silently overwritten
- **Editor integration** — `$EDITOR` opens for composing or editing any item
- **Browser integration** — open URL items directly, with optional private/incognito mode
- **Self-contained** — bundles SQLite; no external database or service required

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
stash add --type note   --shortname todo        "Buy milk"
stash add --type url    --shortname gh          "https://github.com"
stash add --type secret --shortname aws-key     "AKIA..."

# Compose a longer note in your editor
stash add --type note --shortname journal --edit

# Read from stdin
echo "some text" | stash add --type note --shortname pipe --stdin

# Show an item
stash show todo                 # content only
stash show todo --verbose       # content + metadata (type, timestamps)

# Edit an item (opens $EDITOR; old version is archived automatically)
stash edit todo

# View version history
stash history todo

# Open a URL in the browser
stash web gh
stash web gh --private          # incognito / private mode

# Delete an item and all its history
stash purge todo

# End the session
stash auth logout
```

## Command reference

### `stash auth login`

Authenticates against the vault. On first run, creates a new vault and prompts
you to set a master password. On subsequent runs, prompts for the existing password.
The derived key is cached in `.session` (permissions 0600) until you log out.

### `stash auth logout`

Removes the cached session key. The vault database is not affected.

### `stash add`

```
stash add -t/--type <url|note|secret> -s/--shortname <name> [options] [TEXT]
```

| Option | Description |
|--------|-------------|
| `-t`, `--type` | Item type: `url`, `note`, or `secret` (required) |
| `-s`, `--shortname` | Identifier used in all other commands (required) |
| `-e`, `--edit` | Open `$EDITOR` to compose content (pre-populated with TEXT if given) |
| `--stdin` | Read content from standard input |
| `TEXT` | Inline content as a positional argument |

Exactly one of `--edit`, `--stdin`, or positional `TEXT` must be supplied.

### `stash show <shortname>`

Prints the item's content. Add `--verbose` / `-v` to also show the type and timestamps.

### `stash edit <shortname>`

Opens the item in `$EDITOR`. If the content changes, the previous version is
automatically saved to history before the update is written.

### `stash history <shortname>`

Shows all archived versions followed by the current content, each labelled with
its version number and timestamp.

### `stash web [-p] <shortname>`

Opens a `url`-type item in the default browser. Pass `-p` / `--private` to open
in private/incognito mode (tries Firefox, Chrome, Chromium, and Brave in order).

### `stash purge <shortname>`

Deletes an item and its entire history after a confirmation prompt.

## Security notes

- The database file is **not** encrypted as a whole; only the content fields are.
  Metadata (shortnames, types, timestamps) is stored in plaintext.
- The session file stores the raw 32-byte key in base64. It is written with
  mode 0600 and lives only as long as you stay logged in.
- `$EDITOR` opens items in a temporary file. Some editors create swap or backup
  files in the same directory; be aware of this when editing secrets.

## License

MIT — see [LICENSE](LICENSE).
