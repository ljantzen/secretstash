# Contributing

## Prerequisites

- Rust 1.85+ (`rustup update stable`)
- [`just`](https://github.com/casey/just) — task runner (`cargo install just`)
- [`jj`](https://github.com/jj-vcs/jj) — version control (optional; plain git works too)

Optional for coverage:

```sh
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

## Building

```sh
just build          # debug
just build-release  # release
just install        # install stash to ~/.cargo/bin
```

## Testing

```sh
just test           # all tests
just test-lib       # secretstash library only
just test-cli       # secretstash-cli only
just test-verbose   # show test output (--nocapture)
just test-all       # include integration / doc tests
```

The library test suite uses in-memory SQLite databases where possible, so most tests are fast and leave no files on disk. File-based tests (SQLCipher encryption, permission checks) run in `tempdir` and clean up after themselves.

## Code style

```sh
just fmt        # format everything
just clippy     # lint with -D warnings
just lint       # fmt-check + clippy together (what CI runs)
```

CI enforces `just lint` on every push. Please run it before submitting a change.

## Documentation

```sh
just doc        # build and open secretstash library docs
just doc-check  # check for doc warnings/errors
```

## Development loop

```sh
just dev        # cargo check + cargo test in sequence
just watch      # rebuild and re-test on every file change (requires cargo-watch)
```

## Version control

The repository uses [Jujutsu](https://github.com/jj-vcs/jj) with a git mirror on GitHub. You can contribute using plain git — the workflow is the same from GitHub's perspective.

With jj:

```sh
jj status       # working-copy status
jj log          # commit graph
jj diff         # current diff
jj squash       # fold working-copy changes into the parent commit
jj git push     # push to the git remote
```

## Releasing

Releases are driven by `release.sh`, which bumps the version, commits, creates an annotated git tag, and pushes it. GitHub Actions picks up the tag and builds binaries for all platforms.

```sh
# Bump the patch version automatically
./release.sh

# Specify a version explicitly
./release.sh 1.2.0

# Also publish to crates.io after the GitHub Actions build succeeds
./release.sh --publish 1.2.0
```

The release script requires a clean working directory and the GitHub CLI (`gh`).

## Project layout

```
secretstash/        Core library (Db, crypto, session, config, clipboard, keychain)
  src/
    db.rs           Encrypted SQLite (SQLCipher) — items, history, tags
    crypto.rs       Argon2id key derivation; ChaCha20-Poly1305 decrypt
    session.rs      Session key storage (OS keychain + fallback file)
    config.rs       Config file and path resolution
    clipboard.rs    System clipboard integration
    keychain.rs     macOS Keychain / Linux Secret Service wrapper
    commands/       One module per stash subcommand

secretstash-cli/    CLI binary
  src/
    cli.rs          Clap argument definitions
    main.rs         Entry point

.github/workflows/
  ci.yml            Lint + test on every push
  release.yml       Cross-platform build + GitHub Release on version tags
```
