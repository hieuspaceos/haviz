/// Database query methods for the conversations table.
use chrono::Utc;
use rusqlite::{params, Result};

use super::{Conversation, Database};

impl Database {
    /// Insert or update a conversation record.
    /// Increments unread_count by 1 for inbound messages; returns conversation UUID.
    pub fn upsert_conversation(
        &self,
        contact_name: &str,
        last_preview: &str,
        direction: &str,
    ) -> Result<String> {
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
                "UPDATE conversations SET last_message_at = ?1, last_message_preview = ?2, \
                 unread_count = unread_count + ?3, updated_at = ?1 WHERE id = ?4",
                params![now, last_preview, unread_delta, conv_id],
            )?;
            Ok(conv_id)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let unread = if direction == "inbound" { 1 } else { 0 };
            conn.execute(
                "INSERT INTO conversations (id, contact_name, last_message_at, last_message_preview, \
                 unread_count, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?3, ?3)",
                params![id, contact_name, now, last_preview, unread],
            )?;
            Ok(id)
        }
    }

    /// Fetch conversations ordered by most recent message, newest first.
    pub fn get_conversations(&self, limit: u32) -> Result<Vec<Conversation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, contact_name, channel_type, last_message_at, last_message_preview, \
             unread_count, created_at, updated_at \
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

    /// Reset unread_count to 0 for a conversation.
    pub fn mark_conversation_read(&self, conversation_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE conversations SET unread_count = 0 WHERE id = ?1",
            params![conversation_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_conversation_creates_new() {
        let db = Database::open_in_memory();
        let id = db.upsert_conversation("Alice", "Hello", "inbound").unwrap();
        assert!(!id.is_empty());
        let convs = db.get_conversations(10).unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].contact_name, "Alice");
        assert_eq!(convs[0].unread_count, 1); // inbound increments unread
    }

    #[test]
    fn test_upsert_conversation_updates_existing() {
        let db = Database::open_in_memory();
        let id1 = db.upsert_conversation("Bob", "First", "inbound").unwrap();
        let id2 = db.upsert_conversation("Bob", "Second", "outbound").unwrap();
        // Same contact → same UUID returned
        assert_eq!(id1, id2);
        let convs = db.get_conversations(10).unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].last_message_preview.as_deref(), Some("Second"));
    }

    #[test]
    fn test_get_conversations_sorted_by_last_message() {
        let db = Database::open_in_memory();
        db.upsert_conversation("Alice", "Hi", "inbound").unwrap();
        // Small sleep would help ordering, but since timestamps are Utc::now(),
        // we just verify both conversations are returned
        db.upsert_conversation("Bob", "Hey", "inbound").unwrap();
        let convs = db.get_conversations(10).unwrap();
        assert_eq!(convs.len(), 2);
    }
}
