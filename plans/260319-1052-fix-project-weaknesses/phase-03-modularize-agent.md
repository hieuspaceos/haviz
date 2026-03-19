---
phase: 3
title: "Modularize Agent"
priority: HIGH
status: pending
effort: 10h
depends_on: []
---

# Phase 3: Modularize Agent

## Context Links

- [agent/src/bin/haviz_app.rs](../../agent/src/bin/haviz_app.rs) — 891 LOC monolith
- [agent/src/db.rs](../../agent/src/db.rs) — 331 LOC, schema hardcoded
- [agent/src/server.rs](../../agent/src/server.rs) — 204 LOC
- [agent/Cargo.toml](../../agent/Cargo.toml) — current dependencies
- [docs/code-standards.md](../../docs/code-standards.md) — 200 LOC max per file rule

## Overview

`haviz_app.rs` is an 891 LOC monolith containing: tray icon setup, webview creation, IPC handlers, Zalo JS evaluation queue, env loading, server startup, AI draft handler, screenshot handler, 8+ Zalo control route handlers, and AppleScript execution. Code standards mandate <200 LOC per file.

Additionally, `db.rs` (331 LOC) has schema hardcoded inline with `CREATE TABLE IF NOT EXISTS` — no versioned migration system.

## Key Insights

- `haviz_app.rs` has clear separation boundaries:
  1. **Window/webview setup** (lines 42-173) — event loop, window builder, sidebar
  2. **Server startup + env loading** (lines 175-215) — tokio runtime, config, router
  3. **AI draft handler** (lines 217-261) — single endpoint
  4. **Screenshot handler** (lines 263-291) — macOS-specific
  5. **Zalo control handlers** (lines 293-891) — AppleScript wrappers, IPC, JS queue
- IPC data (`ZALO_CONVERSATIONS`, `ZALO_MESSAGES`, `ZALO_JS_QUEUE`) is static Mutex — can be extracted to shared state module
- `run_osascript()` helper is reused across all Zalo handlers — belongs in platform/macos
- DB schema is `CREATE TABLE IF NOT EXISTS` — no way to add columns or migrate existing data
- `rusqlite-migration` crate is lightweight and well-maintained for SQLite migrations

## Requirements

### Functional
- Split haviz_app.rs into modules, each <200 LOC
- Add SQLite migration system with version tracking
- Extract shared IPC state to standalone module
- Move platform-specific handlers to platform/ module
- Keep all existing functionality working identically

### Non-Functional
- Zero behavior change — pure refactoring
- Each new file <200 LOC
- Clear module boundaries with minimal public API surface
- All current routes and IPC still work after refactoring

## Architecture

### New Module Structure

```
agent/src/
├── bin/
│   └── haviz_app.rs              # Slim entry: window setup + event loop (~150 LOC)
├── app/                           # NEW: Desktop app modules
│   ├── mod.rs                     # Re-exports
│   ├── webview.rs                 # Dashboard + sidebar webview creation (~80 LOC)
│   ├── ipc.rs                     # IPC state (ZALO_CONVERSATIONS, ZALO_MESSAGES, JS_QUEUE) (~60 LOC)
│   ├── tray.rs                    # System tray (future, placeholder) (~20 LOC)
│   └── app_config.rs             # .env.local loading + agent startup (~50 LOC)
├── routes/                        # NEW: Extract route handlers from haviz_app.rs
│   ├── mod.rs                     # Re-exports + extended router builder
│   ├── ai_draft.rs               # AI draft handler (~50 LOC)
│   ├── screenshot.rs             # Screenshot handler (~40 LOC)
│   └── zalo_control.rs           # All Zalo handlers: search, open, send, conversations, messages (~180 LOC)
├── platform/
│   ├── mod.rs                     # #[cfg] platform selection
│   ├── macos/
│   │   ├── mod.rs
│   │   ├── accessibility.rs       # AX API (existing)
│   │   ├── automation.rs          # AppleScript send_message (existing)
│   │   └── osascript.rs          # NEW: run_osascript() helper (~50 LOC)
│   └── windows/                   # (Phase 4)
│       └── mod.rs
├── db/                            # NEW: Split db.rs into module directory
│   ├── mod.rs                     # Re-export Database struct
│   ├── schema.rs                 # Table creation (migrations handle this now)
│   ├── migrations.rs             # Migration runner + version tracking (~50 LOC)
│   ├── queries_messages.rs       # Message CRUD queries (~80 LOC)
│   ├── queries_conversations.rs  # Conversation CRUD queries (~80 LOC)
│   ├── queries_drafts.rs         # Draft CRUD queries (~60 LOC)
│   ├── queries_templates.rs      # Template CRUD queries (~60 LOC)
│   └── migrations/               # SQL migration files
│       ├── 001_initial_schema.sql
│       └── (future migrations)
├── server.rs                      # Existing (keep, ~200 LOC)
├── ai.rs                          # Existing (keep, 121 LOC)
├── polling.rs                     # Existing (keep, 128 LOC)
├── message_parser.rs             # Existing (keep, 64 LOC)
├── config.rs                      # Existing (keep, 40 LOC)
├── channels/                      # Existing (keep)
├── lib.rs                         # Update: add new module declarations
└── main.rs                        # Existing CLI entry (keep)
```

## Related Code Files

### Create
- `agent/src/app/mod.rs`
- `agent/src/app/webview.rs`
- `agent/src/app/ipc.rs`
- `agent/src/app/tray.rs`
- `agent/src/app/app_config.rs`
- `agent/src/routes/mod.rs`
- `agent/src/routes/ai_draft.rs`
- `agent/src/routes/screenshot.rs`
- `agent/src/routes/zalo_control.rs`
- `agent/src/platform/macos/osascript.rs`
- `agent/src/db/mod.rs`
- `agent/src/db/migrations.rs`
- `agent/src/db/queries_messages.rs`
- `agent/src/db/queries_conversations.rs`
- `agent/src/db/queries_drafts.rs`
- `agent/src/db/queries_templates.rs`
- `agent/src/db/migrations/001_initial_schema.sql`

### Modify
- `agent/src/bin/haviz_app.rs` — strip to ~150 LOC, import from new modules
- `agent/src/lib.rs` — add `pub mod app; pub mod routes;` and change `pub mod db;`
- `agent/src/platform/macos/mod.rs` — add `pub mod osascript;`
- `agent/Cargo.toml` — add `rusqlite-migration` dependency

### Delete
- `agent/src/db.rs` — replaced by `agent/src/db/` directory module

## Implementation Steps

### Step 1: Add migration dependency (15min)
1. Add `rusqlite-migration = "1"` to Cargo.toml
2. Verify it compiles

### Step 2: Split db.rs into module directory (2h)
1. Create `agent/src/db/` directory
2. Create `agent/src/db/migrations/001_initial_schema.sql`:
   - Move the CREATE TABLE statements from db.rs
   - Include all 4 tables + 3 indexes
3. Create `agent/src/db/migrations.rs`:
   - Use `rusqlite_migration::Migrations` to define and apply migrations
   - Load SQL from embedded files or inline strings
   - `pub fn run_migrations(conn: &Connection) -> Result<()>`
4. Create `agent/src/db/queries_messages.rs`:
   - Move `message_exists_by_hash()`, `insert_message()`, `get_messages()`, `get_recent_messages()`
   - Keep `impl Database` methods, import Database from mod.rs
5. Create `agent/src/db/queries_conversations.rs`:
   - Move `upsert_conversation()`, `get_conversations()`, `mark_conversation_read()`
6. Create `agent/src/db/queries_drafts.rs`:
   - Move `insert_draft()`, `get_pending_drafts()`, `update_draft_status()`
7. Create `agent/src/db/queries_templates.rs`:
   - Move `get_templates()`, `insert_template()`, `increment_template_usage()`
8. Create `agent/src/db/mod.rs`:
   - Define `Database` struct with `conn: Mutex<Connection>`
   - Define all data structs (Message, Conversation, AiDraft, Template)
   - `Database::open()` — open connection, run migrations, return Self
   - Re-export query modules
9. Delete `agent/src/db.rs`
10. Update `agent/src/lib.rs` — module declaration

### Step 3: Extract run_osascript to platform module (30min)
1. Create `agent/src/platform/macos/osascript.rs`:
   - Move `run_osascript()` function from haviz_app.rs
   - Make it `pub fn run_osascript(script: &str) -> Result<String, String>`
2. Update `agent/src/platform/macos/mod.rs` to declare the module

### Step 4: Extract IPC state (1h)
1. Create `agent/src/app/ipc.rs`:
   - Move `ZALO_CONVERSATIONS`, `ZALO_MESSAGES`, `ZALO_JS_QUEUE` statics
   - Move `eval_zalo_js()` function
   - Move `UserEvent` enum
2. Create `agent/src/app/mod.rs`:
   - Declare submodules

### Step 5: Extract route handlers (2h)
1. Create `agent/src/routes/ai_draft.rs`:
   - Move `AiDraftRequest` struct and `ai_draft_handler` from haviz_app.rs
2. Create `agent/src/routes/screenshot.rs`:
   - Move `screenshot_handler` from haviz_app.rs
3. Create `agent/src/routes/zalo_control.rs`:
   - Move all Zalo handlers: `zalo_search_handler`, `zalo_open_handler`, `zalo_send_handler`, `zalo_search_and_send_handler`, `zalo_conversations_handler`, `zalo_messages_handler`, `zalo_messages_callback`
   - Move request/response structs
   - Import `run_osascript` from platform module
   - Import IPC state from app module
4. Create `agent/src/routes/mod.rs`:
   - `pub fn extended_router() -> Router` — build the extended router with all routes
   - This replaces the inline route additions in haviz_app.rs `start_agent()`

### Step 6: Extract webview + app config (1.5h)
1. Create `agent/src/app/webview.rs`:
   - Function to create dashboard webview (params: window ref, bounds)
   - Function to create Zalo sidebar webview (params: window ref, bounds, IPC handler)
2. Create `agent/src/app/app_config.rs`:
   - Move .env.local loading logic from `start_agent()`
   - `pub fn load_dotenv()` — reads .env.local and sets env vars
3. Create `agent/src/app/tray.rs`:
   - Placeholder module with TODO comments for future system tray implementation

### Step 7: Slim down haviz_app.rs (1.5h)
1. Rewrite `haviz_app.rs` to be thin orchestrator:
   - Import from `app::ipc`, `app::webview`, `app::app_config`
   - `main()`: print banner, spawn agent thread, create event loop, create window, create webviews, run event loop
   - `start_agent()`: call `app_config::load_dotenv()`, load config, open DB, build `routes::extended_router()`, bind and serve
   - Event loop handler: process JS queue (delegated to ipc module)
   - Target: ~150 LOC

### Step 8: Verify everything compiles and works (1h)
1. `cargo build` — ensure no compile errors
2. `cargo run --bin haviz_app` — verify app starts
3. Test all API endpoints still work
4. Test webview loads correctly
5. Test Zalo sidebar IPC works
6. Verify DB migration creates tables correctly on fresh DB

## Todo List

- [ ] Add rusqlite-migration to Cargo.toml
- [ ] Create db/ module directory with migrations
- [ ] Split db.rs queries into 4 query files
- [ ] Create db/mod.rs with Database struct + migration runner
- [ ] Extract run_osascript to platform/macos/osascript.rs
- [ ] Extract IPC state to app/ipc.rs
- [ ] Extract AI draft handler to routes/ai_draft.rs
- [ ] Extract screenshot handler to routes/screenshot.rs
- [ ] Extract Zalo control handlers to routes/zalo_control.rs
- [ ] Create routes/mod.rs with extended_router builder
- [ ] Extract webview creation to app/webview.rs
- [ ] Extract .env.local loading to app/app_config.rs
- [ ] Create app/tray.rs placeholder
- [ ] Slim haviz_app.rs to ~150 LOC
- [ ] Update lib.rs with new module declarations
- [ ] Verify compilation
- [ ] Smoke test all functionality

## Success Criteria

- `haviz_app.rs` < 200 LOC
- All new files < 200 LOC
- `db.rs` replaced by `db/` directory with migration system
- `cargo build` succeeds with zero warnings (or only pre-existing ones)
- All existing functionality works identically (manual smoke test)
- Migration system creates tables on fresh DB
- DB with existing data still works (backward compatible)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Static Mutex IPC breaks when moved | Medium | High | Test IPC thoroughly; static references work across modules |
| Migration breaks existing DB data | Low | High | Migration 001 uses CREATE TABLE IF NOT EXISTS |
| Import cycle between modules | Low | Medium | Clear dependency direction: routes → app → platform |
| wry/tao API requires specific ownership | Medium | Medium | Keep WebView creation in main thread, extract builders only |

## Security Considerations

- No security changes in this phase (pure refactoring)
- Auth middleware (Phase 2) will be easier to add after modularization
- `run_osascript` is a potential command injection vector — ensure it only accepts known scripts, never user input directly
