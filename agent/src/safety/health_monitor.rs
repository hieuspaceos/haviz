use std::time::Instant;

use crate::safety::SafetyResult;

// Score thresholds
const SCORE_START: i32 = 100;
const THRESHOLD_WARNING: i32 = 80;   // < 80: reduce speed 50%
const THRESHOLD_CRITICAL: i32 = 50;  // < 50: manual send only
const THRESHOLD_SUSPENDED: i32 = 20; // < 20: stop completely

// Score deltas (from ARCHITECTURE.md §10.5)
const DELTA_SEND_SUCCESS: i32 = 1;
const DELTA_REPLY_RECEIVED: i32 = 3;
const DELTA_NORMAL_24H: i32 = 5;
const DELTA_SEND_FAILED: i32 = -10;
const DELTA_THREE_CONSECUTIVE_FAILURES: i32 = -30;
const DELTA_MESSAGE_HIDDEN: i32 = -20;
const DELTA_NO_NEW_MESSAGES: i32 = -5;
const DELTA_BLOCKED_BY_RECIPIENT: i32 = -15;

// Trigger for consecutive-failure penalty
const CONSECUTIVE_FAILURE_TRIGGER: u32 = 3;

/// Operational status derived from the health score.
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Score >= 80 — full speed
    Healthy,
    /// Score 50–79 — operate at 50% speed
    Warning,
    /// Score 20–49 — manual sends only
    Critical,
    /// Score < 20 — all automated sending halted
    Suspended,
}

/// Tracks account health via a score that rises on success and falls on failures.
pub struct HealthMonitor {
    score: i32,
    consecutive_failures: u32,
    last_success: Option<Instant>,
    status: HealthStatus,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            score: SCORE_START,
            consecutive_failures: 0,
            last_success: None,
            status: HealthStatus::Healthy,
        }
    }

    /// Layer 5 safety check. Returns Block when score is too low to auto-send.
    pub fn check(&self) -> SafetyResult {
        match self.status {
            HealthStatus::Suspended => SafetyResult::Block {
                reason: format!(
                    "Account suspended (health score {}/100). Check Zalo manually.",
                    self.score
                ),
            },
            HealthStatus::Critical => SafetyResult::Block {
                reason: format!(
                    "Health score critical ({}/100). Manual sends only.",
                    self.score
                ),
            },
            // Warning and Healthy both allow automated sends;
            // caller should throttle to 50% when Warning.
            HealthStatus::Warning | HealthStatus::Healthy => SafetyResult::Allow,
        }
    }

    /// Record a successful send (+1) and reset consecutive-failure counter.
    pub fn record_success(&mut self) {
        self.apply_delta(DELTA_SEND_SUCCESS);
        self.consecutive_failures = 0;
        self.last_success = Some(Instant::now());
    }

    /// Record a failed send (−10). Applies extra −30 on 3rd consecutive failure.
    pub fn record_failure(&mut self) {
        self.apply_delta(DELTA_SEND_FAILED);
        self.consecutive_failures += 1;
        if self.consecutive_failures >= CONSECUTIVE_FAILURE_TRIGGER {
            self.apply_delta(DELTA_THREE_CONSECUTIVE_FAILURES);
        }
    }

    /// Record that a reply was received from a recipient (+3).
    pub fn record_reply_received(&mut self) {
        self.apply_delta(DELTA_REPLY_RECEIVED);
    }

    /// Record 24 h of normal activity (+5).
    pub fn record_normal_24h(&mut self) {
        self.apply_delta(DELTA_NORMAL_24H);
    }

    /// Record that a sent message was hidden/deleted by Zalo (−20).
    pub fn record_message_hidden(&mut self) {
        self.apply_delta(DELTA_MESSAGE_HIDDEN);
    }

    /// Record that no new messages arrived for >30 min (−5).
    pub fn record_no_new_messages(&mut self) {
        self.apply_delta(DELTA_NO_NEW_MESSAGES);
    }

    /// Record that the agent was blocked by a recipient (−15).
    pub fn record_blocked_by_recipient(&mut self) {
        self.apply_delta(DELTA_BLOCKED_BY_RECIPIENT);
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_status(&self) -> &HealthStatus {
        &self.status
    }

    /// Apply a score delta, clamp to [0, 100], then refresh status.
    fn apply_delta(&mut self, delta: i32) {
        self.score = (self.score + delta).clamp(0, SCORE_START);
        self.status = match self.score {
            s if s < THRESHOLD_SUSPENDED => HealthStatus::Suspended,
            s if s < THRESHOLD_CRITICAL => HealthStatus::Critical,
            s if s < THRESHOLD_WARNING => HealthStatus::Warning,
            _ => HealthStatus::Healthy,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_healthy() {
        let hm = HealthMonitor::new();
        assert_eq!(hm.get_score(), 100);
        assert_eq!(*hm.get_status(), HealthStatus::Healthy);
        assert!(matches!(hm.check(), SafetyResult::Allow));
    }

    #[test]
    fn test_success_increments_score() {
        let mut hm = HealthMonitor::new();
        // Score starts at 100 (capped), success keeps it at 100
        hm.record_success();
        assert_eq!(hm.get_score(), 100);
    }

    #[test]
    fn test_three_consecutive_failures_trigger_penalty() {
        let mut hm = HealthMonitor::new();
        // 3 failures: 3×(−10) + 1×(−30) = −60 → score = 40 → Critical
        hm.record_failure();
        hm.record_failure();
        hm.record_failure();
        assert!(hm.get_score() <= THRESHOLD_CRITICAL);
        assert!(matches!(hm.check(), SafetyResult::Block { .. }));
    }

    #[test]
    fn test_suspended_below_threshold() {
        let mut hm = HealthMonitor::new();
        // Drive score below 20
        for _ in 0..10 {
            hm.apply_delta(-15);
        }
        assert_eq!(*hm.get_status(), HealthStatus::Suspended);
        assert!(matches!(hm.check(), SafetyResult::Block { .. }));
    }

    #[test]
    fn test_warning_status_allows_send() {
        let mut hm = HealthMonitor::new();
        // Score = 70 → Warning
        hm.apply_delta(-30);
        assert_eq!(*hm.get_status(), HealthStatus::Warning);
        assert!(matches!(hm.check(), SafetyResult::Allow));
    }

    #[test]
    fn test_score_clamps_at_zero() {
        let mut hm = HealthMonitor::new();
        hm.apply_delta(-9999);
        assert_eq!(hm.get_score(), 0);
    }
}
