# Rust project tasks

# Default task - show available recipes
default:
    @just --list

# Build tasks
build:
    cargo build

build-release:
    cargo build --release

# Install tasks
install:
    cargo install --path stashvault-cli --force

uninstall:
    cargo uninstall stash

# Test tasks
test:
    cargo test

test-lib:
    cargo test -p stashvault

test-cli:
    cargo test -p stashvault-cli

test-verbose:
    cargo test -- --nocapture

test-all:
    cargo test --all-targets

# Code quality checks
check:
    cargo check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

clippy-fix:
    cargo clippy --fix --allow-dirty --all-targets --all-features

lint: fmt-check clippy
    @echo "Linting passed"

# Documentation
doc:
    cargo doc -p stashvault --no-deps --open

doc-check:
    cargo doc --no-deps 2>&1 | grep -E "warning|error" || true

# Development
run *ARGS:
    cargo run -p stashvault-cli -- {{ARGS}}

watch:
    cargo watch -x build -x test

check-watch:
    cargo watch -x check

# Cleaning
clean:
    cargo clean

# Code coverage
coverage:
    cargo llvm-cov --open

# Version control (jj)
status:
    jj status

log:
    jj log

diff:
    jj diff

squash:
    jj squash

jj-restore:
    jj restore

rebase-main:
    jj rebase -d main

# Git mirror
git-sync:
    jj git push

git-fetch:
    jj git fetch

# Release tasks
release-check:
    @echo "Running full release checklist..."
    just lint
    just test-all
    just doc-check

# Format and test in sequence
fmt-test: fmt
    cargo test

# Quick dev loop
dev: check test
    @echo "Development checks passed"
