---
phase: 6
title: "Safety Engine"
priority: MEDIUM
status: pending
effort: 8h
depends_on: [3]
---

# Phase 6: Safety Engine

## Context Links

- [ARCHITECTURE.md Section 10.3](../../ARCHITECTURE.md) — 5-layer safety engine spec with constants
- [ARCHITECTURE.md Section 10.4](../../ARCHITECTURE.md) — Rate limiting architecture (3 tiers)
- [ARCHITECTURE.md Section 10.5](../../ARCHITECTURE.md) — Account health monitoring
- [ARCHITECTURE.md Section 10.6](../../ARCHITECTURE.md) — Anti-broadcast protection
- [ARCHITECTURE.md Section 10.9](../../ARCHITECTURE.md) — Safety DB schema
- [agent/src/server.rs](../../agent/src/server.rs) — send_message route (no safety checks)
- [agent/src/bin/haviz_app.rs](../../agent/src/bin/haviz_app.rs) — approve_draft handler (no safety)

## Overview

ARCHITECTURE.md specifies a comprehensive 5-layer safety engine but 0% is implemented. Currently, `send_message` and `approve_draft` in server.rs send immediately without any rate limiting, working hours checks, duplicate detection, or health monitoring. This is critical for Zalo personal accounts where aggressive automation causes bans.

## Key Insights

- All 5 layers are specified in ARCHITECTURE.md Section 10.3 with exact constants
- Safety is LOCAL (Agent-side) — not cloud-side. Agent checks before every send.
- Core data structures: in-memory rate counters + SQLite for persistence across restarts
- Layer 1 (rate limiting) and Layer 3 (working hours) are the most critical for ban prevention
- Layer 4 (content safety) and Layer 5 (health monitoring) are important but less urgent
- Layer 2 (human-like delays) is partially implemented via existing random delays in AppleScript automation
- Implement as a module directory `agent/src/safety/` with clear separation per layer

## Requirements

### Functional
- **Rate limiting**: per-conversation (5/min, 30/hr) + global (60/hr, 300/day) + min 3s between messages
- **Human-like delays**: random jitter on search→click→paste→send, typing simulation 2-5s
- **Working hours**: 7-22 weekdays, 8-20 weekends, reduced speed at lunch (12-13)
- **Content safety**: anti-duplicate (5min window, 85% similarity threshold), anti-broadcast (same content >3 recipients in 30min), blocked patterns
- **Account health**: score 0-100, events adjust score, throttle/stop based on score
- **Message queue**: queue messages that fail safety checks with scheduled send time
- **Safety check** returns: `Allow`, `Queue(reason, eta)`, `Block(reason)`

### Non-Functional
- Safety check latency <5ms (in-memory lookups)
- No false positives on normal usage patterns (1-2 messages/min)
- Persists state across Agent restarts (SQLite tables)
- Clear user-facing messages explaining why a message was blocked/queued

## Architecture

### Module Structure

```
agent/src/safety/
├── mod.rs                  # SafetyEngine struct + check() entry point (~80 LOC)
├── rate_limiter.rs         # Layer 1: Token bucket rate limiting (~100 LOC)
├── human_delay.rs          # Layer 2: Random delay generation (~60 LOC)
├── working_hours.rs        # Layer 3: Time-of-day checks (~60 LOC)
├── content_safety.rs       # Layer 4: Duplicate + broadcast + blocked patterns (~120 LOC)
└── health_monitor.rs       # Layer 5: Account health scoring (~100 LOC)
```

### Safety Check Flow

```
send_message(to, content)
       │
       ▼
SafetyEngine::check(to, content)
       │
       ├── Layer 1: rate_limiter.check(conversation_id)
       │   → Allow / Queue("rate_limit", next_available_time)
       │
       ├── Layer 3: working_hours.check(now)
       │   → Allow / Queue("outside_hours", next_morning_time)
       │
       ├── Layer 4: content_safety.check(content, conversation_id)
       │   → Allow / Block("duplicate") / Block("broadcast") / Block("blocked_pattern")
       │
       ├── Layer 5: health_monitor.check(account_id)
       │   → Allow / Block("health_critical")
       │
       ▼
   SafetyResult::Allow
       │
       ├── Layer 2: human_delay.generate()
       │   → sleep(random 2-5s typing + jitter)
       │
       ▼
   Execute send
```

## Related Code Files

### Create
- `agent/src/safety/mod.rs` — SafetyEngine, SafetyResult enum, check() orchestrator
- `agent/src/safety/rate_limiter.rs` — in-memory token bucket
- `agent/src/safety/human_delay.rs` — random delay generation with jitter
- `agent/src/safety/working_hours.rs` — time-of-day validation
- `agent/src/safety/content_safety.rs` — duplicate, broadcast, blocked pattern detection
- `agent/src/safety/health_monitor.rs` — account health scoring
- SQLite migration for safety tables (if not already in Phase 3 migration)

### Modify
- `agent/src/lib.rs` — add `pub mod safety;`
- `agent/src/server.rs` — inject SafetyEngine into send_message handler
- `agent/src/bin/haviz_app.rs` — inject SafetyEngine into approve_draft handler
- `agent/src/routes/zalo_control.rs` (after Phase 3) — add safety checks to send handlers
- `agent/src/db/migrations/` — add migration 002 for safety tables (message_queue, account_health, safety_audit_log)

### Delete
- None

## Implementation Steps

### Step 1: Safety result types + engine skeleton (1h)
1. Create `agent/src/safety/mod.rs`:
   ```rust
   pub mod rate_limiter;
   pub mod human_delay;
   pub mod working_hours;
   pub mod content_safety;
   pub mod health_monitor;

   pub enum SafetyResult {
       Allow,
       Queue { reason: String, send_at: chrono::DateTime<chrono::Utc> },
       Block { reason: String, message: String },
   }

   pub struct SafetyEngine {
       rate_limiter: rate_limiter::RateLimiter,
       health: health_monitor::HealthMonitor,
       content: content_safety::ContentSafety,
   }

   impl SafetyEngine {
       pub fn new() -> Self { ... }
       pub fn check(&mut self, conversation_id: &str, content: &str) -> SafetyResult { ... }
       pub fn record_send_result(&mut self, conversation_id: &str, success: bool) { ... }
   }
   ```
2. Update `lib.rs` — add `pub mod safety;`

### Step 2: Rate limiter (1.5h)
1. Create `agent/src/safety/rate_limiter.rs`:
   - Constants from ARCHITECTURE.md 10.3:
     - `MAX_PER_MINUTE: 5`, `MAX_PER_HOUR_CONV: 30`
     - `MAX_PER_HOUR_GLOBAL: 60`, `MAX_PER_DAY_GLOBAL: 300`
     - `MIN_BETWEEN_MS: 3000`
   - `RateLimiter` struct: HashMap of conversation_id → sliding window counters
   - Sliding window: `Vec<Instant>` per conversation, prune expired entries
   - `check(conversation_id) -> SafetyResult`:
     - Check min-between-messages gap
     - Check per-conversation per-minute/per-hour limits
     - Check global per-hour/per-day limits
     - Return Queue with next available time if exceeded
   - `record(conversation_id)` — add timestamp to window

### Step 3: Working hours (1h)
1. Create `agent/src/safety/working_hours.rs`:
   - Constants: weekday 7-22, weekend 8-20, lunch 12-13 (reduced speed)
   - `check(now: DateTime<Local>) -> SafetyResult`:
     - Determine if weekday/weekend
     - Check current hour against allowed range
     - If outside → Queue with next morning time
     - If lunch → Allow but flag for reduced rate
   - `next_allowed_time(now) -> DateTime<Local>`:
     - Compute next valid send window

### Step 4: Content safety (1.5h)
1. Create `agent/src/safety/content_safety.rs`:
   - `ContentSafety` struct:
     - `recent_sends: HashMap<String, Vec<(String, Instant)>>` — content_hash → [(conversation_id, time)]
   - **Anti-duplicate**: hash current content, check if same hash sent to same conversation within 5min
   - **Anti-broadcast**: check if same hash sent to >3 different conversations within 30min
   - **Blocked patterns**: regex check against blocked pattern list
   - `check(content, conversation_id) -> SafetyResult`:
     - Hash content (SHA-256)
     - Check duplicate window
     - Check broadcast window
     - Check blocked patterns
     - Return Block with user-facing message if triggered
   - `record_send(content_hash, conversation_id)` — track send

### Step 5: Health monitor (1h)
1. Create `agent/src/safety/health_monitor.rs`:
   - `HealthMonitor` struct: `score: u32` (starts at 100), event log
   - Score adjustment events (from ARCHITECTURE.md):
     - Send success: +1
     - Receive reply: +3
     - 24h normal: +5
     - Send failed: -10
     - 3 consecutive failures: -30
     - Blocked by recipient: -15
   - `check() -> SafetyResult`:
     - score < 20 → Block("health_critical")
     - score < 50 → Block("health_warning") (manual only)
     - score < 80 → Allow (but flag for reduced rate)
     - score >= 80 → Allow
   - `record_event(event: HealthEvent)` — adjust score
   - Persist score to SQLite on changes

### Step 6: Human-like delays (30min)
1. Create `agent/src/safety/human_delay.rs`:
   - Constants from ARCHITECTURE.md 10.3:
     - `TYPING_SIMULATION: (2000, 5000)ms`
     - `BETWEEN_CONVERSATIONS: (3000, 8000)ms`
     - `JITTER_PERCENT: 0.30`
   - `generate_typing_delay() -> Duration` — random in range with jitter
   - `generate_between_delay() -> Duration` — random in range with jitter
   - `apply_jitter(base_ms: u64) -> u64` — +/-30% random variation
   - Note: these return Durations, caller uses `tokio::time::sleep()`

### Step 7: DB migration for safety tables (30min)
1. Create `agent/src/db/migrations/002_safety_tables.sql`:
   ```sql
   CREATE TABLE IF NOT EXISTS message_queue (
     id TEXT PRIMARY KEY,
     conversation_id TEXT NOT NULL,
     content TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'queued',
     reason TEXT,
     scheduled_at TEXT,
     retry_count INTEGER DEFAULT 0,
     created_at TEXT NOT NULL
   );
   CREATE TABLE IF NOT EXISTS safety_audit_log (
     id TEXT PRIMARY KEY,
     event_type TEXT NOT NULL,
     conversation_id TEXT,
     details TEXT,
     created_at TEXT NOT NULL
   );
   ```
2. Register migration in `db/migrations.rs`

### Step 8: Integrate into send flow (1h)
1. Update `server.rs` `send_message` handler:
   - Before sending: `safety_engine.check(to, message)`
   - Match on SafetyResult: Allow → proceed, Queue → store in message_queue, Block → return error
   - After sending: `safety_engine.record_send_result(to, success)`
2. Update `routes/zalo_control.rs` (post Phase 3):
   - Same safety checks for Zalo send handlers
3. Pass `SafetyEngine` as shared state (Arc<Mutex<SafetyEngine>>)

## Todo List

- [ ] Create safety/mod.rs with SafetyEngine + SafetyResult
- [ ] Implement rate_limiter.rs with sliding window
- [ ] Implement working_hours.rs with time checks
- [ ] Implement content_safety.rs with duplicate + broadcast + blocklist
- [ ] Implement health_monitor.rs with scoring
- [ ] Implement human_delay.rs with jitter generation
- [ ] Add DB migration 002 for safety tables
- [ ] Integrate safety checks into send_message handler
- [ ] Integrate safety checks into zalo_control send handlers
- [ ] Unit tests for each safety module
- [ ] Smoke test: rate limit triggers after 5 rapid sends
- [ ] Smoke test: working hours blocks outside 7-22

## Success Criteria

- Sending >5 messages/minute to same conversation → queued with "rate_limit" reason
- Sending outside 7:00-22:00 → queued with next morning send time
- Sending identical content to >3 conversations → blocked with broadcast warning
- Health score <50 → manual-only mode
- Random typing delays applied before every send (2-5s)
- All safety events logged to safety_audit_log table
- Message queue stores delayed messages with scheduled_at time

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Safety too aggressive — blocks legitimate sends | Medium | High | Start with generous limits; log but don't block initially (dry-run mode) |
| In-memory state lost on restart | Medium | Medium | Persist rate counters to SQLite; reload on startup |
| Clock manipulation bypasses working hours | Low | Low | Use system clock; no user-configurable override |
| Health score never recovers | Low | Medium | +5/24h ensures gradual recovery; expose reset in settings |

## Security Considerations

- Safety engine runs locally — cannot be bypassed by API manipulation
- Audit log provides forensic trail for account bans
- Rate limits prevent automation detection by Zalo's server-side monitoring
- Broadcast detection prevents mass messaging that violates Zalo ToS
- Health monitoring provides early warning before Zalo takes action

## Unresolved Questions

1. **Dry-run mode**: Should we implement a "dry-run" mode first that logs safety decisions without enforcing? Would help tune thresholds.
2. **User override**: Should user be able to override safety checks for urgent messages? Architecture doc shows warning popup for broadcast 4-5 recipients — implies some override capability.
3. **Cloud sync of health score**: ARCHITECTURE.md mentions syncing health to cloud `account_health` table. Defer to when cloud sync is implemented, or track locally only for now?
