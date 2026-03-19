# Haviz System Architecture

**Version:** 1.0
**Last Updated:** 2026-03-19
**Status:** Phase 1 - Core MVP (In Progress)

## Overview

Haviz follows a **3-tier local-first data architecture** with a desktop agent, web dashboard, and cloud backend (Phase 2+).

## System Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    USER'S MACHINE                        │
│                                                           │
│  ┌──────────────────────────────────────────────────┐   │
│  │        Haviz Desktop Agent (Rust)                │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │  Message Readers                           │  │   │
│  │  │  • Zalo Desktop (AX API)                    │  │   │
│  │  │  • Zalo Web (chrome-headless-shell + CDP)  │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  │                      ↓                            │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │  Local SQLite Database (Tier 3)            │  │   │
│  │  │  • Messages (never synced to cloud)        │  │   │
│  │  │  • Conversations, contacts, drafts         │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  │                      ↓                            │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │  Axum HTTP Server (REST API)               │  │   │
│  │  │  • Serves web UI                           │  │   │
│  │  │  • Exposes local API                       │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
│           ↓ (Embedded webview)           ↓              │
│  ┌─────────────────────┐  ┌───────────────────────┐   │
│  │  Web UI             │  │  Chrome Extension     │   │
│  │  (Svelte 5)         │  │  (Manifest V3)        │   │
│  │  • Conversations    │  │  • Reads Zalo Web     │   │
│  │  • Drafts           │  │  • Monitors messages  │   │
│  │  • Templates        │  │  • Sends notifications│   │
│  └─────────────────────┘  └───────────────────────┘   │
│                                                           │
└─────────────────────────────────────────────────────────┘
                          ↓ (HTTPS)
┌─────────────────────────────────────────────────────────┐
│                   HAVIZ CLOUD                            │
│                                                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Cloud Backend (Phase 2: Hono + Drizzle)          │ │
│  │  • User authentication (Supabase)                  │ │
│  │  • API for cloud channels (Zalo OA, Messenger)   │ │
│  │  • Template management                            │ │
│  │  • Analytics aggregation                          │ │
│  └────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Data Tiers                                        │ │
│  │  Tier 1 (Plain): Templates, settings              │ │
│  │  Tier 2 (E2E Encrypted AES-256): Metadata         │ │
│  │  Tier 3 (Local Only): Messages, PII               │ │
│  └────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Databases                                         │ │
│  │  • PostgreSQL 16 (user data, templates)           │ │
│  │  • Redis 7 (sessions, cache)                      │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│              EXTERNAL SERVICES                           │
│                                                           │
│  • Groq API (Llama 4 Scout) — AI draft generation      │
│  • Zalo OA API (Phase 2) — Official Account channel    │
│  • Facebook Graph API (Phase 2) — Messenger            │
│  • Telegram Bot API (Phase 2) — Telegram               │
│  • Twilio/VNPT API (Phase 3) — SMS/Phone              │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### Desktop Agent (Rust)

**Purpose:** Core intelligence center. Monitors conversations, generates drafts, sends replies.

**Key Modules:**
- `src/server.rs` — Axum HTTP server (204 LOC)
- `src/db.rs` — SQLite database layer (331 LOC)
- `src/polling.rs` — Message polling loop (128 LOC)
- `src/ai.rs` — Groq API integration (120 LOC)
- `src/channels/` — Channel readers/senders (169 LOC)
- `src/bin/haviz_app.rs` — Desktop app UI (891 LOC)

**Data Flow:**
1. Poll Zalo for new messages (3s interval)
2. Parse messages → store in SQLite
3. Generate AI draft (templates first, then Groq)
4. Display to user in web UI
5. User approves → check safety engine → send

See [Agent & Data Flow Details](./architecture/agent-dataflow.md)

### Web UI (Svelte 5)

**Purpose:** Dashboard for managing conversations, drafts, templates.

**Key Components:**
- InboxView — Conversations + message thread (270 LOC)
- Sidebar — Navigation (86 LOC)
- Topbar — Status bar (53 LOC)
- API client — REST communication (102 LOC)

**Features:**
- Real-time message updates
- Multi-account switcher
- Virtual scrolling (1000+ messages)
- Draft editing & approval

See [Web UI Details](./architecture/web-ui.md)

### Chrome Extension

**Purpose:** Read Zalo Web messages, integrate with web browser.

**Technology:** Manifest V3, MutationObserver, content script

**Files:**
- `manifest.json` — Configuration (14 LOC)
- `src/content/zalo-reader.js` — Message extraction (92 LOC)

See [Chrome Extension Details](./architecture/chrome-extension.md)

## Data Architecture (3-Tier Model)

| Tier | Location | Encryption | Contents | Access |
|------|----------|-----------|----------|--------|
| **Tier 1** | Cloud | None | Templates, settings | Haviz + User |
| **Tier 2** | Cloud | AES-256 E2E | Metadata, contacts | User only |
| **Tier 3** | Local SQLite | None | Messages, PII | User only |

**Rationale:**
- Tier 1: No PII, safe to sync
- Tier 2: Sensitive but can be encrypted end-to-end
- Tier 3: Most sensitive stays local, never synced

See [Data Architecture](./architecture/data-architecture.md)

## Message Reading Channels

### 1. Zalo Desktop (AX API)

**Supported:** macOS
**Method:** macOS Accessibility API

Reads messages directly from Zalo.app UI tree without browser automation.

### 2. Zalo Web (CDP)

**Supported:** All platforms
**Method:** chrome-headless-shell + Chrome DevTools Protocol

Headless Chromium (558MB) opens zalo.me and reads DOM.

### 3. Chrome Extension

**Supported:** All platforms
**Method:** MutationObserver on Zalo Web DOM

Lightweight (5MB) content script integrated into user's browser.

See [Channel Readers](./architecture/message-reading.md)

## AI Draft Generation

```
Message detected
     ↓
Check templates first (0 cost)
     ├─ Match found? → Use template
     └─ No match? → Call Groq API
            ↓
      Groq Llama 4 Scout
      (~200 tokens, $0.002)
            ↓
      Show draft to user
      (Approve/Edit/Reject)
```

See [AI Draft Pipeline](./architecture/ai-drafts.md)

## Safety Engine (5 Layers)

1. **Rate Limiting** — 5/min, 30/hour, 300/day
2. **Human-like Behavior** — Delays, typing simulation
3. **Working Hours** — 7-22 weekdays, 8-20 weekends
4. **Content Safety** — No duplicates, no broadcasts
5. **Account Health** — 0-100 score, warnings at <20

See [Safety Engine](./architecture/safety-engine.md)

## Database Schema (SQLite)

Key tables:
- `conversations` — Metadata per conversation
- `messages` — Raw messages with timestamps
- `contacts` — Contact information
- `drafts` — AI-generated draft history
- `channels` — Account credentials
- `style_profiles` — User writing style

See [Database Schema](./architecture/database.md)

## Deployment Architecture

### Phase 1: Local

Agent runs locally on user's machine, web UI embedded in wry webview.

### Phase 2+: Hybrid

Agent still local (messages + SQLite), but cloud backend handles Zalo OA, Messenger, Telegram webhooks.

See [Deployment](./architecture/deployment.md)

## Multi-Channel Support (Roadmap)

| Channel | Phase | Status |
|---------|-------|--------|
| Zalo Personal | 1 | Implemented |
| Zalo OA | 2 | Planned |
| Messenger | 2 | Planned |
| Telegram | 2 | Planned |
| SMS/Phone | 3 | Planned |

## Design Principles

1. **Local-First** — Messages never leave user's machine
2. **Privacy-Preserved** — Metadata encrypted, no message logging
3. **Multi-Channel Ready** — Trait-based channel abstraction
4. **Safety-First** — 5-layer safety engine
5. **Scalable** — Support 1000+ conversations, 100+ users
6. **Type-Safe** — Rust agent, TypeScript web
7. **Observable** — Logging, metrics, health scoring

---

## Documentation Index

| Document | Purpose |
|----------|---------|
| [Agent & Data Flow](./architecture/agent-dataflow.md) | Polling loop, message processing, AI pipeline |
| [Web UI](./architecture/web-ui.md) | Svelte components, state management, real-time updates |
| [Chrome Extension](./architecture/chrome-extension.md) | Manifest V3, content script, message extraction |
| [Data Architecture](./architecture/data-architecture.md) | 3-tier model, encryption, cloud sync |
| [Message Reading](./architecture/message-reading.md) | Channel readers (AX API, CDP, Extension) |
| [AI Drafts](./architecture/ai-drafts.md) | Template matching, Groq API, user editing |
| [Safety Engine](./architecture/safety-engine.md) | Rate limiting, account health, working hours |
| [Database](./architecture/database.md) | SQLite schema, migrations, indexes |
| [Deployment](./architecture/deployment.md) | Local, hybrid, scaling architecture |

---

## Quick Reference

**Agent Technology Stack:**
- Rust 1.70+, axum 0.7, tokio, wry, SQLite

**Web UI Stack:**
- Svelte 5, Vite 8, Tailwind 4, TypeScript 5.6

**External Services:**
- Groq API (AI), Zalo (messaging), Supabase (auth, Phase 2+)

**Performance Targets:**
- Message poll: <500ms
- Draft generation: <10s
- Web UI load: <1s
- Send confirmation: <2s

---

**See also:**
- `docs/project-overview-pdr.md` — Product requirements
- `docs/code-standards.md` — Coding conventions
- `docs/project-roadmap.md` — Development phases
- `ARCHITECTURE.md` — Detailed Vietnamese architecture
