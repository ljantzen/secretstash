# migrate

```
stash migrate
```

Converts a vault from the old field-level-encrypted plain SQLite format to the current whole-database SQLCipher format. Run this once after upgrading from v1.0.x.

The same master password is used — no password change is required. The migration writes a new encrypted database to a sibling file first, then renames it into place atomically. The original file is not modified until the rename succeeds, so a crash mid-migration leaves the old vault intact.

After migration, run `stash auth login` to start a new session.
