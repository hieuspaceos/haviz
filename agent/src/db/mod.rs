/// Database module — SQLite-backed persistence via rusqlite.
/// Struct definitions and open() live here; query methods are split by entity.
use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub(crate) mod migrations;
mod queries_conversations;
mod queries_drafts;
mod queries_messages;
mod queries_templates;

// ── Data structs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub direction: String,
    pub sender_name: String,
    pub content: String,
    pub content_hash: String,
    pub status: String,
    pub zalo_timestamp: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub id: String,
    pub contact_name: String,
    pub channel_type: String,
    pub last_message_at: Option<String>,
    pub last_message_preview: Option<String>,
    pub unread_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiDraft {
    pub id: String,
    pub conversation_id: String,
    pub trigger_message_id: Option<String>,
    pub content: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub content: String,
    pub category: Option<String>,
    pub match_patterns: Vec<String>,
    pub usage_count: i32,
}

// ── Database handle ──────────────────────────────────────────────────────────

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Open the SQLite database at `path`, apply all pending migrations, and
    /// configure WAL + foreign keys.
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // Run schema migrations — panic on failure (unrecoverable startup error)
        migrations::run_migrations(&mut conn)
            .expect("DB migration failed");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory database with full schema applied — for tests only.
    #[cfg(test)]
    pub(crate) fn open_in_memory() -> Self {
        let mut conn = Connection::open_in_memory().expect("in-memory DB failed");
        migrations::run_migrations(&mut conn).expect("migration failed");
        Self { conn: Mutex::new(conn) }
    }
}
