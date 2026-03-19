use std::time::Duration;

// Delay ranges (milliseconds) between automation steps — simulates human timing
const SEARCH_TO_CLICK_DELAY: (u64, u64) = (800, 2000);
const CLICK_TO_PASTE_DELAY: (u64, u64) = (500, 1200);
const PASTE_TO_SEND_DELAY: (u64, u64) = (300, 800);
const TYPING_SIMULATION_MS: (u64, u64) = (2000, 5000);
const BETWEEN_CONVERSATIONS_MS: (u64, u64) = (3000, 8000);

/// Variance: ±30% jitter applied to every delay to avoid pattern detection
const JITTER_PERCENT: f64 = 0.30;

/// Generate a random u64 in [0, range) using system time as a simple seed.
/// Avoids pulling the `rand` crate for this lightweight use case.
fn pseudo_rand(range: u64) -> u64 {
    if range == 0 {
        return 0;
    }
    // Mix bits from current nanosecond timestamp (good enough for jitter)
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    // Xorshift-like mix
    let mixed = nanos ^ (nanos << 13) ^ (nanos >> 7);
    mixed % range
}

/// Generate a random duration in [min_ms, max_ms] with ±JITTER_PERCENT variance.
pub fn random_delay(min_ms: u64, max_ms: u64) -> Duration {
    let base = min_ms + pseudo_rand(max_ms.saturating_sub(min_ms) + 1);
    // Jitter: ±30% of base
    let jitter_range = (base as f64 * JITTER_PERCENT) as u64;
    let jitter = if jitter_range > 0 {
        pseudo_rand(jitter_range * 2 + 1)
    } else {
        0
    };
    let final_ms = base.saturating_add(jitter).saturating_sub(jitter_range);
    Duration::from_millis(final_ms)
}

/// Delay between opening Zalo search and clicking the conversation.
pub fn search_delay() -> Duration {
    random_delay(SEARCH_TO_CLICK_DELAY.0, SEARCH_TO_CLICK_DELAY.1)
}

/// Delay between clicking conversation and pasting the message.
pub fn click_delay() -> Duration {
    random_delay(CLICK_TO_PASTE_DELAY.0, CLICK_TO_PASTE_DELAY.1)
}

/// Delay between pasting and pressing Send.
pub fn paste_delay() -> Duration {
    random_delay(PASTE_TO_SEND_DELAY.0, PASTE_TO_SEND_DELAY.1)
}

/// Simulates "typing indicator" visible time before sending.
pub fn typing_delay() -> Duration {
    random_delay(TYPING_SIMULATION_MS.0, TYPING_SIMULATION_MS.1)
}

/// Delay when switching between conversations to avoid rapid-fire pattern.
pub fn conversation_switch_delay() -> Duration {
    random_delay(BETWEEN_CONVERSATIONS_MS.0, BETWEEN_CONVERSATIONS_MS.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_delay_within_range() {
        for _ in 0..20 {
            let d = random_delay(500, 1000);
            // With ±30% jitter the result can be slightly outside the nominal
            // range, but must remain positive and under an expanded ceiling.
            assert!(d.as_millis() > 0);
            assert!(d.as_millis() <= 1500); // 1000 + 30% headroom
        }
    }

    #[test]
    fn test_convenience_fns_return_positive_duration() {
        assert!(search_delay().as_millis() > 0);
        assert!(click_delay().as_millis() > 0);
        assert!(paste_delay().as_millis() > 0);
        assert!(typing_delay().as_millis() > 0);
        assert!(conversation_switch_delay().as_millis() > 0);
    }
}
