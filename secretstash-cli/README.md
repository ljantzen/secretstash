# stash

[![CI](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A command-line manager for notes and URLs — stored in an encrypted local database.

Each item is identified by a short name and versioned: every edit is archived so you can review the full history at any time.

**📖 Full documentation, configuration reference, and command reference: [the user manual](https://ljantzen.github.io/secretstash/)** (source in [`book/`](../book/)).

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

The cryptographic primitives used here are believed sound but have not undergone a security review. See [Security Notes](https://ljantzen.github.io/secretstash/security.html) in the manual before storing anything that needs to withstand a serious, determined attacker.

## Installation

```sh
cargo install --path secretstash-cli
```

Pre-built binaries for Linux, macOS, and Windows are available on the [releases page](https://github.com/ljantzen/secretstash/releases). See [Installation](https://ljantzen.github.io/secretstash/installation.html) in the manual for platform-specific archive names and shell completions.

## Getting started

```sh
stash auth login              # first run creates a vault and sets a master password
stash add note todo "Buy milk"
stash list
```

Everything else — configuration (`stash.toml`, alternate database locations), the full command reference, shell-history hygiene, and security notes — is covered in the **[user manual](https://ljantzen.github.io/secretstash/)**.

## License

MIT — see [LICENSE](../LICENSE).
