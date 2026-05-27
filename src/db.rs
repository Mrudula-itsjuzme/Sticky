use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

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

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn init() -> Result<Self> {
        let mut db_path = glib::user_data_dir();
        db_path.push("antigrav");
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
                always_on_top BOOLEAN
            )",
            [],
        )?;

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

        Ok(Self { conn })
    }

    pub fn get_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare("SELECT id, x, y, width, height, color, always_on_top FROM notes")?;
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

    pub fn get_blocks(&self, note_id: i64) -> Result<Vec<TextBlock>> {
        let mut stmt = self.conn.prepare("SELECT id, note_id, x, y, width, height, content FROM blocks WHERE note_id = ?")?;
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

    pub fn create_note(&self, x: i32, y: i32, color: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (x, y, width, height, color, always_on_top) VALUES (?, ?, ?, ?, ?, ?)",
            params![x, y, 300, 300, color, true],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_note_pos(&self, id: i64, x: i32, y: i32) -> Result<()> {
        self.conn.execute("UPDATE notes SET x = ?, y = ? WHERE id = ?", params![x, y, id])?;
        Ok(())
    }

    pub fn update_note_color(&self, id: i64, color: &str) -> Result<()> {
        self.conn.execute("UPDATE notes SET color = ? WHERE id = ?", params![color, id])?;
        Ok(())
    }

    pub fn update_note_size(&self, id: i64, w: i32, h: i32) -> Result<()> {
        self.conn.execute("UPDATE notes SET width = ?, height = ? WHERE id = ?", params![w, h, id])?;
        Ok(())
    }

    pub fn update_note_always_on_top(&self, id: i64, always: bool) -> Result<()> {
        self.conn.execute("UPDATE notes SET always_on_top = ? WHERE id = ?", params![always, id])?;
        Ok(())
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn upsert_block(&self, block: &TextBlock) -> Result<i64> {
        if block.id == 0 {
            self.conn.execute(
                "INSERT INTO blocks (note_id, x, y, width, height, content) VALUES (?, ?, ?, ?, ?, ?)",
                params![block.note_id, block.x, block.y, block.width, block.height, block.content],
            )?;
            Ok(self.conn.last_insert_rowid())
        } else {
            self.conn.execute(
                "UPDATE blocks SET x = ?, y = ?, width = ?, height = ?, content = ? WHERE id = ?",
                params![block.x, block.y, block.width, block.height, block.content, block.id],
            )?;
            Ok(block.id)
        }
    }
    
    pub fn delete_block(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM blocks WHERE id = ?", [id])?;
        Ok(())
    }
}
unsafe impl Send for Db {}
unsafe impl Sync for Db {}
