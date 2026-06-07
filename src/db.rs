use anyhow::{Result, anyhow};
use rusqlite::{Connection, params};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

pub struct Item {
    pub id: i64,
    pub shortname: String,
    pub item_type: String,
    pub content_enc: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct HistoryEntry {
    pub content_enc: Vec<u8>,
    pub nonce: Vec<u8>,
    pub version: i64,
    pub created_at: String,
}

pub struct Tag {
    pub id: i64,
    pub tag_enc: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                shortname   TEXT UNIQUE NOT NULL,
                item_type   TEXT NOT NULL CHECK(item_type IN ('url','note','secret')),
                content_enc BLOB NOT NULL,
                nonce       BLOB NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                content_enc BLOB NOT NULL,
                nonce       BLOB NOT NULL,
                version     INTEGER NOT NULL,
                created_at  TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS item_tags (
                id      INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                tag_enc BLOB NOT NULL,
                nonce   BLOB NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        match self.conn.query_row(
            "SELECT value FROM meta WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn insert_item(
        &self,
        shortname: &str,
        item_type: &str,
        content_enc: &[u8],
        nonce: &[u8],
    ) -> Result<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO items (shortname, item_type, content_enc, nonce, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![shortname, item_type, content_enc, nonce, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_item(&self, shortname: &str, content_enc: &[u8], nonce: &[u8]) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE items SET content_enc=?1, nonce=?2, updated_at=?3 WHERE shortname=?4",
            params![content_enc, nonce, now, shortname],
        )?;
        if n == 0 {
            return Err(anyhow!("Item '{}' not found", shortname));
        }
        Ok(())
    }

    pub fn get_item(&self, shortname: &str) -> Result<Option<Item>> {
        match self.conn.query_row(
            "SELECT id, shortname, item_type, content_enc, nonce, created_at, updated_at
             FROM items WHERE shortname=?1",
            params![shortname],
            |row| {
                Ok(Item {
                    id: row.get(0)?,
                    shortname: row.get(1)?,
                    item_type: row.get(2)?,
                    content_enc: row.get(3)?,
                    nonce: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        ) {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn add_history(&self, item_id: i64, content_enc: &[u8], nonce: &[u8]) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let version: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM history WHERE item_id=?1",
            params![item_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO history (item_id, content_enc, nonce, version, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![item_id, content_enc, nonce, version, now],
        )?;
        Ok(())
    }

    pub fn get_history(&self, item_id: i64) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT content_enc, nonce, version, created_at
             FROM history WHERE item_id=?1 ORDER BY version ASC",
        )?;
        let entries = stmt
            .query_map(params![item_id], |row| {
                Ok(HistoryEntry {
                    content_enc: row.get(0)?,
                    nonce: row.get(1)?,
                    version: row.get(2)?,
                    created_at: row.get(3)?,
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
            return Err(anyhow!("Item '{}' not found", shortname));
        }
        Ok(())
    }

    pub fn add_tag(&self, item_id: i64, tag_enc: &[u8], nonce: &[u8]) -> Result<()> {
        self.conn.execute(
            "INSERT INTO item_tags (item_id, tag_enc, nonce) VALUES (?1, ?2, ?3)",
            params![item_id, tag_enc, nonce],
        )?;
        Ok(())
    }

    pub fn get_tags(&self, item_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, tag_enc, nonce FROM item_tags WHERE item_id=?1 ORDER BY rowid ASC",
        )?;
        let tags = stmt
            .query_map(params![item_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    tag_enc: row.get(2)?,
                    nonce: row.get(3)?,
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

    pub fn list_items(&self) -> Result<Vec<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, shortname, item_type, content_enc, nonce, created_at, updated_at
             FROM items ORDER BY shortname ASC",
        )?;
        let items = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    shortname: row.get(1)?,
                    item_type: row.get(2)?,
                    content_enc: row.get(3)?,
                    nonce: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub fn item_exists(&self, shortname: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM items WHERE shortname=?1",
            params![shortname],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let db = Db { conn };
        db.migrate().unwrap();
        db
    }

    // ── meta ──────────────────────────────────────────────────────────────

    #[test]
    fn meta_roundtrip() {
        let db = mem_db();
        db.set_meta("key", "value").unwrap();
        assert_eq!(db.get_meta("key").unwrap(), Some("value".into()));
    }

    #[test]
    fn meta_overwrite() {
        let db = mem_db();
        db.set_meta("k", "v1").unwrap();
        db.set_meta("k", "v2").unwrap();
        assert_eq!(db.get_meta("k").unwrap(), Some("v2".into()));
    }

    #[test]
    fn meta_missing_returns_none() {
        let db = mem_db();
        assert_eq!(db.get_meta("nope").unwrap(), None);
    }

    // ── items ─────────────────────────────────────────────────────────────

    #[test]
    fn insert_and_get() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"enc", b"nonce").unwrap();
        assert!(id > 0);
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.shortname, "k");
        assert_eq!(item.item_type, "note");
        assert_eq!(item.content_enc, b"enc");
        assert_eq!(item.nonce, b"nonce");
    }

    #[test]
    fn get_missing_returns_none() {
        assert!(mem_db().get_item("ghost").unwrap().is_none());
    }

    #[test]
    fn item_exists_true_after_insert() {
        let db = mem_db();
        db.insert_item("k", "url", b"e", b"n").unwrap();
        assert!(db.item_exists("k").unwrap());
    }

    #[test]
    fn item_exists_false_for_unknown() {
        assert!(!mem_db().item_exists("x").unwrap());
    }

    #[test]
    fn duplicate_shortname_fails() {
        let db = mem_db();
        db.insert_item("k", "note", b"e", b"n").unwrap();
        assert!(db.insert_item("k", "note", b"e2", b"n2").is_err());
    }

    #[test]
    fn invalid_item_type_fails() {
        assert!(mem_db().insert_item("k", "bogus", b"e", b"n").is_err());
    }

    #[test]
    fn update_item() {
        let db = mem_db();
        db.insert_item("k", "note", b"old", b"n1").unwrap();
        db.update_item("k", b"new", b"n2").unwrap();
        let item = db.get_item("k").unwrap().unwrap();
        assert_eq!(item.content_enc, b"new");
        assert_eq!(item.nonce, b"n2");
    }

    #[test]
    fn update_nonexistent_fails() {
        assert!(mem_db().update_item("ghost", b"e", b"n").is_err());
    }

    #[test]
    fn delete_item() {
        let db = mem_db();
        db.insert_item("k", "secret", b"e", b"n").unwrap();
        db.delete_item("k").unwrap();
        assert!(!db.item_exists("k").unwrap());
    }

    #[test]
    fn delete_nonexistent_fails() {
        assert!(mem_db().delete_item("ghost").is_err());
    }

    // ── history ───────────────────────────────────────────────────────────

    #[test]
    fn history_empty_for_new_item() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        assert!(db.get_history(id).unwrap().is_empty());
    }

    #[test]
    fn history_versions_increment() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        db.add_history(id, b"v1", b"n1").unwrap();
        db.add_history(id, b"v2", b"n2").unwrap();
        db.add_history(id, b"v3", b"n3").unwrap();
        let h = db.get_history(id).unwrap();
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].version, 1);
        assert_eq!(h[1].version, 2);
        assert_eq!(h[2].version, 3);
    }

    #[test]
    fn history_preserves_content() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"current", b"cn").unwrap();
        db.add_history(id, b"old content", b"old nonce").unwrap();
        let h = db.get_history(id).unwrap();
        assert_eq!(h[0].content_enc, b"old content");
        assert_eq!(h[0].nonce, b"old nonce");
    }

    #[test]
    fn history_ordered_ascending() {
        let db = mem_db();
        let id = db.insert_item("k", "url", b"e", b"n").unwrap();
        db.add_history(id, b"a", b"na").unwrap();
        db.add_history(id, b"b", b"nb").unwrap();
        let h = db.get_history(id).unwrap();
        assert!(h.windows(2).all(|w| w[0].version < w[1].version));
    }

    // ── tags ──────────────────────────────────────────────────────────────

    #[test]
    fn add_and_get_tags() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        db.add_tag(id, b"tag1_enc", b"n1").unwrap();
        db.add_tag(id, b"tag2_enc", b"n2").unwrap();
        let tags = db.get_tags(id).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag_enc, b"tag1_enc");
        assert_eq!(tags[1].tag_enc, b"tag2_enc");
    }

    #[test]
    fn get_tags_empty() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn delete_tag() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        db.add_tag(id, b"enc", b"nonce").unwrap();
        let tags = db.get_tags(id).unwrap();
        db.delete_tag(tags[0].id).unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn tags_cascade_on_item_delete() {
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        db.add_tag(id, b"enc1", b"n1").unwrap();
        db.add_tag(id, b"enc2", b"n2").unwrap();
        db.delete_item("k").unwrap();
        assert!(db.get_tags(id).unwrap().is_empty());
    }

    #[test]
    fn tags_are_per_item() {
        let db = mem_db();
        let id1 = db.insert_item("a", "note", b"e", b"n").unwrap();
        let id2 = db.insert_item("b", "note", b"e", b"n").unwrap();
        db.add_tag(id1, b"tag_a", b"na").unwrap();
        db.add_tag(id2, b"tag_b", b"nb").unwrap();
        assert_eq!(db.get_tags(id1).unwrap().len(), 1);
        assert_eq!(db.get_tags(id2).unwrap().len(), 1);
        assert_eq!(db.get_tags(id1).unwrap()[0].tag_enc, b"tag_a");
    }

    // ── list_items ────────────────────────────────────────────────────────

    #[test]
    fn list_items_empty() {
        assert!(mem_db().list_items().unwrap().is_empty());
    }

    #[test]
    fn list_items_returns_all() {
        let db = mem_db();
        db.insert_item("a", "note", b"e1", b"n1").unwrap();
        db.insert_item("b", "url", b"e2", b"n2").unwrap();
        db.insert_item("c", "secret", b"e3", b"n3").unwrap();
        assert_eq!(db.list_items().unwrap().len(), 3);
    }

    #[test]
    fn list_items_ordered_by_shortname() {
        let db = mem_db();
        db.insert_item("charlie", "note", b"e", b"n").unwrap();
        db.insert_item("alpha", "note", b"e", b"n").unwrap();
        db.insert_item("bravo", "note", b"e", b"n").unwrap();
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
        let db = mem_db();
        let id = db.insert_item("k", "note", b"e", b"n").unwrap();
        db.add_history(id, b"v1", b"n1").unwrap();
        db.add_history(id, b"v2", b"n2").unwrap();
        db.delete_item("k").unwrap();
        assert!(db.get_history(id).unwrap().is_empty());
    }
}
