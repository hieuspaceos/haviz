pub mod content_safety;
pub mod health_monitor;
pub mod human_delay;
pub mod rate_limiter;
pub mod working_hours;

use content_safety::ContentSafety;
use health_monitor::HealthMonitor;
use rate_limiter::RateLimiter;

/// Outcome of a pre-send safety check.
pub enum SafetyResult {
    /// All checks passed — send immediately.
    Allow,
    /// Sending allowed but must be deferred until `send_at`.
    Queue {
        reason: String,
        send_at: chrono::DateTime<chrono::Local>,
    },
    /// Sending is forbidden. Surface `reason` to the user.
    Block { reason: String },
}

/// 5-layer safety engine that must be consulted before every automated send.
///
/// Layer order:
///   1. Rate limiting      — token-bucket sliding window
///   3. Working hours      — queue outside business hours
///   4. Content safety     — duplicate / broadcast / blocked-pattern detection
///   5. Account health     — score-based kill switch
///
/// Layer 2 (human-like delays) is NOT a gate; the caller applies those delays
/// at execution time using functions from the `human_delay` module.
pub struct SafetyEngine {
    rate_limiter: RateLimiter,
    content_safety: ContentSafety,
    health_monitor: HealthMonitor,
}

impl SafetyEngine {
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(),
            content_safety: ContentSafety::new(),
            health_monitor: HealthMonitor::new(),
        }
    }

    /// Run layers 1, 3, 4, 5 before sending a message.
    /// Returns the first non-Allow result encountered, or Allow if all pass.
    pub fn check(&mut self, conversation_id: &str, content: &str) -> SafetyResult {
        // Layer 5: account health — cheapest check, fail fast on suspension
        if let r @ SafetyResult::Block { .. } = self.health_monitor.check() {
            return r;
        }

        // Layer 1: rate limiting
        if let r @ (SafetyResult::Queue { .. } | SafetyResult::Block { .. }) =
            self.rate_limiter.check(conversation_id)
        {
            return r;
        }

        // Layer 3: working hours
        if let r @ SafetyResult::Queue { .. } = working_hours::check() {
            return r;
        }

        // Layer 4: content safety
        if let r @ SafetyResult::Block { .. } =
            self.content_safety.check(conversation_id, content)
        {
            return r;
        }

        SafetyResult::Allow
    }

    /// Must be called after every send attempt to keep internal state consistent.
    pub fn record_send_result(&mut self, success: bool, conversation_id: &str, content: &str) {
        if success {
            self.rate_limiter.record_send(conversation_id);
            self.content_safety.record_send(conversation_id, content);
            self.health_monitor.record_success();
        } else {
            self.health_monitor.record_failure();
        }
    }

    /// Expose health monitor for external event recording (replies, hidden msgs, etc.).
    pub fn health_monitor_mut(&mut self) -> &mut HealthMonitor {
        &mut self.health_monitor
    }

    /// Returns true if the engine is currently in lunch-hour mode.
    /// Callers may halve their send rate during this window.
    pub fn is_lunch_hour(&self) -> bool {
        working_hours::is_lunch_hour()
    }
}

impl Default for SafetyEngine {
    fn default() -> Self {
        Self::new()
    }
}
