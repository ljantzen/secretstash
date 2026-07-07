use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use rusqlite::{Connection, params};
use zeroize::Zeroizing;

use crate::{config, crypto};

/// Migrate a vault from the old field-level-encrypted plain `SQLite` format to
/// the new whole-database `SQLCipher` format.
///
/// Uses the same master password and Argon2id key as the old vault, so the
/// existing session (if any) continues to work after migration.
///
/// The migrated database is written to a sibling `.db.new` file first, then
/// atomically renamed into place. The salt file is written before the rename
/// so that a crash between the two leaves the vault in a recoverable state
/// (old plain DB still intact, salt file points to the same key).
pub fn migrate(db_path: &std::path::Path) -> Result<()> {
    let salt_path = config::salt_path_for_db(db_path);

    if salt_path.exists() {
        return Err(anyhow!(
            "Vault at '{}' is already in SQLCipher format (salt file exists). \
             Nothing to do.",
            db_path.display()
        ));
    }

    if !db_path.exists() {
        return Err(anyhow!(
            "No vault found at '{}'. Run 'stash auth login' to create one.",
            db_path.display()
        ));
    }

    let password = rpassword::prompt_password("Master password: ")?;
    migrate_with_password(db_path, &password)
}

/// Inner migration logic, separated so tests can supply the password directly
/// without needing a TTY.
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub(crate) fn migrate_with_password(db_path: &std::path::Path, password: &str) -> Result<()> {
    let salt_path = config::salt_path_for_db(db_path);

    if salt_path.exists() {
        return Err(anyhow!(
            "Vault at '{}' is already in SQLCipher format (salt file exists). \
             Nothing to do.",
            db_path.display()
        ));
    }

    if !db_path.exists() {
        return Err(anyhow!(
            "No vault found at '{}'. Run 'stash auth login' to create one.",
            db_path.display()
        ));
    }

    // Open old plain-SQLite vault.
    let old = Connection::open(db_path)?;
    old.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Read Argon2id salt from old meta table.
    let old_salt: String = old
        .query_row("SELECT value FROM meta WHERE key = 'salt'", [], |row| {
            row.get(0)
        })
        .map_err(|_| {
            anyhow!(
                "Could not read vault metadata. \
                 The file may already be encrypted or corrupted."
            )
        })?;

    let key = crypto::derive_key(password, &old_salt)?;

    let canary_enc = B64.decode(
        old.query_row(
            "SELECT value FROM meta WHERE key = 'canary_enc'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| anyhow!("Vault metadata corrupted (missing canary_enc)"))?,
    )?;
    let canary_nonce = B64.decode(
        old.query_row(
            "SELECT value FROM meta WHERE key = 'canary_nonce'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| anyhow!("Vault metadata corrupted (missing canary_nonce)"))?,
    )?;
    crypto::decrypt(&key, &canary_enc, &canary_nonce).map_err(|_| anyhow!("Incorrect password"))?;

    println!("Password verified. Migrating vault…");

    // Read all old data before touching the new DB.
    struct OldItem {
        id: i64,
        shortname: String,
        item_type: String,
        content_enc: Vec<u8>,
        nonce: Vec<u8>,
        created_at: String,
        updated_at: String,
        browser: Option<String>,
    }
    struct OldHistory {
        item_id: i64,
        content_enc: Vec<u8>,
        nonce: Vec<u8>,
        version: i64,
        created_at: String,
    }
    struct OldTag {
        item_id: i64,
        tag_enc: Vec<u8>,
        nonce: Vec<u8>,
    }

    let old_items: Vec<OldItem> = {
        let mut s = old.prepare(
            "SELECT id, shortname, item_type, content_enc, nonce, \
                    created_at, updated_at, browser \
             FROM items ORDER BY id ASC",
        )?;
        s.query_map([], |row| {
            Ok(OldItem {
                id: row.get(0)?,
                shortname: row.get(1)?,
                item_type: row.get(2)?,
                content_enc: row.get(3)?,
                nonce: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                browser: row.get(7)?,
            })
        })?
        .collect::<Result<_, _>>()?
    };

    let old_history: Vec<OldHistory> = {
        let mut s = old.prepare(
            "SELECT item_id, content_enc, nonce, version, created_at \
             FROM history ORDER BY item_id ASC, version ASC",
        )?;
        s.query_map([], |row| {
            Ok(OldHistory {
                item_id: row.get(0)?,
                content_enc: row.get(1)?,
                nonce: row.get(2)?,
                version: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?
    };

    let old_tags: Vec<OldTag> = {
        let mut s = old.prepare(
            "SELECT item_id, tag_enc, nonce \
             FROM item_tags ORDER BY item_id ASC, rowid ASC",
        )?;
        s.query_map([], |row| {
            Ok(OldTag {
                item_id: row.get(0)?,
                tag_enc: row.get(1)?,
                nonce: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?
    };

    drop(old);

    // Create new SQLCipher database in a sibling file.
    let new_path = db_path.with_extension("db.new");
    if new_path.exists() {
        std::fs::remove_file(&new_path)?;
    }

    let new = open_sqlcipher(&new_path, &key)?;
    new.execute_batch(
        "CREATE TABLE IF NOT EXISTS items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            shortname   TEXT UNIQUE NOT NULL,
            item_type   TEXT NOT NULL CHECK(item_type IN ('url','note')),
            content     TEXT NOT NULL,
            browser     TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS history (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            content     TEXT NOT NULL,
            version     INTEGER NOT NULL,
            created_at  TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS item_tags (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            tag     TEXT NOT NULL
        );",
    )?;

    // Insert items, preserving original row IDs and timestamps.
    for item in &old_items {
        let content = crypto::decrypt(&key, &item.content_enc, &item.nonce)
            .map_err(|e| anyhow!("Failed to decrypt '{}': {e}", item.shortname))?;
        // Validate UTF-8 by borrowing the still-zeroized bytes, then keep the
        // decrypted plaintext in a Zeroizing<String> so it is wiped on drop.
        let content_str = Zeroizing::new(
            std::str::from_utf8(&content)
                .map_err(|e| anyhow!("Non-UTF-8 content in '{}': {e}", item.shortname))?
                .to_owned(),
        );

        new.execute(
            "INSERT INTO items \
             (id, shortname, item_type, content, browser, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.shortname,
                item.item_type,
                content_str.as_str(),
                item.browser,
                item.created_at,
                item.updated_at,
            ],
        )?;
    }

    for entry in &old_history {
        let content = crypto::decrypt(&key, &entry.content_enc, &entry.nonce)
            .map_err(|e| anyhow!("Failed to decrypt history entry: {e}"))?;
        let content_str = Zeroizing::new(
            std::str::from_utf8(&content)
                .map_err(|e| anyhow!("Non-UTF-8 content in history: {e}"))?
                .to_owned(),
        );

        new.execute(
            "INSERT INTO history (item_id, content, version, created_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                entry.item_id,
                content_str.as_str(),
                entry.version,
                entry.created_at
            ],
        )?;
    }

    for tag in &old_tags {
        let tag_bytes = crypto::decrypt(&key, &tag.tag_enc, &tag.nonce)
            .map_err(|e| anyhow!("Failed to decrypt tag: {e}"))?;
        let tag_str = Zeroizing::new(
            std::str::from_utf8(&tag_bytes)
                .map_err(|e| anyhow!("Non-UTF-8 tag: {e}"))?
                .to_owned(),
        );

        new.execute(
            "INSERT INTO item_tags (item_id, tag) VALUES (?1, ?2)",
            params![tag.item_id, tag_str.as_str()],
        )?;
    }

    drop(new);

    // Write salt file before the rename. If we crash between these two
    // operations, the old plain DB is still intact and can be re-migrated.
    config::write_salt_file(&salt_path, &old_salt)?;

    std::fs::rename(&new_path, db_path)?;

    println!(
        "Migration complete. {} item(s), {} history entry/entries, {} tag(s) migrated.",
        old_items.len(),
        old_history.len(),
        old_tags.len()
    );
    println!("Run 'stash auth login' to start a new session.");
    Ok(())
}

fn open_sqlcipher(path: &std::path::Path, key: &[u8; 32]) -> Result<Connection> {
    use std::fmt::Write;

    // Create the file with 0600 before SQLite writes any encrypted pages.
    config::precreate_private(path);

    let mut hex = Zeroizing::new(String::with_capacity(64));
    for b in key {
        let _ = write!(hex, "{b:02x}");
    }
    let key_pragma = Zeroizing::new(format!("PRAGMA key = \"x'{}'\"", hex.as_str()));

    let conn = Connection::open(path)?;
    conn.execute_batch(&key_pragma)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; \
         PRAGMA foreign_keys=ON; \
         PRAGMA secure_delete=ON;",
    )?;
    config::restrict_db_permissions(path);
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD as B64};
    use chacha20poly1305::{
        ChaCha20Poly1305, Nonce,
        aead::{Aead, KeyInit},
    };
    use rand_core::{OsRng, TryRngCore};
    use rusqlite::{Connection, params};

    use crate::{config, crypto, db::Db};

    /// Encrypt a field with ChaCha20-Poly1305, mirroring the old vault format.
    fn field_encrypt(key: &[u8; 32], plaintext: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let cipher = ChaCha20Poly1305::new(key.into());
        let mut nonce_bytes = [0u8; 12];
        OsRng.try_fill_bytes(&mut nonce_bytes).unwrap();
        let nonce: &Nonce = (&nonce_bytes).into();
        let ct = cipher.encrypt(nonce, plaintext).unwrap();
        (ct, nonce_bytes.to_vec())
    }

    /// Build a plain-SQLite vault in the old field-level-encrypted format.
    /// Returns the Argon2id salt that was used.
    fn build_old_vault(
        db_path: &std::path::Path,
        password: &str,
        items: &[(&str, &str, &str)],  // (shortname, type, content)
        history: &[(&str, i64, &str)], // (shortname, version, content)
        tags: &[(&str, &str)],         // (shortname, tag)
    ) -> String {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(password, &salt).unwrap();
        let key_bytes: &[u8; 32] = &key;

        let conn = Connection::open(db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE items (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 shortname TEXT UNIQUE NOT NULL,
                 item_type TEXT NOT NULL,
                 content_enc BLOB NOT NULL,
                 nonce BLOB NOT NULL,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL,
                 browser TEXT
             );
             CREATE TABLE history (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                 content_enc BLOB NOT NULL,
                 nonce BLOB NOT NULL,
                 version INTEGER NOT NULL,
                 created_at TEXT NOT NULL
             );
             CREATE TABLE item_tags (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                 tag_enc BLOB NOT NULL,
                 nonce BLOB NOT NULL
             );",
        )
        .unwrap();

        // canary
        let (canary_enc, canary_nonce) = field_encrypt(key_bytes, b"stash-auth-canary-v1");
        conn.execute("INSERT INTO meta VALUES ('salt', ?1)", [&salt])
            .unwrap();
        conn.execute(
            "INSERT INTO meta VALUES ('canary_enc', ?1)",
            [&B64.encode(&canary_enc)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO meta VALUES ('canary_nonce', ?1)",
            [&B64.encode(&canary_nonce)],
        )
        .unwrap();

        let ts = "2024-01-01T00:00:00+00:00";
        for (shortname, item_type, content) in items {
            let (enc, nonce) = field_encrypt(key_bytes, content.as_bytes());
            conn.execute(
                "INSERT INTO items (shortname, item_type, content_enc, nonce, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![shortname, item_type, enc, nonce, ts],
            )
            .unwrap();
        }

        for (shortname, version, content) in history {
            let item_id: i64 = conn
                .query_row(
                    "SELECT id FROM items WHERE shortname = ?1",
                    [shortname],
                    |r| r.get(0),
                )
                .unwrap();
            let (enc, nonce) = field_encrypt(key_bytes, content.as_bytes());
            conn.execute(
                "INSERT INTO history (item_id, content_enc, nonce, version, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item_id, enc, nonce, version, ts],
            )
            .unwrap();
        }

        for (shortname, tag) in tags {
            let item_id: i64 = conn
                .query_row(
                    "SELECT id FROM items WHERE shortname = ?1",
                    [shortname],
                    |r| r.get(0),
                )
                .unwrap();
            let (enc, nonce) = field_encrypt(key_bytes, tag.as_bytes());
            conn.execute(
                "INSERT INTO item_tags (item_id, tag_enc, nonce) VALUES (?1, ?2, ?3)",
                params![item_id, enc, nonce],
            )
            .unwrap();
        }

        salt
    }

    // ── guard conditions ──────────────────────────────────────────────────

    #[test]
    fn migrate_no_db_fails() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("nonexistent.db");
        let err = migrate_with_password(&db_path, "any-password-123").unwrap_err();
        assert!(err.to_string().contains("No vault found"), "got: {err}");
    }

    #[test]
    fn migrate_already_migrated_fails() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let salt_path = config::salt_path_for_db(&db_path);
        std::fs::write(&salt_path, "dummy-salt\n").unwrap();
        let err = migrate_with_password(&db_path, "any-password-123").unwrap_err();
        assert!(
            err.to_string().contains("already in SQLCipher format"),
            "got: {err}"
        );
    }

    #[test]
    fn migrate_wrong_password_fails_and_leaves_no_salt_file() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        build_old_vault(
            &db_path,
            "correct-password-123",
            &[("k", "note", "hello")],
            &[],
            &[],
        );

        let err = migrate_with_password(&db_path, "wrong-password-456").unwrap_err();
        assert!(err.to_string().contains("Incorrect password"), "got: {err}");
        assert!(
            !config::salt_path_for_db(&db_path).exists(),
            "salt file must not be created on failure"
        );
    }

    // ── happy path ────────────────────────────────────────────────────────

    #[test]
    fn migrate_empty_vault() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let password = "test-password-12345";
        let old_salt = build_old_vault(&db_path, password, &[], &[], &[]);

        migrate_with_password(&db_path, password).unwrap();

        let salt_path = config::salt_path_for_db(&db_path);
        assert!(salt_path.exists());
        let written_salt = std::fs::read_to_string(&salt_path).unwrap();
        assert_eq!(written_salt.trim(), old_salt, "salt must be preserved");

        let key = crypto::derive_key(password, &old_salt).unwrap();
        let db = Db::open(&db_path, &key).unwrap();
        assert!(db.list_items().unwrap().is_empty());
    }

    #[test]
    fn migrate_preserves_item_content() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let password = "test-password-12345";
        let old_salt = build_old_vault(
            &db_path,
            password,
            &[
                ("note1", "note", "hello world"),
                ("url1", "url", "https://example.com"),
            ],
            &[],
            &[],
        );

        migrate_with_password(&db_path, password).unwrap();

        let key = crypto::derive_key(password, &old_salt).unwrap();
        let db = Db::open(&db_path, &key).unwrap();

        let note = db.get_item("note1").unwrap().unwrap();
        assert_eq!(note.content, "hello world");
        assert_eq!(note.item_type, "note");

        let url = db.get_item("url1").unwrap().unwrap();
        assert_eq!(url.content, "https://example.com");
        assert_eq!(url.item_type, "url");
    }

    #[test]
    fn migrate_preserves_history() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let password = "test-password-12345";
        let old_salt = build_old_vault(
            &db_path,
            password,
            &[("k", "note", "current content")],
            &[("k", 1, "v1 content"), ("k", 2, "v2 content")],
            &[],
        );

        migrate_with_password(&db_path, password).unwrap();

        let key = crypto::derive_key(password, &old_salt).unwrap();
        let db = Db::open(&db_path, &key).unwrap();
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.content, "current content");

        let history = db.get_history(item.id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content, "v1 content");
        assert_eq!(history[1].content, "v2 content");
    }

    #[test]
    fn migrate_preserves_tags() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let password = "test-password-12345";
        let old_salt = build_old_vault(
            &db_path,
            password,
            &[("k", "note", "hello")],
            &[],
            &[("k", "work"), ("k", "personal")],
        );

        migrate_with_password(&db_path, password).unwrap();

        let key = crypto::derive_key(password, &old_salt).unwrap();
        let db = Db::open(&db_path, &key).unwrap();
        let item = db.get_item("k").unwrap().unwrap();
        let tags: Vec<String> = db
            .get_tags(item.id)
            .unwrap()
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert_eq!(tags, vec!["work", "personal"]);
    }

    #[test]
    fn migrate_new_db_is_not_readable_as_plain_sqlite() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        let password = "test-password-12345";
        build_old_vault(&db_path, password, &[("k", "note", "secret")], &[], &[]);

        migrate_with_password(&db_path, password).unwrap();

        // Opening the migrated file without a key must fail.
        let plain = Connection::open(&db_path).unwrap();
        assert!(
            plain.execute_batch("PRAGMA journal_mode=WAL;").is_err(),
            "migrated DB must not be readable without a key"
        );
    }

    #[test]
    fn migrate_old_db_not_modified_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("stash.db");
        build_old_vault(&db_path, "correct-123", &[("k", "note", "data")], &[], &[]);
        let original_size = std::fs::metadata(&db_path).unwrap().len();

        migrate_with_password(&db_path, "wrong-456").unwrap_err();

        // Original DB untouched
        assert_eq!(std::fs::metadata(&db_path).unwrap().len(), original_size);
        // No sibling .db.new left behind
        assert!(!db_path.with_extension("db.new").exists());
    }
}
