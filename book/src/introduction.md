# Introduction

**stash** is a command-line manager for notes and URLs, stored in an encrypted local database. Each item is identified by a short name and versioned: every edit is archived so you can review the full history at any time.

## Features

- **Two item types** — `note`, `url`
- **Encrypted at rest** — whole-database SQLCipher encryption (AES-256-CBC); Argon2id key derivation
- **Titles** — an optional human-readable title on every item, tracked through version history
- **Tags** — attach multiple tags to any item; filter and search by tag
- **Version history** — every edit is archived; nothing is silently overwritten
- **Editor integration** — `$EDITOR` opens for composing or editing any item
- **Browser integration** — open URL items directly, with optional private/incognito mode; store a per-item preferred browser and a per-item always-private preference
- **Export / import** — dump the vault to a portable JSON file and restore it to any vault
- **Self-contained** — bundles SQLite; no external database or service required

## Disclaimer

Although the cryptographic primitives implemented in this program are believed to be sound, they have not undergone a security review. The encryption provided will protect against snooping family members, and probably colleagues (depending on where you work). But if your notes and URLs need protection from serious and determined attackers, you may want to store them elsewhere. See [Security Notes](./security.md) before relying on stash for anything sensitive.

## How this manual is organized

- [Installation](./installation.md) and [Configuration](./configuration.md) get you set up.
- [Quick Start](./quick-start.md) is a tour of the most common commands.
- [Command Reference](./commands/index.md) documents every subcommand and flag in detail.
- [Keeping Secrets Out of Shell History](./shell-history.md) and [Security Notes](./security.md) cover operational and cryptographic details worth understanding before you store anything sensitive.
