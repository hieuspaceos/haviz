# Phase Implementation Report

## Executed Phase
- Phase: Phase 3 — Modularize Rust Agent
- Plan: none (ad-hoc task)
- Status: completed

## Files Modified
| File | Action | Lines |
|------|--------|-------|
| agent/Cargo.toml | added rusqlite_migration = "1" dep | 26 |
| agent/src/lib.rs | added app, routes modules | 10 |
| agent/src/platform/macos/mod.rs | added pub mod osascript | 3 |
| agent/src/bin/haviz_app.rs | REWRITE — 891 → 123 lines | 123 |

## Files Created
| File | Lines | Purpose |
|------|-------|---------|
| agent/src/db/mod.rs | 82 | Database struct, open(), data structs |
| agent/src/db/migrations.rs | 61 | rusqlite_migration schema v1 |
| agent/src/db/queries_messages.rs | 85 | message_exists_by_hash, insert_message, get_messages, get_recent_messages |
| agent/src/db/queries_conversations.rs | 79 | upsert_conversation, get_conversations, mark_conversation_read |
| agent/src/db/queries_drafts.rs | 55 | insert_draft, get_pending_drafts, update_draft_status |
| agent/src/db/queries_templates.rs | 57 | get_templates, insert_template, increment_template_usage |
| agent/src/platform/macos/osascript.rs | 27 | run_osascript() helper |
| agent/src/app/mod.rs | 5 | declares app submodules |
| agent/src/app/ipc.rs | 36 | ZALO_MESSAGES, ZALO_CONVERSATIONS, ZALO_JS_QUEUE statics + eval_zalo_js() + UserEvent |
| agent/src/app/app_config.rs | 30 | load_dotenv() — loads .env.local |
| agent/src/app/webview.rs | 79 | build_dashboard(), build_zalo_sidebar() |
| agent/src/app/tray.rs | 2 | placeholder TODO |
| agent/src/routes/mod.rs | 24 | extended_router() assembles all routes |
| agent/src/routes/ai_draft.rs | 39 | POST /api/ai/draft handler |
| agent/src/routes/screenshot.rs | 33 | GET /api/screenshot handler |
| agent/src/routes/zalo_control.rs | 504 | All Zalo Web sidebar handlers (JS strings inflate LOC) |

## Files Deleted
- agent/src/db.rs — replaced by agent/src/db/ directory module

## Tasks Completed
- [x] Add rusqlite_migration dep to Cargo.toml
- [x] Split db.rs into db/mod.rs + 4 query submodules + migrations.rs
- [x] Extract run_osascript to platform/macos/osascript.rs
- [x] Create app/ module with ipc.rs, app_config.rs, webview.rs, tray.rs
- [x] Create routes/ module with ai_draft.rs, screenshot.rs, zalo_control.rs, mod.rs
- [x] Slim haviz_app.rs from 891 → 123 lines
- [x] Update lib.rs to declare all new modules

## Tests Status
- cargo check: cannot run (cargo not on PATH in bash env on Windows; macOS-target code not compilable on Windows anyway)
- Manual verification: all `use crate::` paths traced, no `haviz_agent::` references inside lib modules, all pub API surfaces preserved

## Key Decisions
- `db::Database::open()` uses `expect()` for migration error (unrecoverable startup failure, simpler than ffi error mapping)
- `zalo_control.rs` is 504 lines due to verbatim inline JS strings — cannot be split without changing behavior; this is an exception to the 200-line rule
- `main.rs` (non-GUI binary) unchanged — its inline `mod db;` now resolves to `db/mod.rs` automatically
- All `crate::` paths inside lib; `haviz_agent::` paths only in bin/haviz_app.rs

## Issues Encountered
- cargo not available in bash shell on this Windows machine — static code review performed instead
- rusqlite ffi error construction is non-trivial in rusqlite 0.31; simplified to `expect()` in Database::open()

## Unresolved Questions
- rusqlite_migration v1 `M::up()` requires `&'static str`; MIGRATION_001 is a `const &'static str` so this should be fine, but needs confirmation on macOS with actual cargo check
- If rusqlite_migration v1 is incompatible with rusqlite 0.31, may need to pin to rusqlite_migration = "1.0" or upgrade rusqlite
