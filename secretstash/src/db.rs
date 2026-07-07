use anyhow::{Result, anyhow};
use rusqlite::{Connection, ErrorCode, params};
use std::path::Path;
use zeroize::Zeroizing;

use crate::config;

pub struct Db {
    conn: Connection,
}

pub struct Item {
    pub id: i64,
    pub shortname: String,
    pub item_type: String,
    pub content: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub browser: Option<String>,
    pub private: Option<bool>,
}

pub struct HistoryEntry {
    pub content: String,
    pub title: Option<String>,
    pub version: i64,
    pub created_at: String,
}

pub struct Tag {
    pub id: i64,
    pub tag: String,
}

/// Hex-encode a key into a `Zeroizing` string so the only copies of the key
/// material in this representation are wiped when dropped. Writing into a
/// pre-sized buffer avoids the per-byte temporary allocations that
/// `map(format!).collect()` would leave un-zeroized on the heap.
fn hex_key(key: &[u8; 32]) -> Zeroizing<String> {
    use std::fmt::Write;
    let mut s = Zeroizing::new(String::with_capacity(64));
    for b in key {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// SQLCipher reports `SQLITE_NOTADB` when the key is wrong (the decrypted
/// first page fails the file-format sanity check) or when the file predates
/// whole-database encryption. Any other error (I/O, permissions, disk full,
/// ...) is a real failure unrelated to the password.
fn is_bad_key_error(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(se, _) if se.code == ErrorCode::NotADatabase
    )
}

impl Db {
    /// Open (or create) a SQLCipher-encrypted database at `path` using `key`.
    /// Returns an error if the file exists but the key is wrong, with a hint
    /// to run `stash migrate` when the file looks like a plain `SQLite` DB.
    pub fn open(path: &Path, key: &[u8; 32]) -> Result<Self> {
        // Create the DB file with 0600 up front so SQLite never materialises a
        // fresh vault with the process umask (typically world-readable).
        config::precreate_private(path);

        let conn = Connection::open(path)?;
        let key_pragma = Zeroizing::new(format!("PRAGMA key = \"x'{}'\"", hex_key(key).as_str()));
        conn.execute_batch(&key_pragma)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; \
             PRAGMA foreign_keys=ON; \
             PRAGMA secure_delete=ON;",
        )
        .map_err(|e| {
            if !is_bad_key_error(&e) {
                return anyhow::Error::from(e);
            }
            if path.exists() && !path.with_extension("salt").exists() {
                anyhow!(
                    "Cannot open vault: it appears to be in the old unencrypted format. \
                     Run 'stash migrate' to convert it."
                )
            } else {
                anyhow!("Cannot open vault: wrong password or corrupted database.")
            }
        })?;
        // The WAL/SHM sidecars are created by the pragma above; lock them down too.
        config::restrict_db_permissions(path);
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                shortname   TEXT UNIQUE NOT NULL,
                item_type   TEXT NOT NULL CHECK(item_type IN ('url','note')),
                content     TEXT NOT NULL,
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

        let columns: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(items)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(std::result::Result::ok)
            .collect();

        if !columns.iter().any(|c| c == "browser") {
            self.conn
                .execute_batch("ALTER TABLE items ADD COLUMN browser TEXT")?;
        }
        if !columns.iter().any(|c| c == "private") {
            self.conn
                .execute_batch("ALTER TABLE items ADD COLUMN private INTEGER")?;
        }
        if !columns.iter().any(|c| c == "title") {
            self.conn
                .execute_batch("ALTER TABLE items ADD COLUMN title TEXT")?;
        }

        let history_cols: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(history)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(std::result::Result::ok)
            .collect();
        if !history_cols.iter().any(|c| c == "title") {
            self.conn
                .execute_batch("ALTER TABLE history ADD COLUMN title TEXT")?;
        }

        Ok(())
    }

    /// Change the `SQLCipher` encryption key (used by `stash auth reset`).
    pub fn rekey(&self, new_key: &[u8; 32]) -> Result<()> {
        let rekey_pragma = Zeroizing::new(format!(
            "PRAGMA rekey = \"x'{}'\"",
            hex_key(new_key).as_str()
        ));
        self.conn.execute_batch(&rekey_pragma)?;
        Ok(())
    }

    pub fn insert_item(
        &self,
        shortname: &str,
        item_type: &str,
        content: &str,
        title: Option<&str>,
        browser: Option<&str>,
    ) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO items (shortname, item_type, content, title, browser, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![shortname, item_type, content, title, browser, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_item_full(
        &self,
        shortname: &str,
        item_type: &str,
        content: &str,
        title: Option<&str>,
        browser: Option<&str>,
        created_at: &str,
        updated_at: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO items (shortname, item_type, content, title, browser, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![shortname, item_type, content, title, browser, created_at, updated_at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_history_entry(
        &self,
        item_id: i64,
        content: &str,
        title: Option<&str>,
        version: i64,
        created_at: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history (item_id, content, title, version, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![item_id, content, title, version, created_at],
        )?;
        Ok(())
    }

    pub fn get_item(&self, shortname: &str) -> Result<Option<Item>> {
        match self.conn.query_row(
            "SELECT id, shortname, item_type, content, created_at, updated_at, browser, private, title
             FROM items WHERE shortname=?1",
            params![shortname],
            |row| {
                Ok(Item {
                    id: row.get(0)?,
                    shortname: row.get(1)?,
                    item_type: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    browser: row.get(6)?,
                    private: row.get(7)?,
                    title: row.get(8)?,
                })
            },
        ) {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_history(&self, item_id: i64) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT content, version, created_at, title
             FROM history WHERE item_id=?1 ORDER BY version ASC",
        )?;
        let entries = stmt
            .query_map(params![item_id], |row| {
                Ok(HistoryEntry {
                    content: row.get(0)?,
                    version: row.get(1)?,
                    created_at: row.get(2)?,
                    title: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    pub fn delete_item(&self, shortname: &str) -> Result<()> {
        let n = self
            .conn
            .execute("DELETE FROM items WHERE shortname=?1", params![shortname])?;
        if n == 0 {
            return Err(anyhow!("Item '{shortname}' not found"));
        }
        Ok(())
    }

    pub fn add_tag(&self, item_id: i64, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO item_tags (item_id, tag) VALUES (?1, ?2)",
            params![item_id, tag],
        )?;
        Ok(())
    }

    pub fn get_tags(&self, item_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, tag FROM item_tags WHERE item_id=?1 ORDER BY rowid ASC")?;
        let tags = stmt
            .query_map(params![item_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    tag: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn delete_tag(&self, tag_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM item_tags WHERE id=?1", params![tag_id])?;
        Ok(())
    }

    pub fn list_all_tags(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag, COUNT(*) FROM item_tags GROUP BY tag ORDER BY tag ASC")?;
        let tags = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn list_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, shortname, item_type, content, created_at, updated_at, browser, private, title
             FROM items ORDER BY shortname ASC",
        )?;
        let items = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    shortname: row.get(1)?,
                    item_type: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    browser: row.get(6)?,
                    private: row.get(7)?,
                    title: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub fn rename_item(&self, old: &str, new: &str) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE items SET shortname=?1 WHERE shortname=?2",
            params![new, old],
        )?;
        if n == 0 {
            return Err(anyhow!("Item '{old}' not found"));
        }
        Ok(())
    }

    pub fn get_history_version(&self, item_id: i64, version: i64) -> Result<Option<HistoryEntry>> {
        match self.conn.query_row(
            "SELECT content, version, created_at, title
             FROM history WHERE item_id=?1 AND version=?2",
            params![item_id, version],
            |row| {
                Ok(HistoryEntry {
                    content: row.get(0)?,
                    version: row.get(1)?,
                    created_at: row.get(2)?,
                    title: row.get(3)?,
                })
            },
        ) {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_latest_history(&self, item_id: i64) -> Result<Option<HistoryEntry>> {
        match self.conn.query_row(
            "SELECT content, version, created_at, title
             FROM history WHERE item_id=?1 ORDER BY version DESC LIMIT 1",
            params![item_id],
            |row| {
                Ok(HistoryEntry {
                    content: row.get(0)?,
                    version: row.get(1)?,
                    created_at: row.get(2)?,
                    title: row.get(3)?,
                })
            },
        ) {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_browser(&self, shortname: &str, browser: Option<&str>) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE items SET browser=?1 WHERE shortname=?2",
            params![browser, shortname],
        )?;
        if n == 0 {
            return Err(anyhow!("Item '{shortname}' not found"));
        }
        Ok(())
    }

    pub fn set_private(&self, shortname: &str, private: Option<bool>) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE items SET private=?1 WHERE shortname=?2",
            params![private, shortname],
        )?;
        if n == 0 {
            return Err(anyhow!("Item '{shortname}' not found"));
        }
        Ok(())
    }

    pub fn item_exists(&self, shortname: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM items WHERE shortname=?1",
            params![shortname],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Archive the current content+title as a history entry and write new values atomically.
    pub fn replace_content(
        &self,
        item_id: i64,
        shortname: &str,
        old_content: &str,
        old_title: Option<&str>,
        new_content: &str,
        new_title: Option<&str>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let now = chrono::Utc::now().to_rfc3339();

        let version: i64 = tx.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM history WHERE item_id=?1",
            params![item_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO history (item_id, content, title, version, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![item_id, old_content, old_title, version, now],
        )?;

        let n = tx.execute(
            "UPDATE items SET content=?1, title=?2, updated_at=?3 WHERE shortname=?4",
            params![new_content, new_title, now, shortname],
        )?;
        if n == 0 {
            return Err(anyhow!("Item '{shortname}' not found"));
        }

        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_bad_key_error ──────────────────────────────────────────────────

    #[test]
    fn is_bad_key_error_true_for_not_a_database() {
        let err = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_NOTADB),
            None,
        );
        assert!(is_bad_key_error(&err));
    }

    #[test]
    fn is_bad_key_error_false_for_other_sqlite_errors() {
        let err = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR),
            None,
        );
        assert!(!is_bad_key_error(&err));
    }

    #[test]
    fn is_bad_key_error_false_for_non_sqlite_errors() {
        assert!(!is_bad_key_error(&rusqlite::Error::QueryReturnedNoRows));
    }

    // ── Db::open error messages ───────────────────────────────────────────

    #[test]
    fn open_plain_sqlite_suggests_migrate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stash.db");
        // Build a plain (unencrypted) SQLite file — no .salt file alongside it.
        let plain = Connection::open(&path).unwrap();
        plain.execute_batch("CREATE TABLE dummy (x TEXT);").unwrap();
        drop(plain);

        let key = [0x42u8; 32];
        let err = Db::open(&path, &key)
            .err()
            .expect("expected Db::open to fail");
        let msg = err.to_string();
        assert!(
            msg.contains("stash migrate"),
            "expected 'stash migrate' hint in error, got: {msg}"
        );
    }

    #[test]
    fn open_sqlcipher_wrong_key_no_migrate_hint() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stash.db");
        let salt_path = path.with_extension("salt");
        let correct_key = [0x11u8; 32];

        // Create a properly-encrypted SQLCipher vault.
        {
            let db = Db::open(&path, &correct_key).unwrap();
            db.insert_item("k", "note", "x", None, None).unwrap();
        }
        // A salt file marks this as already-migrated SQLCipher format.
        std::fs::write(&salt_path, "dummy-salt\n").unwrap();

        let wrong_key = [0x22u8; 32];
        let err = Db::open(&path, &wrong_key)
            .err()
            .expect("expected Db::open to fail");
        let msg = err.to_string();
        assert!(
            !msg.contains("stash migrate"),
            "must NOT suggest 'stash migrate' for wrong password, got: {msg}"
        );
        assert!(
            msg.contains("wrong password") || msg.contains("corrupted"),
            "expected wrong-password error, got: {msg}"
        );
    }

    // ── file permissions ─────────────────────────────────────────────────

    #[cfg(unix)]
    #[test]
    fn open_creates_vault_with_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let key = [0x42u8; 32];

        let db = Db::open(&path, &key).unwrap();
        db.insert_item("k", "note", "x", None, None).unwrap();

        // The main DB file and any WAL/SHM sidecars must not be group/world
        // accessible — only the owner may read the encrypted vault.
        for suffix in ["", "-wal", "-shm"] {
            let mut p = path.as_os_str().to_os_string();
            p.push(suffix);
            let p = std::path::PathBuf::from(p);
            if p.exists() {
                let mode = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
                assert_eq!(
                    mode,
                    0o600,
                    "{} has mode {:o}, expected 600",
                    p.display(),
                    mode
                );
            }
        }
    }

    // ── SQLCipher proof-of-concept ────────────────────────────────────────

    #[test]
    fn sqlcipher_encrypted_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let key = [0x42u8; 32];

        {
            let db = Db::open(&path, &key).unwrap();
            db.insert_item("k", "note", "hello sqlcipher", None, None)
                .unwrap();
        }

        // Correct key — data readable
        let db = Db::open(&path, &key).unwrap();
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.content, "hello sqlcipher");
        drop(db);

        // Wrong key — must fail
        let wrong = [0x99u8; 32];
        assert!(Db::open(&path, &wrong).is_err());
    }

    #[test]
    fn sqlcipher_rekey_changes_encryption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let key1 = [0x11u8; 32];
        let key2 = [0x22u8; 32];

        {
            let db = Db::open(&path, &key1).unwrap();
            db.insert_item("k", "note", "secret", None, None).unwrap();
            db.rekey(&key2).unwrap();
        }

        // Old key no longer works
        assert!(Db::open(&path, &key1).is_err());

        // New key works and data is intact
        let db = Db::open(&path, &key2).unwrap();
        assert_eq!(db.get_item("k").unwrap().unwrap().content, "secret");
    }

    // ── items ─────────────────────────────────────────────────────────────

    #[test]
    fn insert_and_get() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "hello", None, None).unwrap();
        assert!(id > 0);
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.shortname, "k");
        assert_eq!(item.item_type, "note");
        assert_eq!(item.content, "hello");
    }

    #[test]
    fn get_missing_returns_none() {
        assert!(
            Db::open_in_memory()
                .unwrap()
                .get_item("ghost")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn item_exists_true_after_insert() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("k", "url", "https://x.com", None, None)
            .unwrap();
        assert!(db.item_exists("k").unwrap());
    }

    #[test]
    fn item_exists_false_for_unknown() {
        assert!(!Db::open_in_memory().unwrap().item_exists("x").unwrap());
    }

    #[test]
    fn duplicate_shortname_fails() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("k", "note", "a", None, None).unwrap();
        assert!(db.insert_item("k", "note", "b", None, None).is_err());
    }

    #[test]
    fn invalid_item_type_fails() {
        assert!(
            Db::open_in_memory()
                .unwrap()
                .insert_item("k", "bogus", "x", None, None)
                .is_err()
        );
    }

    #[test]
    fn delete_item() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("k", "note", "x", None, None).unwrap();
        db.delete_item("k").unwrap();
        assert!(!db.item_exists("k").unwrap());
    }

    #[test]
    fn delete_nonexistent_fails() {
        assert!(Db::open_in_memory().unwrap().delete_item("ghost").is_err());
    }

    // ── history ───────────────────────────────────────────────────────────

    #[test]
    fn history_empty_for_new_item() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        assert!(db.get_history(id).unwrap().is_empty());
    }

    #[test]
    fn replace_content_archives_old_and_writes_new() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "old", None, None).unwrap();
        db.replace_content(id, "k", "old", None, "new", None)
            .unwrap();
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.content, "new");
        let history = db.get_history(id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "old");
    }

    #[test]
    fn replace_content_increments_version() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "v1", None, None).unwrap();
        db.replace_content(id, "k", "v1", None, "v2", None).unwrap();
        db.replace_content(id, "k", "v2", None, "v3", None).unwrap();
        let history = db.get_history(id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
    }

    #[test]
    fn history_ordered_ascending() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .insert_item("k", "url", "https://a.com", None, None)
            .unwrap();
        db.replace_content(id, "k", "https://a.com", None, "https://b.com", None)
            .unwrap();
        db.replace_content(id, "k", "https://b.com", None, "https://c.com", None)
            .unwrap();
        let h = db.get_history(id).unwrap();
        assert!(h.windows(2).all(|w| w[0].version < w[1].version));
    }

    // ── tags ──────────────────────────────────────────────────────────────

    #[test]
    fn add_and_get_tags() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        db.add_tag(id, "work").unwrap();
        db.add_tag(id, "personal").unwrap();
        let tags = db.get_tags(id).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag, "work");
        assert_eq!(tags[1].tag, "personal");
    }

    #[test]
    fn get_tags_empty() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn delete_tag() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        db.add_tag(id, "work").unwrap();
        let tag_id = db.get_tags(id).unwrap()[0].id;
        db.delete_tag(tag_id).unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn tags_cascade_on_item_delete() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        db.add_tag(id, "a").unwrap();
        db.add_tag(id, "b").unwrap();
        db.delete_item("k").unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn list_all_tags_with_counts() {
        let db = Db::open_in_memory().unwrap();
        let id1 = db.insert_item("a", "note", "x", None, None).unwrap();
        let id2 = db.insert_item("b", "note", "y", None, None).unwrap();
        db.add_tag(id1, "work").unwrap();
        db.add_tag(id1, "personal").unwrap();
        db.add_tag(id2, "work").unwrap();
        let tags = db.list_all_tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], ("personal".to_string(), 1));
        assert_eq!(tags[1], ("work".to_string(), 2));
    }

    #[test]
    fn list_all_tags_empty() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.list_all_tags().unwrap().is_empty());
    }

    #[test]
    fn tags_are_per_item() {
        let db = Db::open_in_memory().unwrap();
        let id1 = db.insert_item("a", "note", "x", None, None).unwrap();
        let id2 = db.insert_item("b", "note", "y", None, None).unwrap();
        db.add_tag(id1, "alpha").unwrap();
        db.add_tag(id2, "beta").unwrap();
        assert_eq!(db.get_tags(id1).unwrap()[0].tag, "alpha");
        assert_eq!(db.get_tags(id2).unwrap()[0].tag, "beta");
    }

    // ── list_items ────────────────────────────────────────────────────────

    #[test]
    fn list_items_empty() {
        assert!(
            Db::open_in_memory()
                .unwrap()
                .list_items()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn list_items_ordered_by_shortname() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("charlie", "note", "x", None, None).unwrap();
        db.insert_item("alpha", "note", "x", None, None).unwrap();
        db.insert_item("bravo", "note", "x", None, None).unwrap();
        let names: Vec<_> = db
            .list_items()
            .unwrap()
            .into_iter()
            .map(|i| i.shortname)
            .collect();
        assert_eq!(names, ["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn delete_cascades_to_history() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "v1", None, None).unwrap();
        db.replace_content(id, "k", "v1", None, "v2", None).unwrap();
        db.delete_item("k").unwrap();
        assert!(db.get_history(id).unwrap().is_empty());
    }

    // ── rename_item ───────────────────────────────────────────────────────

    #[test]
    fn rename_item() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("old", "note", "x", None, None).unwrap();
        db.rename_item("old", "new").unwrap();
        assert!(db.get_item("old").unwrap().is_none());
        assert_eq!(db.get_item("new").unwrap().unwrap().shortname, "new");
    }

    #[test]
    fn rename_nonexistent_fails() {
        assert!(
            Db::open_in_memory()
                .unwrap()
                .rename_item("ghost", "other")
                .is_err()
        );
    }

    #[test]
    fn rename_to_existing_name_fails() {
        let db = Db::open_in_memory().unwrap();
        db.insert_item("a", "note", "x", None, None).unwrap();
        db.insert_item("b", "note", "y", None, None).unwrap();
        assert!(db.rename_item("a", "b").is_err());
    }

    // ── get_history_version / get_latest_history ──────────────────────────

    #[test]
    fn get_history_version_found() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "v1", None, None).unwrap();
        db.replace_content(id, "k", "v1", None, "v2", None).unwrap();
        let e = db.get_history_version(id, 1).unwrap().unwrap();
        assert_eq!(e.content, "v1");
        assert_eq!(e.version, 1);
    }

    #[test]
    fn get_history_version_not_found() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        assert!(db.get_history_version(id, 99).unwrap().is_none());
    }

    #[test]
    fn get_latest_history_returns_max_version() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "v1", None, None).unwrap();
        db.replace_content(id, "k", "v1", None, "v2", None).unwrap();
        db.replace_content(id, "k", "v2", None, "v3", None).unwrap();
        db.replace_content(id, "k", "v3", None, "v4", None).unwrap();
        let e = db.get_latest_history(id).unwrap().unwrap();
        assert_eq!(e.version, 3);
        assert_eq!(e.content, "v3");
    }

    #[test]
    fn get_latest_history_empty() {
        let db = Db::open_in_memory().unwrap();
        let id = db.insert_item("k", "note", "x", None, None).unwrap();
        assert!(db.get_latest_history(id).unwrap().is_none());
    }
}
