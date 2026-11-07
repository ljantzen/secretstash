# Configuration

## Config file

stash reads `~/.config/stash/stash.toml` on startup (macOS: `~/Library/Application Support/stash/stash.toml`). Create the directory and file if they do not exist.

```toml
# ~/.config/stash/stash.toml
db = "/mnt/usb/stash.db"
session_timeout_minutes = 60   # default: 15; set to 0 to disable timeout
clipboard_clear_seconds = 30   # clear clipboard N seconds after --copy; default: 0 (disabled)
browser = "firefox"            # preferred browser for `stash web`
private = true                 # default privacy mode for URL items with no per-item preference

# Private-mode flags for browsers not built in to stash.
# Key = browser binary name, value = the flag that opens a private window.
[browser_flags]
opera          = "--private"
my-browser     = "--incognito"
```

## Alternate database location

The database path is resolved in this order (first match wins):

| Source | Example |
|--------|---------|
| `--db` flag | `stash --db /tmp/test.db list` |
| `STASH_DB` env var | `export STASH_DB=/mnt/usb/stash.db` |
| `db` in `stash.toml` | `db = "/mnt/usb/stash.db"` |
| Default | `~/.local/share/stash/stash.db` |

## Data storage

| File | Purpose |
|------|---------|
| `~/.local/share/stash/stash.db` | SQLCipher-encrypted database |
| `~/.local/share/stash/stash.salt` | Argon2id salt for key derivation (mode 0600) |
| `~/.local/share/stash/.session` | Cached session key (mode 0600) |

On macOS the base directory is `~/Library/Application Support/stash/`.
