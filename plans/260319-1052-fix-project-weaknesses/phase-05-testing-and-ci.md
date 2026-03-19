---
phase: 5
title: "Testing & CI/CD"
priority: MEDIUM
status: pending
effort: 12h
depends_on: [1, 3]
---

# Phase 5: Testing & CI/CD

## Context Links

- [agent/src/db.rs](../../agent/src/db.rs) — 331 LOC, 0 tests
- [agent/src/message_parser.rs](../../agent/src/message_parser.rs) — 64 LOC, 0 tests
- [agent/src/ai.rs](../../agent/src/ai.rs) — 121 LOC, 0 tests
- [agent/src/polling.rs](../../agent/src/polling.rs) — 128 LOC, 0 tests
- [apps/web/package.json](../../apps/web/package.json) — no test deps
- [apps/web/src/lib/components/](../../apps/web/src/lib/components/) — Svelte components, 0 tests
- [docs/code-standards.md](../../docs/code-standards.md) — test naming conventions defined

## Overview

Zero tests across the entire codebase. No CI/CD pipeline. No GitHub Actions. No Dockerfile for API (added in Phase 1). This phase adds unit tests for Rust agent, component tests for Svelte web UI, integration tests for Hono API, and GitHub Actions for automated lint + test + build.

## Key Insights

- Rust testing: inline `#[cfg(test)] mod tests` per code standards. Focus on pure logic: message_parser, db queries, ai prompt building
- Platform-specific code (AX API, AppleScript, UI Automation) cannot be unit tested without mocking OS APIs — test the logic layer above them instead
- Svelte testing: `vitest` + `@testing-library/svelte` — Svelte 5 compatible
- Hono testing: built-in `app.request()` test helper — no need for supertest
- CI should build all 3 targets: agent (Rust), web (Svelte), api (Hono)
- GitHub Actions: use matrix for macOS + Windows agent builds

## Requirements

### Functional
- Rust unit tests for: message_parser, db (in-memory SQLite), ai (prompt building, no API calls), polling logic
- Svelte component tests for: InboxView, Sidebar, Topbar, LoginPage
- Hono API integration tests: CRUD endpoints with test database
- GitHub Actions workflow: lint, test, build on push/PR

### Non-Functional
- Rust test coverage >60% for tested modules
- All tests run in <60s (CI)
- No external API calls in tests (mock Groq API)
- Tests use in-memory or temp databases (no persistent state)
- CI runs on: ubuntu-latest (API + web), macos-latest (agent macOS), windows-latest (agent Windows)

## Architecture

### Test Structure

```
agent/
├── src/
│   ├── message_parser.rs          # inline #[cfg(test)] mod tests
│   ├── db/
│   │   ├── queries_messages.rs    # inline tests with in-memory SQLite
│   │   ├── queries_conversations.rs
│   │   ├── queries_drafts.rs
│   │   └── queries_templates.rs
│   ├── ai.rs                      # test prompt building (not API calls)
│   └── polling.rs                 # test poll_once logic with mock channel
└── tests/                         # integration tests (optional)

apps/web/
├── src/lib/components/
│   ├── InboxView.svelte
│   ├── InboxView.test.ts          # component test
│   ├── Sidebar.svelte
│   ├── Sidebar.test.ts
│   └── ...
├── vitest.config.ts
└── package.json                   # add vitest + testing-library

apps/api/
├── src/
│   ├── routes/
│   │   ├── conversations.ts
│   │   ├── conversations.test.ts  # Hono app.request() tests
│   │   ├── templates.ts
│   │   ├── templates.test.ts
│   │   └── ...
├── vitest.config.ts
└── package.json

.github/
└── workflows/
    └── ci.yml                     # lint + test + build
```

## Related Code Files

### Create
- `agent/src/message_parser.rs` — add `#[cfg(test)]` module (inline)
- `agent/src/db/queries_messages.rs` — add `#[cfg(test)]` module (inline)
- `agent/src/db/queries_conversations.rs` — add `#[cfg(test)]` module
- `agent/src/db/queries_drafts.rs` — add `#[cfg(test)]` module
- `agent/src/db/queries_templates.rs` — add `#[cfg(test)]` module
- `agent/src/ai.rs` — add `#[cfg(test)]` module
- `apps/web/vitest.config.ts`
- `apps/web/src/lib/components/InboxView.test.ts`
- `apps/web/src/lib/components/Sidebar.test.ts`
- `apps/web/src/lib/components/Topbar.test.ts`
- `apps/api/vitest.config.ts`
- `apps/api/src/routes/conversations.test.ts`
- `apps/api/src/routes/templates.test.ts`
- `apps/api/src/routes/contacts.test.ts`
- `apps/api/src/routes/agents.test.ts`
- `.github/workflows/ci.yml`

### Modify
- `apps/web/package.json` — add vitest, @testing-library/svelte, jsdom
- `apps/api/package.json` — add vitest
- `turbo.json` — add `test` task for all packages
- `agent/Cargo.toml` — ensure `[dev-dependencies]` section exists

### Delete
- None

## Implementation Steps

### Step 1: Rust unit tests — message_parser (1h)
1. Add inline `#[cfg(test)]` to `message_parser.rs`:
   - `test_compute_hash_deterministic` — same input → same hash
   - `test_compute_hash_different_input` — different input → different hash
   - `test_determine_direction_inbound` — unknown sender → inbound
   - `test_determine_direction_outbound` — matching my_name → outbound
   - `test_determine_direction_fuzzy_match` — partial name match
   - `test_determine_direction_empty_my_name` — empty → always inbound
   - `test_parse_snapshot_valid_json` — parse sample ZaloSnapshot JSON
   - `test_parse_snapshot_invalid_json` — error on bad input
   - `test_parse_snapshot_empty_messages` — returns empty Vec

### Step 2: Rust unit tests — db queries (2h)
1. Helper: `fn test_db() -> Database` — open in-memory SQLite, run migrations
2. In `queries_messages.rs`:
   - `test_insert_and_check_message_exists` — insert → exists_by_hash = true
   - `test_message_not_exists` — check non-existent hash → false
   - `test_get_messages_for_conversation` — insert multiple → get by conv_id
   - `test_get_recent_messages_limit` — respects limit param
3. In `queries_conversations.rs`:
   - `test_upsert_creates_new_conversation` — new contact → insert
   - `test_upsert_updates_existing` — same contact → update last_message
   - `test_unread_count_increments` — inbound increments, outbound doesn't
   - `test_mark_read_resets_count` — mark_read → unread_count = 0
   - `test_get_conversations_ordered` — ordered by last_message_at DESC
4. In `queries_drafts.rs`:
   - `test_insert_and_get_draft` — insert → appears in pending_drafts
   - `test_update_draft_status` — approve → no longer in pending
5. In `queries_templates.rs`:
   - `test_insert_and_get_template` — insert → appears in get_templates
   - `test_increment_usage` — increment → count increases
   - `test_templates_ordered_by_usage` — most used first

### Step 3: Rust unit tests — ai.rs (1h)
1. Test prompt building only (no HTTP calls):
   - Extract prompt building into testable function: `build_groq_messages(messages, org_context) -> Vec<GroqMessage>`
   - `test_system_prompt_contains_vietnamese` — system prompt in Vietnamese
   - `test_org_context_appended` — org_context added to system prompt
   - `test_recent_messages_limited_to_5` — >5 messages → only last 5 used
   - `test_direction_maps_to_role` — inbound=user, outbound=assistant
   - `test_empty_messages_error` — should not generate prompt for empty input

### Step 4: Svelte component tests setup (1h)
1. Install test dependencies in `apps/web`:
   - `vitest`, `@testing-library/svelte`, `jsdom`, `@sveltejs/vite-plugin-svelte`
2. Create `apps/web/vitest.config.ts`:
   ```ts
   import { defineConfig } from 'vitest/config';
   import { svelte } from '@sveltejs/vite-plugin-svelte';
   export default defineConfig({
     plugins: [svelte({ hot: false })],
     test: { environment: 'jsdom', globals: true },
   });
   ```
3. Add `"test": "vitest run"` to package.json scripts

### Step 5: Svelte component tests (2h)
1. `InboxView.test.ts`:
   - Renders conversation list with mock data
   - Shows "No conversations" when empty
   - Clicking conversation shows messages
   - Draft panel renders when draft exists
2. `Sidebar.test.ts`:
   - Renders navigation items
   - Active item highlighted
3. `Topbar.test.ts`:
   - Shows app title
   - Shows connection status

### Step 6: API integration tests setup (30min)
1. Add `vitest` to `apps/api` devDependencies
2. Create `apps/api/vitest.config.ts`
3. Add test script to package.json
4. Create test helper: `createTestApp()` — returns Hono app with test DB

### Step 7: API integration tests (2h)
1. Use Hono's `app.request()` for testing (no HTTP server needed)
2. `conversations.test.ts`:
   - GET /conversations → 200 + empty list
   - POST conversation → GET → returns created
   - PATCH /conversations/:id → updates fields
3. `templates.test.ts`:
   - POST /templates → 200 + id
   - GET /templates → includes created
   - PATCH /templates/:id → updates
4. `contacts.test.ts`:
   - GET /contacts → 200
   - PATCH /contacts/:id → updates
5. `agents.test.ts`:
   - POST /agents/register → 200 + token
   - GET /agents → includes registered

### Step 8: GitHub Actions CI workflow (1.5h)
1. Create `.github/workflows/ci.yml`:
   ```yaml
   name: CI
   on: [push, pull_request]
   jobs:
     api:
       runs-on: ubuntu-latest
       services:
         postgres:
           image: postgres:16-alpine
           env: { POSTGRES_DB: haviz_test, POSTGRES_USER: test, POSTGRES_PASSWORD: test }
           ports: ['5432:5432']
       steps:
         - uses: actions/checkout@v4
         - uses: pnpm/action-setup@v4
         - uses: actions/setup-node@v4
           with: { node-version: 20 }
         - run: pnpm install
         - run: pnpm --filter api test
         - run: pnpm --filter api build

     web:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: pnpm/action-setup@v4
         - uses: actions/setup-node@v4
           with: { node-version: 20 }
         - run: pnpm install
         - run: pnpm --filter web test
         - run: pnpm --filter web build
         - run: pnpm --filter web check

     agent-macos:
       runs-on: macos-latest
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo test --manifest-path agent/Cargo.toml
         - run: cargo build --manifest-path agent/Cargo.toml --release

     agent-windows:
       runs-on: windows-latest
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo test --manifest-path agent/Cargo.toml
         - run: cargo build --manifest-path agent/Cargo.toml --release
   ```

### Step 9: Turborepo test integration (30min)
1. Update `turbo.json`:
   - Add `"test"` pipeline task
   - Depends on build (for compiled test assets)
2. Add root-level `pnpm test` script: `turbo test`
3. Verify `pnpm turbo test` runs all tests

## Todo List

- [ ] Rust: message_parser inline tests
- [ ] Rust: db query tests (in-memory SQLite)
- [ ] Rust: ai.rs prompt building tests
- [ ] Svelte: install vitest + testing-library
- [ ] Svelte: vitest.config.ts
- [ ] Svelte: InboxView.test.ts
- [ ] Svelte: Sidebar.test.ts
- [ ] Svelte: Topbar.test.ts
- [ ] API: install vitest
- [ ] API: vitest.config.ts
- [ ] API: conversations.test.ts
- [ ] API: templates.test.ts
- [ ] API: contacts.test.ts
- [ ] API: agents.test.ts
- [ ] GitHub Actions: ci.yml
- [ ] turbo.json: add test task
- [ ] Verify all tests pass locally
- [ ] Verify CI passes on GitHub

## Success Criteria

- `cargo test` (agent) — all tests pass, >60% coverage on message_parser + db
- `pnpm --filter web test` — all component tests pass
- `pnpm --filter api test` — all integration tests pass
- `pnpm turbo test` — runs all test suites
- GitHub Actions CI green on push to main
- CI builds agent on both macOS and Windows runners

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Svelte 5 testing-library compatibility | Medium | Medium | Check @testing-library/svelte supports Svelte 5 runes |
| GitHub Actions macOS runner cost | Low | Low | Only run on PR, not every push to feature branches |
| Flaky tests from timing-dependent code | Medium | Medium | Use deterministic test data, no sleep-based assertions |
| API tests need running Postgres | Low | Medium | Use GitHub Actions service containers |

## Security Considerations

- Test databases use throwaway credentials (never production)
- CI secrets (GROQ_API_KEY, Supabase keys) not needed for unit tests
- API integration tests skip auth middleware (test CRUD logic separately)
- No real API calls to Groq in tests — mock responses
