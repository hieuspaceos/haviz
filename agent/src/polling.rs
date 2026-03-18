use crate::channels::traits::ChannelReader;
use crate::db::Database;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

pub struct Poller {
    db: Arc<Database>,
    channel: Box<dyn ChannelReader + Send>,
    known_hashes: HashSet<String>,
    poll_interval: Duration,
}

impl Poller {
    pub fn new(
        db: Arc<Database>,
        channel: Box<dyn ChannelReader + Send>,
        poll_interval_secs: u64,
    ) -> Self {
        // Load existing hashes from DB to avoid reprocessing on restart
        let known_hashes = Self::load_known_hashes(&db);
        tracing::info!("Loaded {} known message hashes from DB", known_hashes.len());

        Self {
            db,
            channel,
            known_hashes,
            poll_interval: Duration::from_secs(poll_interval_secs),
        }
    }

    fn load_known_hashes(db: &Database) -> HashSet<String> {
        match db.get_recent_messages(5000) {
            Ok(msgs) => msgs.into_iter().map(|m| m.content_hash).collect(),
            Err(e) => {
                tracing::warn!("Failed to load known hashes: {}", e);
                HashSet::new()
            }
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(self.poll_interval);
        loop {
            interval.tick().await;
            if let Err(e) = self.poll_once() {
                if e == "zalo_not_running" {
                    // Silent — Zalo not open is normal
                } else {
                    tracing::error!("Poll error: {}", e);
                }
            }
        }
    }

    fn poll_once(&mut self) -> Result<(), String> {
        let messages = self.channel.read_messages()?;

        let mut new_count = 0;
        for msg in messages {
            if self.known_hashes.contains(&msg.content_hash) {
                continue;
            }

            // Double-check DB (in case hash set got out of sync)
            match self.db.message_exists_by_hash(&msg.content_hash) {
                Ok(true) => {
                    self.known_hashes.insert(msg.content_hash.clone());
                    continue;
                }
                Ok(false) => {}
                Err(e) => {
                    tracing::error!("DB check error: {}", e);
                    continue;
                }
            }

            // New message — store it
            let preview = if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            };

            let conv_id = match self.db.upsert_conversation(&msg.sender, &preview, &msg.direction) {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Failed to upsert conversation: {}", e);
                    continue;
                }
            };

            match self.db.insert_message(
                &conv_id,
                &msg.direction,
                &msg.sender,
                &msg.content,
                &msg.content_hash,
                &msg.timestamp,
            ) {
                Ok(_id) => {
                    self.known_hashes.insert(msg.content_hash.clone());
                    new_count += 1;
                    tracing::info!(
                        "[{}] {} ({}): {}",
                        msg.direction,
                        msg.sender,
                        msg.timestamp,
                        if msg.content.len() > 80 {
                            format!("{}...", &msg.content[..80])
                        } else {
                            msg.content.clone()
                        }
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to insert message: {}", e);
                }
            }
        }

        if new_count > 0 {
            tracing::info!("Stored {} new messages", new_count);
        }

        Ok(())
    }
}
