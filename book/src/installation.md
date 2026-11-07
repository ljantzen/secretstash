# Installation

## Pre-built binaries

Download the latest release for your platform from the [releases page](https://github.com/ljantzen/secretstash/releases):

| Platform | Archive |
|----------|---------|
| Linux x86_64 (glibc) | `secretstash-linux-x86_64.tar.gz` |
| Linux x86_64 (musl / static) | `secretstash-linux-x86_64-musl.tar.gz` |
| macOS aarch64 (Apple Silicon) | `secretstash-macos-aarch64.tar.gz` |
| Windows x86_64 | `secretstash-windows-x86_64.zip` |

Extract the archive and place the `stash` binary somewhere on your `$PATH`.

## From source

Requires Rust 1.85+.

```sh
cargo install --path secretstash-cli
```

## Shell completions

stash can generate completion scripts for bash, zsh, fish, PowerShell, and Elvish via [`stash completions`](./commands/completions.md):

```sh
# bash — add to ~/.bashrc
source <(stash completions bash)

# zsh — add to ~/.zshrc
source <(stash completions zsh)

# fish — save to the completions directory
stash completions fish > ~/.config/fish/completions/stash.fish
```
