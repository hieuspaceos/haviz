/// Database migrations — manages schema version via SQLite user_version.
/// All CREATE TABLE statements live here; add new migrations at the end only.
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

/// SQL for initial schema (migration 001).
const MIGRATION_001: &str = "
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
";

/// Apply all pending migrations to the database connection.
/// Must be called before wrapping conn in a Mutex.
pub fn run_migrations(conn: &mut Connection) -> rusqlite_migration::Result<()> {
    let migrations = Migrations::new(vec![M::up(MIGRATION_001)]);
    migrations.to_latest(conn)
}
