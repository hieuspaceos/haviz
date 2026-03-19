/// Database query methods for the ai_drafts table.
use chrono::Utc;
use rusqlite::{params, Result};

use super::{AiDraft, Database};

impl Database {
    /// Insert a new AI draft and return its UUID.
    pub fn insert_draft(
        &self,
        conversation_id: &str,
        trigger_message_id: Option<&str>,
        content: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ai_drafts (id, conversation_id, trigger_message_id, content, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, conversation_id, trigger_message_id, content, now],
        )?;
        Ok(id)
    }

    /// Fetch all drafts with status = 'pending', newest first.
    pub fn get_pending_drafts(&self) -> Result<Vec<AiDraft>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, trigger_message_id, content, status, created_at \
             FROM ai_drafts WHERE status = 'pending' ORDER BY created_at DESC",
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

    /// Update the status field of a draft (e.g. 'approved', 'rejected').
    pub fn update_draft_status(&self, draft_id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE ai_drafts SET status = ?1 WHERE id = ?2",
            params![status, draft_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_conversation(db: &Database) -> String {
        db.upsert_conversation("TestUser", "Hi", "inbound").unwrap()
    }

    #[test]
    fn test_insert_draft_and_get_pending() {
        let db = Database::open_in_memory();
        let conv_id = seed_conversation(&db);
        let draft_id = db.insert_draft(&conv_id, None, "Draft reply content").unwrap();
        assert!(!draft_id.is_empty());
        let pending = db.get_pending_drafts().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].content, "Draft reply content");
        assert_eq!(pending[0].status, "pending");
    }

    #[test]
    fn test_update_draft_status() {
        let db = Database::open_in_memory();
        let conv_id = seed_conversation(&db);
        let draft_id = db.insert_draft(&conv_id, None, "Some draft").unwrap();
        db.update_draft_status(&draft_id, "approved").unwrap();
        // After approval, should not appear in pending
        let pending = db.get_pending_drafts().unwrap();
        assert!(pending.is_empty());
    }
}
