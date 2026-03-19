/// Database query methods for the messages table.
/// Tests use in-memory SQLite to avoid file I/O.
use chrono::Utc;
use rusqlite::{params, Result};

use super::{Database, Message};

impl Database {
    /// Check if a message with the given content hash already exists (dedup guard).
    pub fn message_exists_by_hash(&self, hash: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE content_hash = ?1",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Insert a new message record, returns the generated UUID.
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

    /// Fetch messages for a specific conversation, newest first.
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

    /// Fetch most recent messages across all conversations.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_conversation(db: &Database) -> String {
        db.upsert_conversation("TestUser", "Hello", "inbound").unwrap()
    }

    #[test]
    fn test_insert_and_exists_by_hash() {
        let db = Database::open_in_memory();
        let conv_id = seed_conversation(&db);
        db.insert_message(&conv_id, "inbound", "TestUser", "Hello", "hash_abc", "10:00").unwrap();
        assert!(db.message_exists_by_hash("hash_abc").unwrap());
        assert!(!db.message_exists_by_hash("nonexistent").unwrap());
    }

    #[test]
    fn test_get_messages_returns_correct_conversation() {
        let db = Database::open_in_memory();
        let conv_id = seed_conversation(&db);
        db.insert_message(&conv_id, "inbound", "User", "Msg 1", "h1", "10:00").unwrap();
        db.insert_message(&conv_id, "outbound", "Me", "Msg 2", "h2", "10:01").unwrap();
        let msgs = db.get_messages(&conv_id, 10).unwrap();
        assert_eq!(msgs.len(), 2);
        // newest first
        assert_eq!(msgs[0].content_hash, "h2");
    }

    #[test]
    fn test_get_recent_messages_limits_correctly() {
        let db = Database::open_in_memory();
        let conv_id = seed_conversation(&db);
        for i in 0..5u32 {
            db.insert_message(&conv_id, "inbound", "User", &format!("msg {}", i), &format!("h{}", i), "10:00").unwrap();
        }
        let msgs = db.get_recent_messages(3).unwrap();
        assert_eq!(msgs.len(), 3);
    }
}
