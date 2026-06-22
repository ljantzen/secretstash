# secretstash

[![CI](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml/badge.svg)](https://github.com/ljantzen/secretstash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Encrypted local storage for notes, URLs, and secrets.

This repository contains two crates:

| Crate | Description |
|-------|-------------|
| [`secretstash`](secretstash/) | Core library — encrypted vault, key derivation, session management |
| [`secretstash-cli`](secretstash-cli/) | `stash` command-line tool built on top of the library |

## What it does

- Stores notes and URLs in a **SQLCipher-encrypted** local database (AES-256-CBC, Argon2id key derivation)
- Every edit is **versioned** — history is preserved and restorable
- Items have **tags** and optional **titles**
- URL items can be opened in the browser, including private/incognito mode
- Export/import via a portable JSON format

## Getting started

See the **[CLI README](secretstash-cli/README.md)** for installation, configuration, and a full command reference.

See the **[library README](secretstash/README.md)** if you want to embed the encrypted vault in your own application.

## Repository layout

```
secretstash/        Core library (Db, crypto, session, config, clipboard, keychain)
secretstash-cli/    CLI binary (`stash`)
.github/workflows/  CI and release pipelines
```

## License

MIT — see [LICENSE](LICENSE).
