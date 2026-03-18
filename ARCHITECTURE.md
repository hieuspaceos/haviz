# HAVIZ Architecture
**Revenue Intelligence Platform for Vietnam**

*"Biến mỗi cuộc trò chuyện thành doanh thu"*

---

## 1. Tổng quan hệ thống

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                                │
│                                                                     │
│  ┌──────────────┐  ┌───────────────┐  ┌──────────────────────────┐ │
│  │  Web App      │  │  Mobile App   │  │  Rust Desktop Agent      │ │
│  │  Next.js 14   │  │  React Native │  │  localhost:9999           │ │
│  │  Vercel       │  │  iOS/Android  │  │  ~3-5MB, system tray     │ │
│  └──────┬────────┘  └──────┬────────┘  └──────┬─────────┬─────────┘ │
│         │                  │                  │         │           │
│     HTTPS+WSS          HTTPS+WSS          WSS(out)   HTTP(in)     │
│         │                  │                  │         │           │
└─────────┼──────────────────┼──────────────────┼─────────┼───────────┘
          │                  │                  │         │
          ▼                  ▼                  ▼         │
┌─────────────────────────────────────────────────┐      │
│                CORE PLATFORM                     │      │
│                                                  │      │
│  ┌────────────────────────────────────────────┐  │      │
│  │  API Server (Hono + Node.js)               │  │      │
│  │  REST: /api/v1/*  │  WebSocket: /ws         │  │      │
│  │  Webhooks: /webhooks/*                     │  │      │
│  └──────┬──────────────────┬──────────────────┘  │      │
│         │                  │                     │      │
│    ┌────▼─────┐     ┌──────▼───────┐             │      │
│    │PostgreSQL│     │    Redis     │             │      │
│    │(Supabase)│     │  + BullMQ   │             │      │
│    └──────────┘     └──────────────┘             │      │
│                                                  │      │
│  ┌────────────────────────────────────────────┐  │      │
│  │  AI Layer                                  │  │      │
│  │  Groq + Llama 4 Scout (draft, chatbot)     │  │      │
│  │  Groq Whisper (voice → text)               │  │      │
│  └────────────────────────────────────────────┘  │      │
│                                                  │      │
└──────────────────────────────────────────────────┘      │
                                                          │
┌─────────────────────────────────────────────────────────┤
│                    CHANNEL LAYER                        │
│                                                         │
│  ┌──────────────────┐                                   │
│  │ Zalo Desktop App │◄── AX API (đọc) + AppleScript    │
│  └──────────────────┘    (gửi) via Rust Agent           │
│  ┌──────────────────┐                                   │
│  │ Zalo Web Webview │◄── Embedded webview trong Rust    │
│  │ (chat.zalo.me)   │    Agent — tọa độ cố định,       │
│  └──────────────────┘    DOM inject + Agent HTTP ───────┘
│  ┌──────────────────┐
│  │ Zalo OA          │──── Official API + Webhook
│  └──────────────────┘
│  ┌──────────────────┐
│  │ FB Messenger     │──── Graph API + Webhook (Phase 3)
│  └──────────────────┘
│  ┌──────────────────┐
│  │ Telegram         │──── Bot API + Webhook (Phase 3)
│  └──────────────────┘
└─────────────────────────────────────────────────────────┘
```

---

## 2. Monorepo Structure

```
haviz/
├── agent/                          # Rust Desktop Agent
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                 # Entry: tray icon + event loop
│   │   ├── config.rs               # Server URL, auth token
│   │   ├── server.rs               # Axum HTTP+WS server :9999
│   │   ├── ws_client.rs            # WS client → Haviz cloud
│   │   ├── polling.rs              # Poll Zalo mỗi 3s cho tin nhắn mới
│   │   ├── message_parser.rs       # Parse raw AX text → structured message
│   │   ├── tray.rs                 # System tray icon + menu
│   │   ├── webview.rs              # Embedded webview (chat.zalo.me)
│   │   ├── channels/
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs           # ChannelReader + ChannelSender traits
│   │   │   ├── zalo_desktop.rs     # Zalo Desktop automation
│   │   │   └── zalo_webview.rs     # Zalo Web via embedded webview
│   │   └── platform/
│   │       ├── mod.rs
│   │       ├── macos/
│   │       │   ├── mod.rs
│   │       │   ├── accessibility.rs # AX API đọc Zalo hierarchy
│   │       │   └── automation.rs    # AppleScript: search, paste, enter
│   │       └── windows/
│   │           ├── mod.rs
│   │           ├── uiautomation.rs  # Win32 UI Automation API
│   │           └── input.rs         # SendInput for keystrokes
│   └── resources/
│       ├── icon.icns               # macOS
│       └── icon.ico                # Windows
│
├── apps/
│   ├── api/                        # Backend API
│   │   ├── src/
│   │   │   ├── server.ts           # Hono entry
│   │   │   ├── db/
│   │   │   │   ├── schema/         # Drizzle schema
│   │   │   │   └── migrations/
│   │   │   ├── routes/
│   │   │   │   ├── auth.ts
│   │   │   │   ├── conversations.ts
│   │   │   │   ├── messages.ts
│   │   │   │   ├── contacts.ts
│   │   │   │   ├── channels.ts
│   │   │   │   ├── templates.ts
│   │   │   │   ├── ai.ts
│   │   │   │   ├── agents.ts
│   │   │   │   └── webhooks.ts
│   │   │   ├── services/
│   │   │   │   ├── message.service.ts
│   │   │   │   ├── conversation.service.ts
│   │   │   │   ├── ai.service.ts   # Groq API
│   │   │   │   ├── agent.service.ts
│   │   │   │   └── notification.service.ts
│   │   │   ├── workers/
│   │   │   │   ├── message-ingest.worker.ts
│   │   │   │   ├── ai-draft.worker.ts
│   │   │   │   ├── send-message.worker.ts
│   │   │   │   └── notification.worker.ts
│   │   │   ├── ws/
│   │   │   │   ├── handler.ts
│   │   │   │   └── events.ts
│   │   │   └── middleware/
│   │   │       ├── auth.ts
│   │   │       └── rate-limit.ts
│   │   ├── package.json
│   │   ├── drizzle.config.ts
│   │   └── Dockerfile
│   │
│   ├── web/                        # Next.js Web App
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── (auth)/login/
│   │   │   │   ├── (dashboard)/
│   │   │   │   │   ├── inbox/      # Universal Inbox
│   │   │   │   │   ├── contacts/
│   │   │   │   │   ├── templates/
│   │   │   │   │   ├── channels/
│   │   │   │   │   ├── analytics/  # Phase 2
│   │   │   │   │   └── settings/
│   │   │   │   └── layout.tsx
│   │   │   ├── components/
│   │   │   │   ├── inbox/
│   │   │   │   │   ├── conversation-list.tsx
│   │   │   │   │   ├── message-thread.tsx
│   │   │   │   │   ├── message-composer.tsx
│   │   │   │   │   └── ai-draft-panel.tsx
│   │   │   │   └── ui/             # shadcn components
│   │   │   ├── hooks/
│   │   │   │   ├── use-websocket.ts
│   │   │   │   └── use-conversations.ts
│   │   │   └── stores/             # Zustand
│   │   │       ├── conversation.store.ts
│   │   │       └── message.store.ts
│   │   └── package.json
│   │
│   └── mobile/                     # React Native (Expo)
│       ├── app/
│       │   ├── (tabs)/inbox.tsx
│       │   ├── conversation/[id].tsx
│       │   └── _layout.tsx
│       └── package.json
│
├── extensions/
│   └── chrome/                     # Chrome Extension
│       ├── manifest.json
│       ├── src/
│       │   ├── content/
│       │   │   └── zalo-reader.ts  # MutationObserver → Agent
│       │   ├── background/
│       │   │   └── service-worker.ts
│       │   └── popup/
│       │       └── popup.html
│       └── package.json
│
├── packages/
│   └── shared/                     # Shared TypeScript types
│       ├── src/types/
│       │   ├── message.ts
│       │   ├── channel.ts
│       │   └── user.ts
│       └── package.json
│
├── docker-compose.yml              # Postgres + Redis local dev
├── turbo.json
├── pnpm-workspace.yaml
├── package.json
└── .env.example
```

---

## 3. Data Flow: Tin nhắn đến → AI Draft → Approve → Gửi lại

```
STEP 1: POLLING
───────────────
Rust Agent poll Zalo Desktop mỗi 3 giây:
  macOS: AX API traverse hierarchy
  → Tìm AXStaticText depth 22 (nội dung), depth 21 (tên), depth 18 (time)
  → Parse: "Phan Trung Kiên" + "E chào chị" + "15:57"
  → So sánh với state cũ (in-memory hash) → detect tin mới

STEP 2: INGEST (Agent → Server)
────────────────────────────────
Agent gửi qua WebSocket → cloud:
  { type: "messages:new", payload: {
      channelType: "zalo_personal",
      messages: [{ sender: "Phan Trung Kiên", content: "E chào chị", time: "15:57" }]
  }}

STEP 3: PROCESS (Server)
─────────────────────────
BullMQ message:ingest worker:
  1. Resolve/create Contact (by channelId + sender name)
  2. Resolve/create Conversation
  3. Insert Message (direction: inbound)
  4. Update Conversation (last_message, unread_count++)
  5. Enqueue → ai:draft queue
  6. Broadcast → WebSocket → all org users

STEP 4: AI DRAFT
─────────────────
BullMQ ai:draft worker:
  1. Load 20 tin nhắn gần nhất + templates
  2. Call Groq API (Llama 4 Scout):
     System: "Trợ lý bán hàng tiếng Việt, thân thiện..."
  3. Insert AiDraft (status: pending)
  4. Broadcast → WebSocket: "ai:draft:ready"

STEP 5: HIỂN THỊ (Web App)
───────────────────────────
  - Inbox cập nhật real-time (conversation nhảy lên đầu)
  - AI Draft Panel hiện gợi ý reply
  - User chọn: Approve | Edit | Reject

STEP 6: APPROVE → GỬI (Web → Server → Agent)
──────────────────────────────────────────────
User click Approve:
  POST /api/v1/ai/drafts/:id/approve
  → Insert Message (direction: outbound, status: pending)
  → BullMQ send-message → push command → Agent WebSocket:
    { type: "message:send", payload: { to: "Phan Trung Kiên", content: "..." } }

STEP 7: GỬI (Agent → Zalo)
───────────────────────────
Rust Agent nhận command:
  macOS: Cmd+F → gõ tên → Enter → chờ 500ms → Cmd+V paste → Enter
  Windows: Win32 SendInput tương tự
  → Respond: { type: "message:sent", messageId: "..." }
```

---

## 4. Database Schema

```sql
-- Tổ chức (multi-tenant)
organizations    (id, name, slug, plan, settings)

-- Users (liên kết Supabase Auth)
users            (id, auth_id, org_id, email, full_name, role, avatar_url)

-- Kênh kết nối
channels         (id, org_id, type, name, status, config, agent_id)
                 -- type: zalo_personal | zalo_oa | messenger | telegram

-- Desktop agents
agents           (id, org_id, user_id, name, platform, version, status, auth_token, last_seen_at)

-- Liên hệ khách hàng
contacts         (id, org_id, display_name, phone, email, tags, metadata)
contact_channels (id, contact_id, channel_id, external_id, external_name)

-- Hội thoại
conversations    (id, org_id, channel_id, contact_id, assigned_to, status, priority,
                  tags, last_message_at, last_message_preview, unread_count)

-- Tin nhắn
messages         (id, conversation_id, org_id, direction, sender_type, sender_id,
                  content_type, content, metadata, status, sent_at)
                 -- direction: inbound | outbound
                 -- status: received | draft | pending | sent | failed

-- AI Draft
ai_drafts        (id, conversation_id, message_id, content, model, confidence,
                  status, approved_by, edited_content)
                 -- status: pending | approved | rejected | edited

-- Templates
templates        (id, org_id, name, content, category, variables, usage_count)

-- Phase 2
voice_reports    (id, org_id, user_id, audio_url, transcript, extracted_data)
analytics_events (id, org_id, event_type, conversation_id, properties)
```

---

## 5. Agent Protocol (WebSocket)

### Agent → Server
| Event | Payload |
|---|---|
| `agent:connect` | `{ agentId, platform, version }` |
| `agent:heartbeat` | `{ uptime, activeChannels }` |
| `messages:new` | `{ channelType, messages[] }` |
| `message:sent` | `{ messageId, status }` |
| `message:failed` | `{ messageId, error }` |

### Server → Agent
| Event | Payload |
|---|---|
| `message:send` | `{ messageId, to, content, channelType }` |
| `messages:poll` | `{ channelType }` |
| `agent:config` | `{ pollInterval, channels }` |

---

## 6. API Endpoints (Phase 1)

### Auth
| Method | Path |
|---|---|
| POST | `/auth/signup` |
| POST | `/auth/login` |
| GET | `/auth/me` |

### Conversations
| Method | Path |
|---|---|
| GET | `/conversations` — list (filter: status, channel, assigned) |
| GET | `/conversations/:id` |
| PATCH | `/conversations/:id` — assign, tag, status |

### Messages
| Method | Path |
|---|---|
| GET | `/conversations/:id/messages` — paginated |
| POST | `/conversations/:id/messages` — send (→ BullMQ) |

### Contacts
| Method | Path |
|---|---|
| GET | `/contacts` |
| PATCH | `/contacts/:id` |

### Templates
| Method | Path |
|---|---|
| GET | `/templates` |
| POST | `/templates` |
| PATCH | `/templates/:id` |

### AI
| Method | Path |
|---|---|
| POST | `/ai/draft` — generate draft |
| POST | `/ai/drafts/:id/approve` |
| POST | `/ai/drafts/:id/reject` |

### Agents
| Method | Path |
|---|---|
| POST | `/agents/register` |
| GET | `/agents` |

### Webhooks
| Method | Path |
|---|---|
| POST | `/webhooks/zalo-oa` |

---

## 7. BullMQ Queues

| Queue | Producer | Consumer |
|---|---|---|
| `message:ingest` | Agent WS, Webhooks | Xử lý tin nhắn đến |
| `ai:draft` | Ingest worker | Tạo AI draft reply |
| `message:send` | REST API, Approve | Gửi tin qua Agent |
| `notification:push` | Ingest worker | Push notification mobile |

---

## 8. Tech Stack Summary

| Layer | Technology |
|---|---|
| **Agent** | Rust, axum 0.7, tokio, AX API (Mac), Win32 (Windows), embedded webview |
| **Backend** | Hono, Node.js, Drizzle ORM |
| **Database** | PostgreSQL (Supabase) |
| **Cache/Queue** | Redis + BullMQ |
| **AI** | Groq API, Llama 4 Scout, Whisper |
| **Web** | Next.js 14, Tailwind CSS, Zustand |
| **Mobile** | React Native (Expo) |
| **Webview** | Embedded webview trong Rust Agent (chat.zalo.me) — thay Chrome Extension |
| **Hosting** | Vercel (web), Railway (API), Supabase (DB) |

---

## 9. Zalo Web Approach — Embedded Webview

Thay vì phụ thuộc browser bên ngoài (Chrome/Safari/Edge), Rust Agent embed **webview** chạy chat.zalo.me:

### Lợi ích:
- **Tọa độ cố định** — ô search, ô chat luôn ở cùng vị trí, không phụ thuộc browser
- **Không cần Extension** — Agent control webview trực tiếp qua DOM inject
- **Cross-platform** — webview hoạt động trên cả Mac + Windows
- **1 app duy nhất** — user chỉ cần cài Rust Agent, không cần mở browser riêng
- **Đọc DOM trực tiếp** — Agent inject JS vào webview đọc tin nhắn real-time

### Rust Webview Stack:
- `wry` hoặc `tauri` — Rust webview library
- WebKit (Mac) / WebView2 (Windows) — native webview engine
- JS inject — Agent inject đọc/gửi script vào chat.zalo.me
- Session persist — lưu cookie Zalo Web để không phải login lại

### So sánh approaches:

| | Zalo Desktop + AX API | Webview (chat.zalo.me) | Browser Extension |
|---|---|---|---|
| Đọc tin nhắn | AX API (OS-level) | DOM inject (JS) | DOM (isolated world) |
| Gửi tin nhắn | AppleScript paste | JS inject + DOM | Cần Agent relay |
| Tọa độ cố định | N/A | ✅ | Phụ thuộc browser |
| Cross-platform | Mac + Windows | Mac + Windows | Chỉ Chrome |
| Cài đặt | Agent + Zalo Desktop | Chỉ Agent | Agent + Extension |
| Detect risk | Rất thấp | Thấp | Thấp |

---

## 10. Validated (2026-03-18)

| Test | Kết quả |
|---|---|
| AX API đọc Zalo Desktop (cá nhân, group, OA) | ✅ |
| Chrome Extension đọc Zalo Web DOM | ✅ |
| Agent gửi tin Zalo Desktop (AppleScript) | ✅ |
| Agent gửi tin Zalo Web — Chrome (auto-click + paste) | ✅ |
| Agent gửi tin Zalo Web — Safari | ✅ |
| Agent gửi tin Zalo Web — Edge | ✅ |
| Agent tự tìm user theo tên + gửi | ✅ |
| Clipboard paste cho tiếng Việt có dấu | ✅ |
| Zalo không detect được (OS-level input) | ✅ |

### AX API Structure (Zalo Desktop - Mac)
```
depth 18: AXStaticText → Timestamp (15:57)
depth 21: AXStaticText → Tên người gửi (Phan Trung Kiên)
depth 22: AXStaticText → Nội dung tin nhắn
depth 22: AXLink      → @mention
depth 22: AXImage      → Hình ảnh (desc chứa filename)
```

### Zalo Web DOM Classes
```
.truncate          → Tên người gửi
.text              → Nội dung tin nhắn
.card-send-time__sendTime → Thời gian
```

---

## 12. Phase Roadmap

| Phase | Thời gian | Scope |
|---|---|---|
| **Phase 1 MVP** | 8 tuần | Rust Agent (AX API + Webview) + Inbox + AI Draft + Template + Mobile |
| **Phase 2 Intelligence** | Tháng 3-4 | Voice Report + Analytics + Training + Chatbot |
| **Phase 3 Platform** | Tháng 5-6 | REST API + MCP + SDK + Messenger + White-label |
