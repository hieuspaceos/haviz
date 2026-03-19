use std::collections::HashMap;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use crate::safety::SafetyResult;

// Duplicate detection: block same content to same conversation within 5 minutes
const DUPLICATE_WINDOW_SECS: u64 = 300;

// Broadcast detection: same content hash sent to more than 3 conversations in 30 minutes
const BROADCAST_DETECTION_WINDOW_MINS: u64 = 30;
const BROADCAST_SAME_CONTENT_LIMIT: usize = 3;

/// Patterns that are unconditionally blocked (spam/scam/phishing signals)
const BLOCKED_PATTERNS: &[&str] = &[
    "bit.ly/",
    "click vào link",
    "chuyển khoản ngay",
];

/// (conversation_id, content_hash, sent_at)
type RecentEntry = (String, String, Instant);

/// Tracks recent messages for duplicate and broadcast detection.
pub struct ContentSafety {
    /// Recent sends: (conversation_id, content_hash, timestamp)
    recent_messages: Vec<RecentEntry>,
    /// content_hash → list of conversation_ids that received it (within window)
    broadcast_tracker: HashMap<String, Vec<String>>,
}

impl ContentSafety {
    pub fn new() -> Self {
        Self {
            recent_messages: Vec::new(),
            broadcast_tracker: HashMap::new(),
        }
    }

    /// Run all content safety checks. Returns Allow, Block, or Queue.
    pub fn check(&mut self, conversation_id: &str, content: &str) -> SafetyResult {
        let now = Instant::now();
        self.clean_old_entries(now);

        // Layer: blocked patterns (unconditional)
        if let Some(pattern) = check_blocked_patterns(content) {
            return SafetyResult::Block {
                reason: format!("Content contains blocked pattern: \"{pattern}\""),
            };
        }

        let hash = compute_content_hash(content);

        // Layer: duplicate detection — same hash to same conversation in last 5 min
        let is_duplicate = self.recent_messages.iter().any(|(conv, h, ts)| {
            conv == conversation_id
                && h == &hash
                && now.duration_since(*ts) < Duration::from_secs(DUPLICATE_WINDOW_SECS)
        });
        if is_duplicate {
            return SafetyResult::Block {
                reason: format!(
                    "Duplicate message detected: same content sent to this conversation within {DUPLICATE_WINDOW_SECS}s"
                ),
            };
        }

        // Layer: broadcast detection — same hash to >3 different conversations in 30 min
        let recipients = self.broadcast_tracker.entry(hash.clone()).or_default();
        let unique_recent: Vec<&String> = recipients
            .iter()
            .filter(|c| c.as_str() != conversation_id)
            .collect();
        if unique_recent.len() >= BROADCAST_SAME_CONTENT_LIMIT {
            return SafetyResult::Block {
                reason: format!(
                    "Broadcast detected: identical content already sent to {} conversations in the last {BROADCAST_DETECTION_WINDOW_MINS} minutes",
                    unique_recent.len()
                ),
            };
        }

        SafetyResult::Allow
    }

    /// Call after a message is successfully sent to update tracking state.
    pub fn record_send(&mut self, conversation_id: &str, content: &str) {
        let hash = compute_content_hash(content);
        self.recent_messages
            .push((conversation_id.to_string(), hash.clone(), Instant::now()));
        self.broadcast_tracker
            .entry(hash)
            .or_default()
            .push(conversation_id.to_string());
    }

    /// Evict entries outside the broadcast detection window to bound memory.
    fn clean_old_entries(&mut self, now: Instant) {
        let cutoff = Duration::from_secs(BROADCAST_DETECTION_WINDOW_MINS * 60);
        self.recent_messages
            .retain(|(_, _, ts)| now.duration_since(*ts) < cutoff);

        // Rebuild broadcast_tracker from surviving recent_messages
        self.broadcast_tracker.clear();
        for (conv, hash, _) in &self.recent_messages {
            self.broadcast_tracker
                .entry(hash.clone())
                .or_default()
                .push(conv.clone());
        }
    }
}

/// SHA-256 hash of normalised (lowercase, trimmed) content.
fn compute_content_hash(content: &str) -> String {
    let normalised = content.trim().to_lowercase();
    let digest = Sha256::digest(normalised.as_bytes());
    hex::encode(digest)
}

/// Returns the first matching blocked pattern found in content, or None.
fn check_blocked_patterns(content: &str) -> Option<&'static str> {
    let lower = content.to_lowercase();
    BLOCKED_PATTERNS
        .iter()
        .copied()
        .find(|&pat| lower.contains(pat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_pattern_bit_ly() {
        let mut cs = ContentSafety::new();
        let result = cs.check("conv1", "Xem tại bit.ly/abc123");
        assert!(matches!(result, SafetyResult::Block { .. }));
    }

    #[test]
    fn test_blocked_pattern_scam() {
        let mut cs = ContentSafety::new();
        let result = cs.check("conv1", "Chuyển khoản ngay để nhận quà");
        assert!(matches!(result, SafetyResult::Block { .. }));
    }

    #[test]
    fn test_duplicate_detection() {
        let mut cs = ContentSafety::new();
        cs.record_send("conv1", "Hello");
        let result = cs.check("conv1", "Hello");
        assert!(matches!(result, SafetyResult::Block { .. }));
    }

    #[test]
    fn test_broadcast_detection() {
        let mut cs = ContentSafety::new();
        for i in 0..BROADCAST_SAME_CONTENT_LIMIT {
            cs.record_send(&format!("conv{i}"), "Same message");
        }
        // Next send of same content to a new conversation should be blocked
        let result = cs.check("conv_new", "Same message");
        assert!(matches!(result, SafetyResult::Block { .. }));
    }

    #[test]
    fn test_clean_content_is_allowed() {
        let mut cs = ContentSafety::new();
        let result = cs.check("conv1", "Chào anh, em gửi báo giá theo yêu cầu.");
        assert!(matches!(result, SafetyResult::Allow));
    }
}
