# Haviz Codebase Summary

**Last Updated:** 2026-03-19
**Total Files:** 99
**Total LOC:** ~5,000 (excluding node_modules, validation POCs)
**Status:** Phase 1 - Core MVP (In Progress)

## Repository Overview

Haviz is a **Revenue Intelligence Platform for Vietnam** built on a monorepo structure using pnpm workspaces and Turborepo. Core principle: **LOCAL-FIRST** — messages never leave user's machine.

Tagline: *"Biến mỗi cuộc trò chuyện thành doanh thu"* (Turn every conversation into revenue)

## Project Structure

```
haviz/
├── agent/               # Rust Desktop Agent (2,139 LOC, 17 .rs files)
├── apps/
│   ├── web/             # Svelte 5 + Vite 8 + Tailwind 4 (574 LOC)
│   ├── api/             # Backend (Hono + Drizzle) — NOT STARTED
│   └── mobile/          # React Native/Expo — NOT STARTED
├── extensions/
│   └── chrome/          # Chrome Extension Manifest V3 (106 LOC)
├── packages/
│   └── shared/          # Shared TypeScript types — NOT STARTED
├── validation/          # POC & browser validation scripts (1,566 LOC)
├── docker-compose.yml   # PostgreSQL 16 + Redis 7
└── Configuration files
```

## File Organization & Key Files

### Agent (Rust) — ~2,300 LOC

Located: `agent/`

| File | Purpose |
|------|---------|
| `src/bin/haviz_app.rs` | Desktop app entry point, wry webview, tray, collapsible sidebar, multi-account UI |
| `src/db.rs` | SQLite database layer (messages, conversations, contacts, drafts) |
| `src/server.rs` | Axum HTTP server, REST API, web UI serving |
| `src/polling.rs` | Polls Zalo Desktop for new messages via AX API (3s interval) |
| `src/ai.rs` | Groq API integration for AI draft generation |
| `src/channels/zalo_web.rs` | Zalo Web via chrome-headless-shell + Chrome DevTools Protocol |
| `src/routes/zalo_control.rs` | IPC routes: auto-dismiss warning, auto-load messages, scoped message extraction |
| `src/routes/zalo_accumulator.rs` | WebView poll → parse → dedup → SQLite upsert (background every 3s) |
| `src/routes/zalo_db_handlers.rs` | DB-backed /messages (accumulation) + /history (paginated) endpoints |
| `src/routes/zalo_scripts.rs` | Inline JS snippets for Zalo WebView (extract, scroll, activate, search) |
| `src/app/webview.rs` | WebView initialization, persistent session data directory |
| `src/app/ipc.rs` | IPC (Inter-Process Communication) between Rust agent and WebView |
| `src/platform/windows/input.rs` | Windows SendInput + clipboard read/write for Zalo Desktop |
| `src/channels/traits.rs` | ChannelReader + ChannelSender abstractions |
| `src/message_parser.rs` | Parse raw AX text into structured messages (cross-platform fallback) |
| `src/main.rs` | CLI entry point, starts axum server |
| `src/config.rs` | Configuration management |
| `src/platform/macos/accessibility.rs` | macOS Accessibility API access |
| `src/platform/macos/automation.rs` | AppleScript automation |
| `src/platform/windows/uiautomation.rs` | Windows UI Automation access |
| `src/channels/zalo_desktop.rs` | Zalo Desktop channel adapter |

**Key Dependencies:**
- `axum 0.7` — HTTP framework
- `tokio` — async runtime
- `rusqlite` — SQLite driver
- `wry` — embedded webview
- `tao` — window/event handling
- `groq-sdk` — AI API client

### Web UI (Svelte 5) — ~650 LOC

Located: `apps/web/src/`

| File | Purpose |
|------|---------|
| `InboxView.svelte` | Main conversation view, message thread, AI draft panel |
| `Sidebar.svelte` | Navigation sidebar with collapse toggle, collapsible layout |
| `Topbar.svelte` | Top navigation bar, status indicators, account switcher |
| `lib/components/icons.ts` | SVG icon components (replaced emojis) |
| `lib/components/inbox-view.css` | Chat bubble styling, message layout |
| `lib/components/sidebar.css` | Sidebar responsive styling, collapse animation |
| `app.css` | Global dark theme with glow effects |
| `lib/api/client.ts` | TypeScript API client for agent REST API |
| `App.svelte` | Root layout component |
| `lib/stores/app.ts` | Svelte reactive stores |
| `LogPanel.svelte` | Debug log panel with collapse |

**Build Config:**
- Svelte 5 (latest)
- Vite 8
- Tailwind CSS 4
- TypeScript 5.6

### Chrome Extension — 106 LOC

Located: `extensions/chrome/`

| File | LOC | Purpose |
|------|-----|---------|
| `manifest.json` | 14 | Manifest V3 configuration |
| `src/content/zalo-reader.js` | 92 | Content script: MutationObserver reads Zalo Web messages |

### Validation Scripts — 1,566 LOC

Located: `validation/`

Collection of POC and validation scripts for different browser approaches:
- `chrome-headless/` — chrome-headless-shell (VALIDATED ✓)
- `ax-api/` — macOS Accessibility API tests
- `zalo-web/` — Zalo Web extension tests
- `lightpanda/` — Lightpanda browser (FAILED)
- `servo/` — Servo browser (CRASHED)
- `browsers/` — General browser automation tests
- `agent/` — Agent prototype tests
- `mobile/` — Mobile validation docs

## Technology Stack

| Layer | Technology | Version | Status |
|-------|-----------|---------|--------|
| Agent Desktop | Rust + axum + wry | 0.7 / latest | Implemented |
| Agent WebView | Embedded Chromium (wry) | latest | Implemented |
| Web UI | Svelte 5 + Vite 8 | 5.0 / 8.0 | Implemented |
| Web Styling | Tailwind CSS 4 | 4.0 | Implemented |
| Chrome Extension | Manifest V3 | 3 | Implemented |
| Backend API | Hono + Drizzle | — | NOT STARTED |
| Database (Cloud) | PostgreSQL 16 | 16 | Docker-ready |
| Cache | Redis 7 | 7 | Docker-ready |
| AI Model | Groq Llama 4 Scout | — | Implemented |
| Mobile | React Native/Expo | — | NOT STARTED |

## Environment Variables

See `.env.example`:

```
VITE_PORT=3333                          # Web UI dev server port
DATABASE_URL=postgresql://...           # PostgreSQL connection
REDIS_URL=redis://...                   # Redis connection
SUPABASE_URL, SUPABASE_ANON_KEY, etc.   # Supabase (auth/storage)
GROQ_API_KEY=...                        # Groq API key for AI
ZALO_OA_ACCESS_TOKEN, REFRESH_TOKEN     # Zalo Official Account API
HAVIZ_API_URL=https://api.haviz.vn      # API base URL
AGENT_AUTH_TOKEN=...                    # Agent authentication token
```

## Development Phases

| Phase | Status | Key Deliverable |
|-------|--------|-----------------|
| 0: PoC/Validation | ✓ Complete | Browser approaches validated, agent prototype working |
| 1: Core MVP | 🔄 In Progress | Agent + Web UI + Chrome Extension |
| 2: Cloud Channels | ⏳ Not Started | Zalo OA API, Messenger Graph API, Telegram Bot API |
| 3: Mobile & Analytics | ⏳ Not Started | React Native mobile app, analytics dashboard |

## Architecture Highlights

### 3-Tier Data Model (Local-First)

| Tier | Location | Encryption | Access |
|------|----------|-----------|--------|
| Tier 1 (Templates, Settings) | Cloud | Plain | Haviz + User |
| Tier 2 (Metadata, Contacts) | Cloud | AES-256 E2E | User only |
| Tier 3 (Messages, PII) | Local SQLite | None needed | User only |

### Data Flow

1. **Message Polling** (every 3s):
   - Agent polls Zalo Desktop via AX API or chrome-headless-shell
   - Scoped message extraction to chat container with fallback to full scan
   - Up to 50 messages extracted per session
   - Raw messages stored in local SQLite

2. **Auto-Load on Conversation Open**:
   - 4s delay for open, 2s for direct search match
   - Messages load in background while user reads thread

3. **AI Draft Generation**:
   - Template matching first (0 cost)
   - If no match: call Groq API with anonymized input
   - ~200 tokens per request, ~$0.05/month per salesperson

4. **Message Sending**:
   - User approves/edits/rejects draft in dashboard
   - Agent sends via Zalo with human-like delays & safety checks
   - Auto-dismiss Zalo multi-tab warning every 5s

### Safety Engine (5 Layers)

1. **Rate Limiting** — 5 msgs/min, 30/hour per conversation, 300/day global
2. **Human-like Behavior** — Random delays, typing simulation
3. **Working Hours** — 7:00-22:00 weekdays, 8:00-20:00 weekends
4. **Content Safety** — Anti-duplicate, anti-broadcast, blacklist patterns
5. **Account Health** — 0-100 score monitoring

### Multi-Channel Support (Roadmap)

| Channel | Status | Type | Notes |
|---------|--------|------|-------|
| Zalo Personal | Implemented | Local | AX API + chrome-headless-shell |
| Zalo OA | Not Started | Cloud | Official webhook API |
| Messenger | Not Started | Cloud | Graph API |
| Telegram | Not Started | Cloud | Bot API |
| Phone/SMS | Not Started | Cloud | Twilio/VNPT integration |

## Database Schema (SQLite)

Implemented in `agent/src/db.rs`:

- `conversations` — Zalo conversation metadata
- `messages` — Raw messages with timestamps
- `contacts` — Contact information
- `drafts` — AI-generated draft history
- `style_profiles` — User writing style profiles

## Key Design Decisions

1. **Svelte 5** (instead of Next.js) — Simpler, faster, better for desktop app embedding
2. **Local-First Architecture** — Messages never leave user's machine; only metadata encrypted in cloud
3. **chrome-headless-shell** — Primary Zalo Web reader (558MB RAM, full Chrome DevTools Protocol support)
4. **Groq Llama 4 Scout** — Fast, cost-effective AI drafts (~$0.05/month per user)
5. **Rust Agent** — High performance, easy system integration (AX API, automation)
6. **Monorepo (pnpm + Turborepo)** — Efficient, shared dependencies

## Development Commands

See `agent/Makefile` and `turbo.json` for automation.

Key workspaces:
- `agent` — Rust agent (compile, run, test)
- `apps/web` — Svelte web UI (dev, build)
- `extensions/chrome` — Chrome extension (manifest-based)

## Build & Deploy Status

| Component | Build | Deploy |
|-----------|-------|--------|
| Agent | ✓ Local (Rust) | Manual (Windows/.dmg/.deb) |
| Web UI | ✓ Vite bundle | Manual / Vercel ready |
| Chrome Ext | ✓ Manifest V3 | Manual / Chrome Web Store ready |
| API Backend | ❌ Not started | — |
| Mobile | ❌ Not started | — |

## Security Considerations

- ✓ Local message storage (SQLite on user machine)
- ✓ AES-256 encryption for cloud metadata
- ✓ Groq API calls anonymized (no user identifiers)
- ✓ Agent authentication via token
- ⚠️ Zalo account credentials (local storage, future: vault integration)
- ⚠️ API backend not yet secured (Phase 2)

## Dependencies & Licenses

- **Rust Agent:** MIT (axum), Apache 2.0 (tokio), various
- **Web UI:** MIT (Svelte), various
- **Chrome Extension:** Manifest V3 standard
- Third-party APIs: Groq, Zalo, Supabase

## Performance Metrics

- **Message polling latency:** 3s interval
- **AI draft generation:** ~5-10s (Groq API)
- **Web UI load time:** <1s (Svelte optimized)
- **Agent memory:** ~150-200MB typical
- **Database:** SQLite ~50-500MB per 10k messages

## Testing Coverage

- ✓ Validation scripts for browser approaches
- ❌ Unit tests (to be added in Phase 1)
- ❌ Integration tests (to be added in Phase 2)

## Deployment Infrastructure

**Current:**
- Docker Compose (PostgreSQL 16 + Redis 7)

**Planned:**
- VN Cloud for API + Database
- Vercel for Web UI
- Manual/Auto-update for Agent

## Next Steps

1. **Phase 1 (Current):**
   - Implement backend API (Hono + Drizzle)
   - Add unit/integration tests
   - Polish Web UI
   - Deploy MVP

2. **Phase 2:**
   - Implement Zalo OA webhook API
   - Add Messenger & Telegram channels
   - Cloud message encryption
   - Mobile app (React Native)

3. **Phase 3:**
   - Analytics dashboard
   - Advanced AI features
   - SMS/Phone channel
   - Enterprise licensing
