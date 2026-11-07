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

**📖 [Read the user manual](https://ljantzen.github.io/secretstash/)** for installation, configuration, and the full `stash` command reference. Its source lives in [`book/`](book/); build it locally with `just book-build` or serve it live with `just book-serve`.

See the **[CLI README](secretstash-cli/README.md)** for a quick overview of the `stash` binary.

See the **[library README](secretstash/README.md)** if you want to embed the encrypted vault in your own application.

## Repository layout

```
secretstash/        Core library (Db, crypto, session, config, clipboard, keychain)
secretstash-cli/    CLI binary (`stash`)
book/               User manual source (mdBook)
.github/workflows/  CI and release pipelines
```

## License

MIT — see [LICENSE](LICENSE).
