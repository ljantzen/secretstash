# auth

## `stash auth login`

```
stash auth login [--timeout <MINUTES>]
```

Authenticates against the vault. On first run, creates a new vault and prompts you to set a master password (minimum 12 characters). On subsequent runs, prompts for the existing password.

If stdin is not a terminal (i.e., data is piped in), the first line is read as the password and the confirmation prompt is skipped. This enables non-interactive use in scripts:

```sh
echo "mypassword" | stash auth login
```

The derived key is cached for up to `session_timeout_minutes` minutes (default 15; 0 = no timeout). The timeout is a **sliding window** — every stash command resets the clock, so the session only expires after that many minutes of inactivity. Pass `--timeout` to override the config value for this login only:

```sh
stash auth login --timeout 60   # expire after 60 minutes of inactivity
stash auth login --timeout 0    # never expire
```

Timeout resolution order: `--timeout` flag > `session_timeout_minutes` in `stash.toml` > default (15 minutes).

On macOS the key is stored in **Keychain**; on Linux it is stored in the **Secret Service** (GNOME Keyring / KWallet) if `secret-tool` is available. In both cases the session file (`~/.local/share/stash/.session`, mode 0600) is kept as a fallback. The login message notes `(keychain)` when the system keychain was used.

## `stash auth status`

Shows whether a session is currently active.

## `stash auth logout`

Removes the cached session key. The vault database is not affected.

## `stash auth reset`

Changes the master password. Prompts for the current password (to verify it), then prompts for the new password (minimum 12 characters, with confirmation). The database is re-keyed in place via SQLCipher's `PRAGMA rekey` — a single atomic operation that re-encrypts every page of the database file.

The current session is cleared on success; run `stash auth login` afterward.
