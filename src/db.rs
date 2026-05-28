use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Represents a single sticky note window with its position, size, and style.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: i64,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub color: String,
    pub always_on_top: bool,
}

/// Represents a content block inside a note's canvas (text, checklist, code, etc).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextBlock {
    pub id: i64,
    pub note_id: i64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub content: String,
}

/// Thread-safe database wrapper for the application's SQLite store.
///
/// SQLite's `Connection` is not `Send`/`Sync` because concurrent access from
/// multiple threads can cause data races. Instead of marking the type with
/// `unsafe impl Send + Sync` (which would be unsound if any method ever ran
/// concurrently), we wrap the connection in a `Mutex` so the Rust compiler
/// enforces exclusive access at runtime. This makes `Db` safely `Send + Sync`
/// through the `Mutex` contract — no `unsafe` required.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Opens (or creates) the database at `~/.local/share/sticky/notes.db`
    /// and runs all necessary migrations.
    pub fn init() -> Result<Self> {
        let mut db_path = glib::user_data_dir();
        db_path.push("sticky");
        std::fs::create_dir_all(&db_path).unwrap_or_default();
        db_path.push("notes.db");

        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                x INTEGER,
                y INTEGER,
                width INTEGER,
                height INTEGER,
                color TEXT,
                always_on_top BOOLEAN,
                deleted BOOLEAN DEFAULT 0
            )",
            [],
        )?;
        // Migration: add `deleted` column for databases created before soft-delete.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN deleted BOOLEAN DEFAULT 0", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                note_id INTEGER,
                x REAL,
                y REAL,
                width REAL,
                height REAL,
                content TEXT,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS block_links (
                source_id INTEGER,
                target_id INTEGER,
                PRIMARY KEY (source_id, target_id),
                FOREIGN KEY(source_id) REFERENCES blocks(id) ON DELETE CASCADE,
                FOREIGN KEY(target_id) REFERENCES blocks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Returns the path to the application's data directory
    /// (e.g. `~/.local/share/sticky/`). Used for temp audio files so we
    /// avoid polluting `/tmp` with user recordings.
    pub fn data_dir() -> std::path::PathBuf {
        let mut p = glib::user_data_dir();
        p.push("sticky");
        std::fs::create_dir_all(&p).unwrap_or_default();
        p
    }

    /// Fetches all non-deleted notes.
    pub fn get_notes(&self) -> Result<Vec<Note>> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, x, y, width, height, color, always_on_top FROM notes WHERE deleted = 0",
        )?;
        let note_iter = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                x: row.get(1)?,
                y: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                color: row.get(5)?,
                always_on_top: row.get(6)?,
            })
        })?;

        let mut notes = Vec::new();
        for note in note_iter {
            notes.push(note?);
        }
        Ok(notes)
    }

    /// Fetches all blocks belonging to a specific note.
    pub fn get_blocks(&self, note_id: i64) -> Result<Vec<TextBlock>> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, note_id, x, y, width, height, content FROM blocks WHERE note_id = ?",
        )?;
        let block_iter = stmt.query_map([note_id], |row| {
            Ok(TextBlock {
                id: row.get(0)?,
                note_id: row.get(1)?,
                x: row.get(2)?,
                y: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                content: row.get(6)?,
            })
        })?;

        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block?);
        }
        Ok(blocks)
    }

    /// Performs a full-text search across all block content using SQL LIKE.
    pub fn search_blocks(&self, query: &str) -> Result<Vec<TextBlock>> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let sql = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, note_id, x, y, width, height, content FROM blocks WHERE content LIKE ?",
        )?;
        let block_iter = stmt.query_map([sql], |row| {
            Ok(TextBlock {
                id: row.get(0)?,
                note_id: row.get(1)?,
                x: row.get(2)?,
                y: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                content: row.get(6)?,
            })
        })?;

        let mut blocks = Vec::new();
        for block in block_iter {
            blocks.push(block?);
        }
        Ok(blocks)
    }

    pub fn create_note(&self, x: i32, y: i32, color: &str) -> Result<i64> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "INSERT INTO notes (x, y, width, height, color, always_on_top) \
             VALUES (?, ?, ?, ?, ?, ?)",
            params![x, y, 300, 300, color, true],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_note_pos(&self, id: i64, x: i32, y: i32) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "UPDATE notes SET x = ?, y = ? WHERE id = ?",
            params![x, y, id],
        )?;
        Ok(())
    }

    pub fn update_note_color(&self, id: i64, color: &str) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "UPDATE notes SET color = ? WHERE id = ?",
            params![color, id],
        )?;
        Ok(())
    }

    pub fn update_note_size(&self, id: i64, w: i32, h: i32) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "UPDATE notes SET width = ?, height = ? WHERE id = ?",
            params![w, h, id],
        )?;
        Ok(())
    }

    pub fn update_note_always_on_top(&self, id: i64, always: bool) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "UPDATE notes SET always_on_top = ? WHERE id = ?",
            params![always, id],
        )?;
        Ok(())
    }

    /// Soft-deletes a note (moves it to the Recycle Bin).
    pub fn delete_note(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute("UPDATE notes SET deleted = 1 WHERE id = ?", [id])?;
        Ok(())
    }

    /// Restores the most recently soft-deleted note from the Recycle Bin.
    pub fn restore_last_deleted_note(&self) -> Result<Option<Note>> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, x, y, width, height, color, always_on_top \
             FROM notes WHERE deleted = 1 ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let note = Note {
                id: row.get(0)?,
                x: row.get(1)?,
                y: row.get(2)?,
                width: row.get(3)?,
                height: row.get(4)?,
                color: row.get(5)?,
                always_on_top: row.get(6)?,
            };
            conn.execute("UPDATE notes SET deleted = 0 WHERE id = ?", [note.id])?;
            return Ok(Some(note));
        }
        Ok(None)
    }

    /// Permanently deletes all soft-deleted notes and their associated blocks.
    pub fn empty_trash(&self) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let mut stmt = conn.prepare("SELECT id FROM notes WHERE deleted = 1")?;
        let mut note_ids = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            note_ids.push(row.get::<_, i64>(0)?);
        }

        for id in note_ids {
            conn.execute("DELETE FROM blocks WHERE note_id = ?", [id])?;
            conn.execute("DELETE FROM notes WHERE id = ?", [id])?;
        }
        Ok(())
    }

    /// Inserts a new block or updates an existing one (upsert by id).
    pub fn upsert_block(&self, block: &TextBlock) -> Result<i64> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        if block.id == 0 {
            conn.execute(
                "INSERT INTO blocks (note_id, x, y, width, height, content) \
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    block.note_id,
                    block.x,
                    block.y,
                    block.width,
                    block.height,
                    block.content
                ],
            )?;
            Ok(conn.last_insert_rowid())
        } else {
            conn.execute(
                "UPDATE blocks SET x = ?, y = ?, width = ?, height = ?, content = ? WHERE id = ?",
                params![
                    block.x,
                    block.y,
                    block.width,
                    block.height,
                    block.content,
                    block.id
                ],
            )?;
            Ok(block.id)
        }
    }

    pub fn delete_block(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute("DELETE FROM blocks WHERE id = ?", [id])?;
        Ok(())
    }

    /// Creates a directional mind-map link between two blocks.
    pub fn link_blocks(&self, source_id: i64, target_id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO block_links (source_id, target_id) VALUES (?, ?)",
            params![source_id, target_id],
        )?;
        Ok(())
    }

    /// Returns all (source_id, target_id) links for blocks belonging to a note.
    pub fn get_links_for_note(&self, note_id: i64) -> Result<Vec<(i64, i64)>> {
        let conn = self.conn.lock().expect("DB mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT l.source_id, l.target_id
             FROM block_links l
             JOIN blocks b ON l.source_id = b.id
             WHERE b.note_id = ?",
        )?;
        let iter = stmt.query_map([note_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut links = Vec::new();
        for l in iter {
            links.push(l?);
        }
        Ok(links)
    }
}
