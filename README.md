# Haviz — Revenue Intelligence Platform for Vietnam

**Tagline:** *"Biến mỗi cuộc trò chuyện thành doanh thu"* (Turn every conversation into revenue)

A LOCAL-FIRST Revenue Intelligence Platform that turns Zalo, Messenger, and SMS conversations into actionable revenue intelligence. Messages never leave your machine.

## Quick Overview

Haviz is a monorepo containing:
- **Agent** (Rust) — Desktop app that monitors Zalo, generates AI-powered reply drafts, sends with safety guardrails
- **Web UI** (Svelte 5) — Dashboard for managing drafts, templates, and analytics
- **Chrome Extension** — Reader for Zalo Web
- **Backend API** (planned) — Hono + Drizzle for cloud channels and analytics

**Status:** Phase 1 — Core MVP (Agent + Web UI + Chrome Extension implemented)

## Project Principles

1. **Local-First** — Messages never leave your machine (stored locally in SQLite)
2. **Privacy-Preserved** — Only anonymized metadata sent to cloud
3. **Revenue-Focused** — AI helps write better sales messages
4. **Safety-First** — 5-layer safety engine prevents spam/account harm

## What's Implemented

| Component | Tech | Status | Key Features |
|-----------|------|--------|--------------|
| Desktop Agent | Rust + axum + wry | ✓ Implemented | Message polling, drafts, safety engine, multi-account |
| Web Dashboard | Svelte 5 + Vite 8 + Tailwind 4 | ✓ Implemented | Collapsible sidebar, message view, draft editor, dark theme |
| Chrome Extension | Manifest V3 | ✓ Implemented | Zalo Web monitoring, message extraction |
| Backend API | Hono + Drizzle | ⏳ Planned | Auth, templates, cloud channels |
| Mobile App | React Native/Expo | ⏳ Planned | Approve drafts on-the-go |
| Zalo OA Support | Cloud API | ⏳ Planned | Official Account integration |

## Getting Started

### Prerequisites
- Node.js 18+ (for web UI + validation scripts)
- Rust 1.70+ (for agent)
- Docker & Docker Compose (for PostgreSQL + Redis)
- pnpm (package manager)

### Installation

```bash
# Clone repo
git clone <repo-url>
cd haviz

# Install dependencies
pnpm install

# Copy env file
cp .env.example .env.local
# Edit .env.local with your API keys (Groq, Zalo OA, Supabase)
```

### Start Docker Services

```bash
# Start PostgreSQL + Redis
docker-compose up -d
```

### Development Commands

```bash
# Web UI dev server (http://localhost:3333)
cd apps/web && pnpm dev

# Agent — build & run desktop app (outputs haviz-app + haviz_app binaries)
cd agent && pnpm build && pnpm start

# Agent — run CLI server only (headless, no WebView)
cd agent && cargo run --bin haviz

# Chrome extension — load in Chrome via chrome://extensions/
# -> "Load unpacked" -> extensions/chrome

# Run all turbo tasks
pnpm turbo build
pnpm turbo test    # (not yet implemented)
```

## Project Structure

```
haviz/
├── agent/                 # Rust desktop agent (2,139 LOC)
├── apps/
│   ├── web/               # Svelte 5 web UI (574 LOC)
│   ├── api/               # Backend API (Hono) — NOT STARTED
│   └── mobile/            # React Native — NOT STARTED
├── extensions/
│   └── chrome/            # Chrome extension (106 LOC)
├── packages/
│   └── shared/            # TS types — NOT STARTED
├── validation/            # POC & browser validation scripts
├── docs/                  # Project documentation
├── docker-compose.yml     # PostgreSQL 16 + Redis 7
└── Configuration files
```

**For detailed architecture, see** `docs/system-architecture.md`

## Key Features (Phase 1)

### Message Reading & Extraction
- **Zalo Desktop** — via macOS Accessibility API (AX)
- **Zalo Web** — via chrome-headless-shell + Chrome DevTools Protocol (CDP)
- **Extraction** — Scoped to chat container with fallback to full document scan (cross-platform)
- **Limit** — Up to 50 messages extracted per session
- **Polling interval** — 3 seconds (near real-time)

### AI Reply Drafts & Auto-Load
- **Template matching first** (0 cost)
- **Groq Llama 4 Scout API** (fallback)
- ~200 tokens/request, ~$0.05/month per salesperson
- **Auto-load** — Messages auto-load 4s after open, 2s after direct search match
- **Human editing** — User approves, edits, or rejects draft in dashboard

### Safety Engine (5 Layers)
1. **Rate limiting** — 5/min, 30/hour, 300/day per conversation
2. **Human-like delays** — Typing simulation, random pauses
3. **Working hours** — 7:00-22:00 weekdays, 8:00-20:00 weekends (VN timezone)
4. **Content safety** — No duplicates, no broadcasts, blacklist patterns
5. **Account health** — 0-100 score, warnings at <20, auto-dismiss multi-tab warning

### Multi-Account Support
- Multiple Zalo personal accounts
- Multiple Zalo OAs
- Single dashboard unified view
- Per-account settings & templates

## Environment Variables

See `.env.example`. Key variables:

```
VITE_PORT=3333                    # Web dev server port
GROQ_API_KEY=...                  # AI model API
ZALO_OA_ACCESS_TOKEN=...          # Zalo Official Account
SUPABASE_URL=...                  # Auth & storage (future)
DATABASE_URL=...                  # PostgreSQL (future)
REDIS_URL=...                     # Redis (future)
```

## Architecture

**3-Tier Data Model:**
- **Tier 1 (Cloud, Plain)** — Templates, settings
- **Tier 2 (Cloud, E2E Encrypted)** — Metadata, contacts
- **Tier 3 (Local SQLite)** — Messages, PII (stays on machine)

**Data Flow:**
1. Agent polls Zalo every 3s
2. Messages stored locally
3. Encrypted metadata synced to cloud
4. AI draft generated (template match first)
5. User approves → send with safety checks

See `docs/system-architecture.md` for full architecture diagram.

## Tech Stack Highlights

| Layer | Choice | Why |
|-------|--------|-----|
| Agent | Rust | Performance, system API access (AX), easy automation |
| Webview | wry | Lightweight, embedded Chromium, cross-platform |
| Web UI | Svelte 5 | Simple, fast, good for desktop embedding |
| Styling | Tailwind 4 | Utility-first, rapid UI development |
| AI | Groq Llama 4 Scout | Cost-effective, fast inference |
| Data (Local) | SQLite | Simple, file-based, no server needed |

## Development Workflow

1. **Feature branches** — `feature/xxx` from `main`
2. **Code standards** — See `docs/code-standards.md`
3. **Commit messages** — Conventional commits (feat:, fix:, docs:, etc.)
4. **Testing** — Run `pnpm turbo test` before PR (when implemented)
5. **Docs** — Update `docs/` directory with changes

## Documentation

- `docs/project-overview-pdr.md` — Product vision, goals, requirements
- `docs/codebase-summary.md` — File structure, LOC breakdown, tech stack
- `docs/code-standards.md` — Naming, patterns, code quality rules
- `docs/system-architecture.md` — Architecture diagrams, data flow, design decisions
- `docs/project-roadmap.md` — Phase breakdown, timeline, milestones
- `ARCHITECTURE.md` — Detailed Vietnamese architecture doc (legacy)

## Key Design Decisions

1. **Svelte 5** — Not Next.js (simpler, faster)
2. **Local-First** — Messages never leave machine
3. **chrome-headless-shell** — Primary Zalo Web reader
4. **Groq Llama 4 Scout** — Fast, cost-effective AI
5. **Monorepo (pnpm + Turborepo)** — Shared deps, efficient builds

## Known Limitations

- ✓ Zalo Personal (Desktop + Web)
- ⏳ Zalo OA (cloud API, Phase 2)
- ⏳ Messenger, Telegram, SMS (Phase 2-3)
- ⏳ Mobile app (React Native, Phase 3)
- ⏳ Analytics dashboard (Phase 3)

## Deployment

### Development
```bash
# Local agent dev
cargo run --manifest-path agent/Cargo.toml

# Local web dev
cd apps/web && pnpm dev
```

### Production (Future)
- **Agent** — Auto-update via Tauri / manual Windows/.dmg/.deb
- **Web UI** — Vercel deployment
- **API Backend** — VN Cloud (Hono + Drizzle)

## Support & Contribution

- **Issues** — GitHub Issues (link in repo)
- **PRs** — Follow code standards in `docs/code-standards.md`
- **Questions** — See `docs/` directory or open a discussion

## Roadmap

**Phase 1 (Current):** ✓ Agent + Web UI + Chrome Extension
**Phase 2:** Zalo OA, Messenger, Telegram, Mobile app
**Phase 3:** Analytics, Advanced AI, SMS, Enterprise features

See `docs/project-roadmap.md` for detailed timeline.

## License

(To be determined — add to repo)

## Contact

**Project Lead:** (Add contact info)
**Slack:** (Add workspace link)

---

**Last Updated:** 2026-03-19
**For latest updates, see** `docs/` **directory**
