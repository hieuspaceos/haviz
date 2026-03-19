use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::safety::SafetyResult;

// Per-conversation limits
const MAX_MESSAGES_PER_MINUTE: usize = 5;
const MAX_MESSAGES_PER_HOUR_PER_CONVERSATION: usize = 30;

// Global limits
const MAX_MESSAGES_PER_HOUR_GLOBAL: usize = 60;
const MAX_MESSAGES_PER_DAY_GLOBAL: usize = 300;

// Minimum interval between messages to the same conversation
const MIN_BETWEEN_MESSAGES_MS: u64 = 3000;

/// In-memory rate limiter using sliding window (timestamp queues).
pub struct RateLimiter {
    /// Per-conversation send timestamps
    conv_timestamps: HashMap<String, VecDeque<Instant>>,
    /// Global send timestamps (all conversations)
    global_timestamps: VecDeque<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            conv_timestamps: HashMap::new(),
            global_timestamps: VecDeque::new(),
        }
    }

    /// Check if sending is allowed. Returns Allow or Queue with next available time.
    pub fn check(&mut self, conversation_id: &str) -> SafetyResult {
        let now = Instant::now();
        self.clean_old_entries(now);

        let conv_queue = self.conv_timestamps.entry(conversation_id.to_string()).or_default();

        // Check minimum interval since last message to this conversation
        if let Some(&last) = conv_queue.back() {
            let elapsed = now.duration_since(last);
            if elapsed < Duration::from_millis(MIN_BETWEEN_MESSAGES_MS) {
                let wait_ms = MIN_BETWEEN_MESSAGES_MS - elapsed.as_millis() as u64;
                let send_at = chrono::Local::now() + chrono::Duration::milliseconds(wait_ms as i64);
                return SafetyResult::Queue {
                    reason: "Minimum interval between messages not elapsed".to_string(),
                    send_at,
                };
            }
        }

        // Check per-conversation per-minute limit
        let per_minute_count = conv_queue.iter()
            .filter(|&&t| now.duration_since(t) < Duration::from_secs(60))
            .count();
        if per_minute_count >= MAX_MESSAGES_PER_MINUTE {
            let send_at = chrono::Local::now() + chrono::Duration::seconds(60);
            return SafetyResult::Queue {
                reason: format!("Per-conversation per-minute limit ({MAX_MESSAGES_PER_MINUTE}) reached"),
                send_at,
            };
        }

        // Check per-conversation per-hour limit
        let per_hour_conv_count = conv_queue.iter()
            .filter(|&&t| now.duration_since(t) < Duration::from_secs(3600))
            .count();
        if per_hour_conv_count >= MAX_MESSAGES_PER_HOUR_PER_CONVERSATION {
            let send_at = chrono::Local::now() + chrono::Duration::seconds(3600);
            return SafetyResult::Queue {
                reason: format!("Per-conversation per-hour limit ({MAX_MESSAGES_PER_HOUR_PER_CONVERSATION}) reached"),
                send_at,
            };
        }

        // Check global per-hour limit
        let global_hour_count = self.global_timestamps.iter()
            .filter(|&&t| now.duration_since(t) < Duration::from_secs(3600))
            .count();
        if global_hour_count >= MAX_MESSAGES_PER_HOUR_GLOBAL {
            let send_at = chrono::Local::now() + chrono::Duration::seconds(3600);
            return SafetyResult::Queue {
                reason: format!("Global per-hour limit ({MAX_MESSAGES_PER_HOUR_GLOBAL}) reached"),
                send_at,
            };
        }

        // Check global per-day limit
        if self.global_timestamps.len() >= MAX_MESSAGES_PER_DAY_GLOBAL {
            let send_at = chrono::Local::now() + chrono::Duration::hours(24);
            return SafetyResult::Queue {
                reason: format!("Global per-day limit ({MAX_MESSAGES_PER_DAY_GLOBAL}) reached"),
                send_at,
            };
        }

        SafetyResult::Allow
    }

    /// Record a successful send for rate tracking.
    pub fn record_send(&mut self, conversation_id: &str) {
        let now = Instant::now();
        self.conv_timestamps
            .entry(conversation_id.to_string())
            .or_default()
            .push_back(now);
        self.global_timestamps.push_back(now);
    }

    /// Remove entries older than 24 hours to bound memory usage.
    fn clean_old_entries(&mut self, now: Instant) {
        let cutoff = Duration::from_secs(86400);
        for queue in self.conv_timestamps.values_mut() {
            while let Some(&front) = queue.front() {
                if now.duration_since(front) > cutoff {
                    queue.pop_front();
                } else {
                    break;
                }
            }
        }
        while let Some(&front) = self.global_timestamps.front() {
            if now.duration_since(front) > cutoff {
                self.global_timestamps.pop_front();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allows_first_message() {
        let mut rl = RateLimiter::new();
        assert!(matches!(rl.check("conv1"), SafetyResult::Allow));
    }

    #[test]
    fn test_queues_within_min_interval() {
        let mut rl = RateLimiter::new();
        rl.record_send("conv1");
        let result = rl.check("conv1");
        assert!(matches!(result, SafetyResult::Queue { .. }));
    }

    #[test]
    fn test_global_day_limit() {
        let mut rl = RateLimiter::new();
        // Fill up to the day limit
        for _ in 0..MAX_MESSAGES_PER_DAY_GLOBAL {
            rl.global_timestamps.push_back(Instant::now());
        }
        let result = rl.check("conv_new");
        assert!(matches!(result, SafetyResult::Queue { .. }));
    }
}
