# secretstash

[![CI](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/secretstash.svg)](https://crates.io/crates/secretstash)

Core library for [secretstash](https://github.com/ljantzen/secretstash) — encrypted local storage for notes, URLs, and secrets.

The `stash` CLI is built on top of this crate. You can use it directly if you want to embed an encrypted vault in your own application.

## What it provides

- **`db`** — `Db` struct: open/create a SQLCipher-encrypted vault, insert/read/update/delete items, manage tags and version history
- **`crypto`** — Argon2id key derivation from a password + salt; ChaCha20-Poly1305 decrypt (used by the migration path)
- **`session`** — save/load/clear a derived key to the OS keychain or a local session file
- **`config`** — resolve config file, database, salt and session paths; read `stash.toml`
- **`clipboard`** — copy text to the system clipboard with optional auto-clear
- **`keychain`** — thin wrapper around the macOS Keychain / Linux Secret Service
- **`commands`** — implementations of all `stash` subcommands (used by the CLI; stable but not semver-committed beyond what the CLI requires)

## Adding it as a dependency

```toml
[dependencies]
secretstash = "1.1"
```

Requires Rust 1.85+ (edition 2024). The crate bundles SQLite via the `bundled-sqlcipher-vendored-openssl` feature of `rusqlite`, so no external database libraries are needed.

## Quick example

```rust
use secretstash::{crypto, db::Db};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Derive a 32-byte key from a password and a stored salt.
    let salt = crypto::generate_salt();
    let key = crypto::derive_key("my-master-password", &salt)?;

    // Open (or create) an encrypted vault.
    let db = Db::open(Path::new("/tmp/example.db"), &key)?;

    // Insert a note.
    db.insert_item("todo", "note", "Buy milk", None, None)?;

    // Read it back.
    let item = db.get_item("todo")?.unwrap();
    println!("{}: {}", item.shortname, item.content);

    Ok(())
}
```

## Key derivation

```rust
use secretstash::crypto;

// Generate a random 32-byte salt (store this alongside the database).
let salt_b64 = crypto::generate_salt();

// Derive a 32-byte key. This is intentionally slow (Argon2id, 64 MiB, 4 passes).
let key = crypto::derive_key("password", &salt_b64)?;
```

## Database operations

`Db::open` creates the file if it doesn't exist and runs schema migrations automatically.

```rust
use secretstash::db::Db;
use std::path::Path;

let db = Db::open(Path::new("vault.db"), &key)?;

// Insert items (type is "note" or "url").
let id = db.insert_item("gh", "url", "https://github.com", Some("GitHub"), None)?;

// Tags
db.add_tag(id, "work")?;
let tags = db.get_tags(id)?;

// Read
let item = db.get_item("gh")?.unwrap();

// Edit — archives the current content as a history entry before writing.
db.replace_content(id, "gh", &item.content, item.title.as_deref(), "https://github.com/new", Some("GitHub (new)"))?;

// History
let history = db.get_history(id)?;

// Rename / delete
db.rename_item("gh", "github")?;
db.delete_item("github")?;
```

On Unix, `Db::open` creates the database file with mode `0600` before SQLite touches it, so the vault is never world-readable even briefly.

## Session management

The session layer caches the derived key so users don't re-enter their password on every command.

```rust
use secretstash::session;

// Save the key (OS keychain if available, falls back to a 0600 file).
// timeout_minutes=0 means the session never expires.
session::save_key(&key, 15)?;

// Load it back (validates expiry; refreshes the sliding window).
let key = session::load_key()?;

// Inspect without refreshing.
let status = session::get_status()?;

// Remove.
session::clear_key()?;
```

## Development

```sh
git clone https://github.com/ljantzen/secretstash
cd secretstash

# Run the library tests
cargo test -p secretstash

# Run all tests
cargo test
```

The test suite includes in-memory database tests (`Db::open_in_memory`) that run without touching the filesystem, plus file-based tests for SQLCipher encryption, permission enforcement, and session parsing.

## License

MIT — see [LICENSE](../LICENSE).
