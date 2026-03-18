use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

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

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                contact_name TEXT NOT NULL,
                channel_type TEXT NOT NULL DEFAULT 'zalo_desktop',
                last_message_at TEXT,
                last_message_preview TEXT,
                unread_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                direction TEXT NOT NULL,
                sender_name TEXT NOT NULL,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'received',
                zalo_timestamp TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE IF NOT EXISTS ai_drafts (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                trigger_message_id TEXT,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                category TEXT,
                match_patterns TEXT NOT NULL DEFAULT '[]',
                usage_count INTEGER DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_messages_hash ON messages(content_hash);
            CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_drafts_conv ON ai_drafts(conversation_id, created_at);
            ",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn message_exists_by_hash(&self, hash: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE content_hash = ?1",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn insert_message(
        &self,
        conversation_id: &str,
        direction: &str,
        sender_name: &str,
        content: &str,
        content_hash: &str,
        zalo_timestamp: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (id, conversation_id, direction, sender_name, content, content_hash, zalo_timestamp, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, conversation_id, direction, sender_name, content, content_hash, zalo_timestamp, now],
        )?;
        Ok(id)
    }

    pub fn upsert_conversation(&self, contact_name: &str, last_preview: &str, direction: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();

        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM conversations WHERE contact_name = ?1 AND channel_type = 'zalo_desktop'",
                params![contact_name],
                |row| row.get(0),
            )
            .ok();

        if let Some(conv_id) = existing {
            let unread_delta = if direction == "inbound" { 1 } else { 0 };
            conn.execute(
                "UPDATE conversations SET last_message_at = ?1, last_message_preview = ?2, unread_count = unread_count + ?3, updated_at = ?1 WHERE id = ?4",
                params![now, last_preview, unread_delta, conv_id],
            )?;
            Ok(conv_id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let unread = if direction == "inbound" { 1 } else { 0 };
            conn.execute(
                "INSERT INTO conversations (id, contact_name, last_message_at, last_message_preview, unread_count, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?3, ?3)",
                params![id, contact_name, now, last_preview, unread],
            )?;
            Ok(id)
        }
    }

    pub fn get_conversations(&self, limit: u32) -> Result<Vec<Conversation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, contact_name, channel_type, last_message_at, last_message_preview, unread_count, created_at, updated_at
             FROM conversations ORDER BY last_message_at DESC NULLS LAST LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                contact_name: row.get(1)?,
                channel_type: row.get(2)?,
                last_message_at: row.get(3)?,
                last_message_preview: row.get(4)?,
                unread_count: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_messages(&self, conversation_id: &str, limit: u32) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, direction, sender_name, content, content_hash, status, zalo_timestamp, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![conversation_id, limit], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                direction: row.get(2)?,
                sender_name: row.get(3)?,
                content: row.get(4)?,
                content_hash: row.get(5)?,
                status: row.get(6)?,
                zalo_timestamp: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_recent_messages(&self, limit: u32) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, direction, sender_name, content, content_hash, status, zalo_timestamp, created_at
             FROM messages ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                direction: row.get(2)?,
                sender_name: row.get(3)?,
                content: row.get(4)?,
                content_hash: row.get(5)?,
                status: row.get(6)?,
                zalo_timestamp: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn mark_conversation_read(&self, conversation_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE conversations SET unread_count = 0 WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }

    // AI Drafts
    pub fn insert_draft(&self, conversation_id: &str, trigger_message_id: Option<&str>, content: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ai_drafts (id, conversation_id, trigger_message_id, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, conversation_id, trigger_message_id, content, now],
        )?;
        Ok(id)
    }

    pub fn get_pending_drafts(&self) -> Result<Vec<AiDraft>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, trigger_message_id, content, status, created_at FROM ai_drafts WHERE status = 'pending' ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AiDraft {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                trigger_message_id: row.get(2)?,
                content: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_draft_status(&self, draft_id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE ai_drafts SET status = ?1 WHERE id = ?2",
            params![status, draft_id],
        )?;
        Ok(())
    }

    // Templates
    pub fn get_templates(&self) -> Result<Vec<Template>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, content, category, match_patterns, usage_count FROM templates ORDER BY usage_count DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let patterns_str: String = row.get(4)?;
            let patterns: Vec<String> = serde_json::from_str(&patterns_str).unwrap_or_default();
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                category: row.get(3)?,
                match_patterns: patterns,
                usage_count: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn insert_template(&self, name: &str, content: &str, category: Option<&str>, patterns: &[String]) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let patterns_json = serde_json::to_string(patterns).unwrap_or_else(|_| "[]".to_string());
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO templates (id, name, content, category, match_patterns) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, content, category, patterns_json],
        )?;
        Ok(id)
    }

    pub fn increment_template_usage(&self, template_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE templates SET usage_count = usage_count + 1 WHERE id = ?1",
            params![template_id],
        )?;
        Ok(())
    }
}
