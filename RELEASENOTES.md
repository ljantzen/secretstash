# Release Notes

## Unreleased (2.0.0)

**Security**
- Reject non-http(s) content in `stash web`, preventing browser-flag injection via item content.
- Zeroize master passwords in memory after use.
- Write `stash export` output with `0600` permissions.
- Only write the session file as a keychain fallback, instead of always.
- Use a tmpfs-backed directory for the editor scratch file when available, so plaintext secrets don't linger on disk.
- Don't misreport real I/O errors as "wrong password" in `Db::open`.

**Added**
- `-p`/`--private` and `-n`/`--no-private` flags on `stash add url`.
- Browser name prefix matching (e.g. `-b fire` resolves to `firefox`) and a `[browser_flags]` section in `stash.toml` for custom browsers' private-mode flags.
- `stash tags` command; `search --tag` is now repeatable for multi-tag search.
- `private` key in `stash.toml`: a default privacy mode for URL items with no per-item preference, mirroring the existing `browser` default.

**Changed**
- `stash browser` command syntax reworked (see README).
- Updated to the ChaCha20-Poly1305 0.11 API.
- Fixed all pedantic clippy warnings.
- Routine dependency bumps (anyhow, chacha20poly1305, open, clap_complete, actions/checkout).

## v1.1.1 — 2026-06-22

- Added crates.io metadata (description, license, repository) to both `Cargo.toml` files.
- Revised documentation.
- Fixed `release.sh`.

## v1.1.0 — 2026-06-21

**Added**
- `stash search` command.
- Per-item title field and per-item "always private" flag.
- `stash auth status` command.
- `export`/`import` commands and a `--timeout` flag for `stash auth login`.
- Shell completions via `clap_complete`.
- Clipboard-clear timeout after `--copy`.
- Support for reading the master password from stdin (non-TTY login).
- Documented how to exclude `stash` from shell history (bash/zsh/fish).

**Changed**
- Renamed crates: `stashvault`/`stashvault-cli` → `secretstash`/`secretstash-cli`; split into a Cargo workspace.
- Switched from field-level encryption to full-database encryption via SQLCipher.
- `stash add` syntax changed: item type and name are now positional arguments.
- Added an encryption disclaimer to the README.

**Dependencies**
- Bumped `toml` 0.8.23 → 1.1.2, `zeroize` 1.8.2 → 1.9.0.

## v1.0.1 — 2026-06-08

- Added `stash auth reset` to re-encrypt the vault under a new master password.
- Removed macOS x86 target from the release GitHub Action.
- Release script fixes.

## v1.0.0 — 2026-06-07

- Initial release.
- Core functionality: add/show/list items, `stash.toml` config file, `--db` flag and `STASH_DB` environment variable, configurable session timeout (default 15 minutes).
