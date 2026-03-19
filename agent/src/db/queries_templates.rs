/// Database query methods for the templates table.
use rusqlite::{params, Result};

use super::{Database, Template};

impl Database {
    /// Fetch all templates ordered by usage_count descending.
    pub fn get_templates(&self) -> Result<Vec<Template>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, content, category, match_patterns, usage_count \
             FROM templates ORDER BY usage_count DESC",
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

    /// Insert a new template and return its UUID.
    pub fn insert_template(
        &self,
        name: &str,
        content: &str,
        category: Option<&str>,
        patterns: &[String],
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let patterns_json = serde_json::to_string(patterns).unwrap_or_else(|_| "[]".to_string());
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO templates (id, name, content, category, match_patterns) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, content, category, patterns_json],
        )?;
        Ok(id)
    }

    /// Increment usage_count for a template by 1.
    pub fn increment_template_usage(&self, template_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE templates SET usage_count = usage_count + 1 WHERE id = ?1",
            params![template_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_templates() {
        let db = Database::open_in_memory();
        let patterns = vec!["xin chào".to_string(), "hello".to_string()];
        let id = db.insert_template("Greeting", "Xin chào quý khách!", Some("greeting"), &patterns).unwrap();
        assert!(!id.is_empty());
        let templates = db.get_templates().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Greeting");
        assert_eq!(templates[0].match_patterns.len(), 2);
        assert_eq!(templates[0].usage_count, 0);
    }

    #[test]
    fn test_increment_template_usage() {
        let db = Database::open_in_memory();
        let id = db.insert_template("Price", "Giá sản phẩm là...", None, &[]).unwrap();
        db.increment_template_usage(&id).unwrap();
        db.increment_template_usage(&id).unwrap();
        let templates = db.get_templates().unwrap();
        assert_eq!(templates[0].usage_count, 2);
    }
}
