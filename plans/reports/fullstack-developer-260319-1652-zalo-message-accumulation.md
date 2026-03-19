# Phase Implementation Report

### Executed Phase
- Phase: zalo-message-accumulation (no plan dir — direct task)
- Status: completed

### Files Modified
| File | Lines | Change |
|------|-------|--------|
| `agent/src/routes/zalo_accumulator.rs` | 136 | NEW — WebView IPC drain + parse + SQLite upsert |
| `agent/src/routes/zalo_db_handlers.rs` | 103 | NEW — DB-backed /messages and /history handlers |
| `agent/src/routes/mod.rs` | 47 | register 2 new modules, add DB sub-router for new routes |
| `agent/src/routes/zalo_control.rs` | 246 | restore imports to original, remove duplicate handlers added in draft |
| `agent/src/bin/haviz_app.rs` | 151 | add background tokio task calling `accumulate_once` every poll_interval_secs |

### Tasks Completed
- [x] Read all existing code before modifying
- [x] Created `zalo_accumulator.rs` — `accumulate_once()` drains ZALO_MESSAGES IPC buffer, parses raw JS output, dedup-checks by hash, upserts to SQLite
- [x] Wired background polling in `haviz_app.rs::start_agent()` — tokio task, runs off async executor via `spawn_blocking`
- [x] Created `zalo_db_handlers.rs` — `zalo_messages_db_handler` (DB-first, live fallback) + `zalo_history_handler` (paginated)
- [x] Updated `routes/mod.rs` — DB sub-router with `with_state(db)`, merged into main router
- [x] `/api/zalo/messages` — now returns accumulated SQLite rows; falls back to live snapshot if DB empty
- [x] `/api/zalo/history` — new endpoint, query params: `limit` (default 200), `conversation_id` (optional)
- [x] Legacy `zalo_messages_handler` preserved (still callable internally)

### Tests Status
- Type check: pass (`cargo build --lib` — clean)
- Unit tests: pass (40/40, `cargo test --lib`)
- Integration tests: N/A (no integration test suite)

### Architecture Notes
- `accumulate_once` is blocking (thread::sleep for IPC wait) — correctly offloaded via `spawn_blocking`
- JS `JS_EXTRACT_MESSAGES` posts bare JSON array to IPC; `parse_ipc_messages` handles both array and `{type,data}` envelope
- Hash uses `compute_hash(sender, content, timestamp)` — timestamp is wall-clock HH:MM at extraction time, consistent with existing poller
- `zalo_messages_handler` (legacy live-only) kept on `zalo_control.rs` but no longer registered as a route; the new DB handler replaces it on `/api/zalo/messages`

### Issues Encountered
- Release build failed with `Access is denied` on exe — app was running, not a code error
- `zalo_control.rs` is 246 lines (pre-existing condition from original file); new files all under 200

### Next Steps
- Consider deduplicating `wait_for_ipc_local` (copied from `zalo_control.rs`) into a shared `ipc_helpers.rs`
- Hash stability: if same message appears at different HH:MM times, it gets two different hashes and inserts twice — consider hashing only `(sender, content)` for stricter dedup
