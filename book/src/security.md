# Security Notes

- The **entire database file** is encrypted with SQLCipher (AES-256-CBC with HMAC-SHA512 page authentication). Shortnames, types, titles, tags, content, and timestamps are all opaque to anyone without the key.
- The **salt file** (`stash.salt`) is stored in plaintext alongside the database. It contains the Argon2id salt used to derive the encryption key from your master password. Losing it means losing access to the vault.
- The **session file** stores the raw 32-byte derived key in base64. It is written with mode 0600 and removed on `stash auth logout`.
- `$EDITOR` opens items in a temporary file. Some editors create swap or backup files in the same directory.

See the [disclaimer in the Introduction](./introduction.md#disclaimer) for the overall threat model stash is — and isn't — designed for.
