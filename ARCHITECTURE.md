# HAVIZ Architecture
**Revenue Intelligence Platform for Vietnam**

*"Biến mỗi cuộc trò chuyện thành doanh thu"*

> **Core Principle: LOCAL-FIRST** — Tin nhắn không bao giờ rời khỏi máy user.
> Haviz bán CÔNG CỤ, không bán DATA.

---

## 1. Tổng quan hệ thống (Local-first Architecture)

```
┌──────────────────────────────────────────────────────────────────────┐
│                    USER'S MACHINE (data owner)                       │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │  Rust Desktop Agent (localhost:9999)                            │  │
│  │  ┌────────────┐ ┌────────────┐ ┌───────────────────────────┐  │  │
│  │  │ SQLite DB  │ │ AI Engine  │ │ Channel Adapters          │  │  │
│  │  │ (Tầng 3)   │ │ Anonymize  │ │ Zalo Desktop (AX API)     │  │  │
│  │  │ Full msgs  │ │ → Call AI  │ │ Zalo Webview (DOM)        │  │  │
│  │  │ PII raw    │ │ → Draft    │ │ Zalo OA (Official API)    │  │  │
│  │  │ Attachments│ │            │ │ Safety Engine (5 layers)  │  │  │
│  │  └──────┬─────┘ └──────┬─────┘ └───────────────────────────┘  │  │
│  │         │              │                                       │  │
│  │    Local API      Groq API                                     │  │
│  │   :9999/api     (anonymized)                                   │  │
│  └─────┬──────────────────────────────────────────────────────────┘  │
│        │                                                             │
│  ┌─────▼──────────────────────────────────────────────────────────┐  │
│  │  Web UI (served from Agent, localhost:9999)                     │  │
│  │  Inbox │ Templates │ AI Drafts │ Settings                       │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
└──────────┬──────────────────────────────────────────────────────────┘
           │ E2E encrypted sync (Tầng 2)
           │ + non-personal data (Tầng 1)
           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    HAVIZ CLOUD (không có raw messages)               │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Tầng 1: Public Data (không chứa personal data)              │   │
│  │  ├─ Templates (generic, team sharing)                         │   │
│  │  ├─ Org settings, channel config, user accounts              │   │
│  │  ├─ AI style profiles (patterns, không chứa tin nhắn gốc)   │   │
│  │  ├─ Analytics aggregated (số tin/ngày, response time)        │   │
│  │  └─ Health scores, safety audit logs                         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Tầng 2: Encrypted Data (E2E — Haviz KHÔNG ĐỌC ĐƯỢC)        │   │
│  │  ├─ Conversation metadata (tên, last activity)               │   │
│  │  ├─ Contact list (tên, tags — PII masked)                    │   │
│  │  ├─ Message previews (encrypted)                             │   │
│  │  └─ AI draft status (pending/approved/rejected)              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Tech: Hono API + PostgreSQL (VN cloud) + Redis + BullMQ           │
│  Auth: Supabase Auth                                                │
│  Webhooks: Zalo OA                                                  │
└─────────────────────────────────────────────────────────────────────┘
           │
     ┌─────┴─────┐
     ▼           ▼
┌─────────┐ ┌──────────┐
│ Web App │ │ Mobile   │
│ Remote  │ │ App      │
│ access  │ │ Approve  │
│ (Tầng   │ │ drafts   │
│  1 + 2) │ │ on-the-go│
└─────────┘ └──────────┘
```

### 3 Tầng Data — Phân loại theo mức độ nhạy cảm

| Tầng | Lưu ở đâu | Mã hóa | Chứa gì | Ai đọc được |
|---|---|---|---|---|
| **Tầng 1** | Haviz Cloud | Không cần | Templates, settings, analytics | Haviz + User |
| **Tầng 2** | Haviz Cloud | E2E (AES-256) | Metadata, contact list, previews | Chỉ User (decrypt trên device) |
| **Tầng 3** | Local SQLite | AES-256 | Full messages, PII, attachments | Chỉ User (trên máy) |

### Web App + Mobile access

```
Cùng mạng (LAN):     Phone → 192.168.x.x:9999 → Agent → Full access (Tầng 1+2+3)
Remote (ngoài):       Phone → Haviz Cloud → Tầng 1+2 (templates, metadata, approve drafts)
                      Full messages → cần VPN hoặc user opt-in encrypted sync
Manager dashboard:    Cloud → Analytics, KPIs, workload (KHÔNG thấy nội dung tin nhắn)
```

---

## 2. Monorepo Structure

```
haviz/
├── agent/                          # Rust Desktop Agent
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                 # Entry: tray icon + event loop
│   │   ├── config.rs               # Server URL, auth token, encryption keys
│   │   ├── server.rs               # Axum HTTP+WS server :9999 (serves web UI)
│   │   ├── ws_client.rs            # WS client → Haviz cloud (Tầng 1+2 sync)
│   │   ├── db.rs                   # Local SQLite (Tầng 3 — full messages, PII)
│   │   ├── crypto.rs              # E2E encryption (AES-256-GCM) cho Tầng 2
│   │   ├── anonymizer.rs         # PII removal trước khi gọi AI API
│   │   ├── sync.rs               # Cloud sync engine (Tầng 1: plain, Tầng 2: E2E)
│   │   ├── polling.rs              # Poll Zalo mỗi 3s cho tin nhắn mới
│   │   ├── message_parser.rs       # Parse raw AX text → structured message
│   │   ├── tray.rs                 # System tray icon + menu
│   │   ├── webview.rs              # Embedded webview (chat.zalo.me)
│   │   ├── safety.rs              # 5-layer safety engine
│   │   ├── health.rs             # Account health scoring + auto-throttle
│   │   ├── broadcast.rs          # Anti-broadcast detection
│   │   ├── fallback.rs           # 4-priority channel fallback
│   │   ├── session.rs            # Session monitor + recovery
│   │   ├── dedup.rs              # Multi-device duplicate detection
│   │   ├── analytics.rs         # Local metrics computation + AI insights
│   │   ├── updater.rs            # OTA auto-update
│   │   ├── channels/
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs           # ChannelReader + ChannelSender traits
│   │   │   ├── zalo_desktop.rs     # Zalo Desktop automation (local, AX API)
│   │   │   ├── zalo_webview.rs     # Zalo Web via embedded webview (local)
│   │   │   └── registry.rs         # Multi-account channel registry + config
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
│   │   │   │   ├── conversations.ts  # Unified (local + cloud)
│   │   │   │   ├── messages.ts       # Cloud messages + proxy to Agent
│   │   │   │   ├── contacts.ts       # Multi-channel contacts + merge
│   │   │   │   ├── channels.ts
│   │   │   │   ├── templates.ts
│   │   │   │   ├── ai.ts
│   │   │   │   ├── analytics.ts      # Dashboard, metrics, insights
│   │   │   │   ├── agents.ts
│   │   │   │   └── webhooks/         # Cloud channel webhooks
│   │   │   │       ├── zalo-oa.ts
│   │   │   │       ├── messenger.ts
│   │   │   │       ├── telegram.ts
│   │   │   │       └── phone.ts
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

## 3. Data Flow: Local-first Pipeline

```
STEP 1: POLLING (Agent — local)
────────────────────────────────
Rust Agent poll Zalo Desktop mỗi 3 giây:
  macOS: AX API traverse hierarchy
  → Tìm AXStaticText depth 22 (nội dung), depth 21 (tên), depth 18 (time)
  → Parse: "Phan Trung Kiên" + "E chào chị" + "15:57"
  → So sánh với state cũ (in-memory hash) → detect tin mới

STEP 2: LOCAL STORAGE (Agent SQLite — Tầng 3)
──────────────────────────────────────────────
Tin nhắn mới → lưu LOCAL SQLite trên máy user:
  INSERT INTO messages (conversation_id, sender, content, time, direction)
  → Data KHÔNG rời khỏi máy user
  → Update conversation metadata (last_message, unread_count)

STEP 3: CLOUD SYNC (encrypted — Tầng 1+2)
──────────────────────────────────────────
Agent sync metadata lên Haviz Cloud:
  Tầng 1 (plain): { conversation_count++, last_activity: "15:57" }
  Tầng 2 (E2E encrypted): { contact: "P***n", preview: encrypt("E chào...") }
  → Cloud KHÔNG nhận được nội dung tin nhắn gốc

STEP 4: AI DRAFT (Agent — local + AI API)
──────────────────────────────────────────
4a. Template matching (local, không tốn AI cost):
  - So sánh tin nhắn inbound với templates (sync từ Cloud Tầng 1)
  - Nếu match → dùng template luôn, KHÔNG gọi AI
  - Ví dụ: "giá bao nhiêu" → template báo giá
  - Tiết kiệm ~50% số lần gọi AI

4b. Nếu không match template → gọi AI:
  Agent thực hiện LOCAL:
  1. Load style profile (cached local, ~50 tokens)
  2. ANONYMIZE context trước khi gửi AI:
     - "Phan Trung Kiên" → "Customer"
     - "0912345678" → [removed]
     - Giữ ngữ cảnh business
  3. Load smart context (3-5 tin nhắn, anonymized, ~200 tokens)
  4. Call Groq API (Llama 4 Scout):
     System prompt = style profile + org context
     Messages = anonymized context
     → Groq KHÔNG nhận được tên, SĐT, thông tin cá nhân
  5. Nhận draft → fill lại tên thật từ Local SQLite
  6. Lưu draft vào Local SQLite (status: pending)
  7. Sync draft status lên Cloud Tầng 2 (encrypted)

Chi phí AI (giữ nguyên — không thay đổi):
  - 200 tokens/request + 50% template match
  - 1 sales: $0.05/tháng | 10 sales: $0.5/tháng | 50 sales: $2.5/tháng

STEP 5: HIỂN THỊ (Web UI — served from Agent)
──────────────────────────────────────────────
  Cùng máy: localhost:9999 → đọc trực tiếp Local SQLite (Tầng 1+2+3)
  Cùng LAN: 192.168.x.x:9999 → Agent serve data (Tầng 1+2+3)
  Remote: Haviz Cloud → Tầng 1+2 only (metadata + encrypted previews)

  - Inbox cập nhật real-time
  - AI Draft Panel hiện gợi ý reply
  - User chọn: Approve | Edit | Reject

STEP 6: APPROVE → GỬI
──────────────────────
User click Approve (từ localhost hoặc remote):
  Local: POST localhost:9999/api/drafts/:id/approve
  Remote: POST cloud → WebSocket push → Agent nhận command
  → Agent update Local SQLite (direction: outbound, status: pending)

STEP 7: GỬI (Agent → Zalo) — có Safety Layer
──────────────────────────────────────────────
Rust Agent thực hiện LOCAL:
  7a. Safety checks:
    - Rate limit check (< 5 tin/phút/conversation?)
    - Working hours check (7:00-22:00?)
    - Duplicate check (tin giống nhau trong 5 phút?)
    - Nếu fail → queue + thông báo user
  7b. Human-like execution:
    macOS: Cmd+F → [random 0.5-1.5s] → gõ tên → Enter
           → [random 0.3-0.8s] → Cmd+V paste
           → [random 0.2-0.6s] → Enter
  7c. Fallback nếu fail:
    Zalo Desktop → Webview → queue thủ công
  → Update Local SQLite: status=sent
  → Sync status lên Cloud Tầng 2 (encrypted)
```

### AI Anonymization Flow

```
Local SQLite (raw)                    Gửi đến Groq API (anonymized)
───────────────────                   ──────────────────────────────
"Phan Trung Kiên:                     "Customer:
 E chào chị, cho e xin               Xin chào, hỏi về báo giá
 báo giá tour Đà Lạt,                sản phẩm X"
 SĐT 0912345678"

Agent anonymizer.rs:
  1. Regex detect PII: SĐT, CCCD, email, tên riêng
  2. Replace tên → "Customer" / "User"
  3. Remove SĐT, email, CCCD, địa chỉ cụ thể
  4. Giữ: sản phẩm, câu hỏi, ngữ cảnh business
  5. Gửi anonymized context → Groq API
  6. Nhận draft → fill lại tên thật từ Local DB
```

---

## 4. Database Schema (Local-first: 2 databases)

### 4.1 Agent Local SQLite (Tầng 3 — trên máy user)

```sql
-- Tin nhắn gốc (KHÔNG rời khỏi máy)
messages         (id, conversation_id, direction, sender_name, content,
                  content_type, metadata_json, status, zalo_timestamp, created_at)
                 -- direction: inbound | outbound
                 -- status: received | draft | pending | sent | failed

-- Hội thoại (local)
conversations    (id, contact_id, channel_type, last_message_at,
                  last_message_preview, unread_count, status)

-- Liên hệ (local, full PII)
contacts         (id, display_name, phone, zalo_id, tags, metadata_json)

-- AI Draft (local)
ai_drafts        (id, conversation_id, trigger_message_id, content,
                  model, status, approved_at, edited_content)
                 -- status: pending | approved | rejected | edited

-- Style Profile (local cache)
style_profiles   (id, profile_json, sample_count, last_analyzed_at)

-- Attachments (local file paths)
attachments      (id, message_id, file_path, file_type, file_size, downloaded_at)

-- Sync state (track đã sync gì lên Cloud)
sync_log         (id, entity_type, entity_id, synced_at, sync_tier)
                 -- sync_tier: 1 (plain) | 2 (encrypted)
```

### 4.2 Haviz Cloud PostgreSQL (Tầng 1+2)

```sql
-- === TẦNG 1: Plain data (không chứa personal data) ===

-- Tổ chức (multi-tenant)
organizations    (id, name, slug, plan, settings)

-- Users (liên kết Supabase Auth)
users            (id, auth_id, org_id, email, full_name, role, avatar_url)

-- Kênh kết nối
channels         (id, org_id, type, name, status, config, source, agent_id)
                 -- type: zalo_personal | zalo_oa | messenger | telegram | phone
                 -- source: 'local' (Agent) | 'cloud' (webhook)
                 -- agent_id: NOT NULL nếu source='local'

-- Desktop agents
agents           (id, org_id, user_id, name, platform, version, status,
                  auth_token, last_seen_at)

-- Templates (generic, team sharing)
templates        (id, org_id, name, content, category, variables, usage_count,
                  match_patterns, auto_match)
                 -- match_patterns: ["giá bao nhiêu", "báo giá", "price"]

-- Style Profile (chỉ patterns, KHÔNG chứa tin nhắn gốc)
style_profiles   (id, user_id, org_id, profile_json, sample_count, last_analyzed_at)
                 -- profile_json: { xung_ho, viet_tat[], emoji_style[],
                 --   do_dai, giong_dieu, dac_diem[] }

-- Analytics: metrics hàng ngày từ Agent (chỉ con số)
daily_metrics    (id, org_id, user_id, date DATE,
                  messages_inbound INT, messages_outbound INT,
                  avg_response_time_seconds INT, first_response_time_seconds INT,
                  ai_drafts_total INT, ai_drafts_approved INT,
                  ai_drafts_edited INT, ai_drafts_rejected INT,
                  templates_used INT, conversations_active INT,
                  conversations_new INT, busiest_hour INT)

-- Analytics: AI insights hàng ngày (kết quả phân tích, không raw data)
daily_insights   (id, org_id, user_id, date DATE,
                  top_topics JSONB,           -- ["báo giá", "ship", "bảo hành"]
                  sentiment JSONB,            -- { positive: 60, neutral: 30, negative: 10 }
                  common_questions JSONB,     -- ["giá bao nhiêu", "ship bao lâu"]
                  suggested_actions JSONB,    -- ["update template X"]
                  quality_score INT)          -- 0-100

-- Analytics: template effectiveness
template_analytics (id, org_id, template_id, date DATE,
                    times_used INT, times_edited INT,
                    avg_response_rate FLOAT)

-- Account health
account_health   (id, user_id, org_id, score, status,
                  messages_sent_today, messages_failed_today)

-- Safety audit log
safety_audit_log (id, org_id, user_id, event_type, details_json, created_at)

-- === TẦNG 2: E2E Encrypted (Haviz server KHÔNG đọc được) ===

-- Encrypted conversation metadata
encrypted_conversations (id, org_id, user_id,
                         encrypted_blob BYTEA,    -- AES-256-GCM encrypted JSON:
                         -- { contact_name, last_preview, unread_count, status }
                         nonce BYTEA, updated_at)

-- Encrypted contact list
encrypted_contacts (id, org_id, user_id,
                    encrypted_blob BYTEA,         -- AES-256-GCM encrypted JSON:
                    -- { display_name, phone_masked, tags }
                    nonce BYTEA, updated_at)

-- AI draft status (encrypted content, plain status)
encrypted_drafts (id, org_id, user_id, conversation_ref TEXT,
                  status TEXT,                    -- pending | approved | rejected
                  encrypted_content BYTEA,        -- AES-256-GCM encrypted draft text
                  nonce BYTEA, created_at)

-- === MULTI-CHANNEL: Unified Inbox ===

-- Contact-to-channel mapping (1 khách hàng nhiều kênh)
contact_channels (id, contact_id, channel_type TEXT, channel_source TEXT,
                  external_id TEXT, external_name TEXT, agent_id TEXT,
                  is_primary BOOLEAN, linked_at TIMESTAMP)
                 -- channel_source: 'local' | 'cloud'
                 -- agent_id: NOT NULL nếu local channel

-- Unified conversation view (cả local + cloud)
unified_conversations (id, org_id, contact_id, channel_type TEXT,
                       channel_source TEXT, agent_id TEXT,
                       status TEXT, assigned_to TEXT,
                       last_activity_at TIMESTAMP,
                       last_preview_encrypted BYTEA,  -- E2E nếu local
                       last_preview_plain TEXT,        -- Plain nếu cloud
                       unread_count INT, tags TEXT[], priority TEXT)

-- Cloud messages (CHỈ cho cloud channels: OA, Messenger, Telegram, Phone)
cloud_messages   (id, conversation_id, org_id, direction TEXT,
                  sender_type TEXT, sender_id TEXT,
                  content TEXT, content_type TEXT,
                  metadata JSONB, status TEXT,
                  channel_type TEXT, sent_at TIMESTAMP)
                 -- Note: Zalo personal messages ở Agent SQLite, KHÔNG ở đây

-- Phase 2
voice_reports    (id, org_id, user_id, encrypted_transcript BYTEA, nonce BYTEA)
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
| `agent:config` | `{ pollInterval, channels, rateLimits, activeHours }` |

### Analytics Events
| Event | Direction | Payload |
|---|---|---|
| `analytics:metrics` | Agent → Server | `{ date, metrics: { messages_in, messages_out, ... } }` |
| `analytics:insights` | Agent → Server | `{ date, top_topics, sentiment, suggestions }` |

### Safety Events
| Event | Direction | Payload |
|---|---|---|
| `rate:exceeded` | Agent → Server | `{ conversationId, limit, window }` |
| `send:queued` | Agent → Server | `{ messageId, reason: "outside_hours\|rate_limit", eta }` |
| `send:fallback` | Agent → Server | `{ messageId, from: "zalo_desktop", to: "webview", reason }` |

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

### Webhooks (Cloud Channels)
| Method | Path |
|---|---|
| POST | `/webhooks/zalo-oa` — Zalo OA messages + events |
| POST | `/webhooks/messenger` — Facebook Messenger (Graph API) |
| POST | `/webhooks/telegram` — Telegram Bot API updates |
| POST | `/webhooks/phone` — Phone/SMS (Twilio/VNPT) |

### Contacts (Multi-Channel)
| Method | Path |
|---|---|
| GET | `/contacts` — list (merged across channels) |
| GET | `/contacts/:id` — detail + all linked channels |
| PATCH | `/contacts/:id` — update |
| POST | `/contacts/merge` — merge 2 contacts |
| GET | `/contacts/:id/channels` — list linked channels |
| POST | `/contacts/:id/channels` — link new channel |

### Analytics (Cloud — chỉ nhận metrics/insights từ Agent)
| Method | Path |
|---|---|
| POST | `/analytics/metrics` — Agent gửi metrics hàng giờ |
| POST | `/analytics/insights` — Agent gửi AI insights hàng ngày |
| GET | `/analytics/dashboard?org_id=xxx` — Manager dashboard |
| GET | `/analytics/team?org_id=xxx&range=7d` — Team comparison |
| GET | `/analytics/templates?org_id=xxx` — Template effectiveness |
| GET | `/analytics/trends?user_id=xxx&range=30d` — Individual trends |

### Safety & Health
| Method | Path |
|---|---|
| GET | `/health` — account health score + status |
| GET | `/health/history` — health score over time |
| GET | `/safety/audit` — safety audit log |
| GET | `/safety/broadcast-log` — broadcast detection log |

### Data Export
| Method | Path |
|---|---|
| GET | `/export/conversations?format=csv\|json` |
| GET | `/export/messages?conversation_id=xxx&format=csv\|json` |
| GET | `/export/contacts?format=csv\|json` |

---

## 7. BullMQ Queues

| Queue | Producer | Consumer |
|---|---|---|
| `message:ingest` | Cloud webhooks (OA, Mess, Tele) | Xử lý tin nhắn cloud channels |
| `ai:draft` | Ingest worker | Tạo AI draft (cloud channels) |
| `ai:draft:local` | Agent | Agent tự tạo draft local (Zalo personal) |
| `message:send` | REST API, Approve | Gửi tin qua Agent hoặc Cloud API |
| `message:scheduled` | Safety engine | Tin nhắn chờ gửi (ngoài giờ, rate limited) |
| `contact:merge` | User action | Merge duplicate contacts across channels |
| `notification:push` | Ingest worker | Push notification mobile |
| `health:check` | Cron (mỗi 5 phút) | Cập nhật health score |
| `analytics:aggregate` | Agent (mỗi giờ) | Nhận metrics từ Agent, aggregate |
| `analytics:insights` | Agent (mỗi ngày) | Nhận AI insights từ Agent |
| `backup:daily` | Cron (2:00 AM) | Encrypted backup → S3/R2 |

---

## 8. AI Pipeline

### 8.1 Smart Draft Flow

```
Tin nhắn inbound đến
        │
        ▼
┌─ Template Match? ──────────────────────────┐
│  So sánh với match_patterns trong templates │
│  "giá bao nhiêu" → match template báo giá  │
└────┬──────────────────┬────────────────────┘
     │ Match            │ Không match
     ▼                  ▼
  Dùng template     Gọi Groq AI
  (0 tokens,        (200 tokens,
   0 cost)           ~$0.00003)
     │                  │
     ▼                  ▼
  AiDraft           AiDraft
  (pending)         (pending)
     │                  │
     └────────┬─────────┘
              ▼
     User: Approve / Edit / Reject
```

### 8.2 Style Profile (cache, phân tích 1 lần)

```
Lần đầu (khi user kết nối):
  50+ tin nhắn outbound cũ
        │
        ▼
  Groq AI phân tích → JSON style profile:
  {
    "xung_ho": "anh/em",
    "viet_tat": ["e", "kk", "dc", "cty"],
    "emoji_style": [":v", "=))))"],
    "do_dai": "ngắn",
    "giong_dieu": "thân mật, hài hước",
    "dac_diem": ["hay đùa", "trêu đồng nghiệp"],
    "vi_du_cau": ["chắc nhậu ít quá nên bị =))))"]
  }
        │
        ▼
  Lưu DB (style_profiles table)
  Tự phân tích lại khi +50 tin nhắn mới

Các lần sau:
  Load style profile từ DB (~50 tokens)
  Inject vào system prompt
  → AI reply đúng style user
```

### 8.3 Smart Context Window

```
Thay vì gửi 20 tin nhắn (~800 tokens):

System prompt (~100 tokens):
  - Style profile (cached)
  - Org context (1 dòng)

Messages (~100 tokens):
  - 3-5 tin nhắn gần nhất
  - Contact metadata (1 dòng: "khách mới, hỏi giá tour")

Tổng: ~200 tokens/request (giảm 75%)
```

### 8.4 Chi phí ước tính

| Quy mô | Không tối ưu | Có tối ưu | Tiết kiệm |
|---|---|---|---|
| 1 sales | $0.5/tháng | $0.05/tháng | 90% |
| 10 sales | $5/tháng | $0.5/tháng | 90% |
| 50 sales | $25/tháng | $2.5/tháng | 90% |

### 8.5 Local Analytics Pipeline (Agent → AI → Cloud Dashboard)

Tất cả analytics đều tính toán LOCAL trên Agent. Cloud chỉ nhận **con số và kết quả**, không raw data.

#### Tầng 1: Metrics đơn giản — Agent đếm, Cloud hiển thị

```
Agent trên máy salesperson tự đếm mỗi giờ:
  {
    messages_in: 45,
    messages_out: 38,
    avg_response_time: 127,       // giây
    first_response_time: 45,
    ai_drafts: { total: 35, approved: 28, edited: 5, rejected: 2 },
    templates_used: 12,
    conversations_active: 15,
    conversations_new: 6,
    busiest_hour: 14
  }
       │
       │ Gửi lên Cloud: CHỈ SỐ, không nội dung tin nhắn
       ▼
  Cloud Dashboard → hiển thị bảng, biểu đồ, so sánh team
```

#### Tầng 2: Smart Insights — Agent gọi AI (anonymized), gửi KẾT QUẢ lên Cloud

```
Agent LOCAL mỗi ngày:
  1. Đọc tin nhắn từ SQLite (local)
  2. ANONYMIZE: xóa tên, SĐT, PII
  3. Gọi AI (Groq) với prompt:
     "Phân tích các tin nhắn này. Trả về JSON:
      { top_topics, sentiment, common_questions,
        suggested_actions, quality_score }"
  4. AI trả về structured JSON (không raw data)
  5. Agent gửi JSON kết quả lên Cloud

  AI nhận: tin nhắn anonymized (không tên, không SĐT)
  Cloud nhận: chỉ kết quả phân tích (JSON)
  Không ai nhận: nội dung tin nhắn gốc
```

#### Data Flow hoàn chỉnh

```
Mỗi Agent (trên máy từng salesperson)
       │
       ├── Mỗi giờ: tính metrics → gửi số lên Cloud
       ├── Mỗi ngày: AI phân tích (anonymized) → gửi insights lên Cloud
       └── Mỗi tuần: weekly summary
       │
       ▼
┌─ Cloud nhận được ──────────────────────────────┐
│                                                 │
│  ✅ metrics: { messages: 45, response_time: 2m }│
│  ✅ insights: { topics: ["báo giá", "ship"] }  │
│  ✅ sentiment: { positive: 60%, negative: 10% } │
│  ✅ suggestions: ["update template X"]          │
│                                                 │
│  ❌ KHÔNG CÓ: tên khách, SĐT, nội dung tin     │
│                                                 │
└─────────────────────────────────────────────────┘
       │
       ▼
┌─ Manager Dashboard ────────────────────────────┐
│                                                 │
│  ┌─────────────┬────────┬────────┬───────────┐ │
│  │ Salesperson │ Tin/ngày│ Avg RT │ AI Accept │ │
│  ├─────────────┼────────┼────────┼───────────┤ │
│  │ Nguyễn A    │  45/38 │  2m07s │    80%    │ │
│  │ Trần B      │  62/55 │  1m15s │    92%    │ │
│  │ Lê C        │  28/20 │  5m42s │    65%    │ │
│  └─────────────┴────────┴────────┴───────────┘ │
│                                                 │
│  Top chủ đề tuần này:                           │
│    1. Hỏi giá (35%)  2. Ship (25%)             │
│    3. Khiếu nại (15%)  4. Bảo hành (12%)       │
│                                                 │
│  💡 Gợi ý:                                      │
│  - Template "báo giá" cần cập nhật giá mới      │
│  - Khiếu nại tăng 15% → review quy trình ship  │
│  - Lê C: response time cao → cần hỗ trợ        │
│                                                 │
└─────────────────────────────────────────────────┘
```

#### Báo cáo nào CẦN vs KHÔNG CẦN raw data

| Report | Cần raw data? | Cách làm local-first |
|---|---|---|
| Số tin nhắn/ngày | Không | Agent đếm → gửi số |
| Response time | Không | Agent tính → gửi số |
| AI draft accept rate | Không | Agent tính → gửi % |
| Template usage ranking | Không | Agent đếm → gửi count |
| Top chủ đề khách hỏi | Cần context | Agent anonymize → AI → gửi topics |
| Sentiment analysis | Cần context | Agent anonymize → AI → gửi % |
| Gợi ý template mới | Cần context | Agent anonymize → AI → gửi text |
| Đọc nội dung conversation | Cần raw | **CHỈ local/LAN** — không lên Cloud |

---

## 8.6 Multi-Channel Architecture (Unified Inbox)

### Nguyên tắc phân loại kênh

| Kênh | Tính chất | Data ở đâu | Consent | Rủi ro |
|---|---|---|---|---|
| **Zalo cá nhân** | Chat riêng tư | **Local** (Agent) | Cần mechanism | Thấp (local-first) |
| **Zalo OA** | Business context | **Cloud** | Follow = consent | **0%** (official API) |
| **Messenger** | Business Page | **Cloud** | Nhắn Page = consent | **0%** (Graph API) |
| **Telegram** | Bot API | **Cloud** | Nhắn Bot = consent | **0%** (Bot API) |
| **Phone/SMS** | Business number | **Cloud** | Gọi đến công ty | **0%** (Twilio/VNPT) |

> Kênh nào user **chủ động liên hệ doanh nghiệp** → Cloud OK.
> Kênh nào là **chat cá nhân** → Local.

### Hybrid Channel Architecture

```
┌─ LOCAL CHANNELS (Agent) ────────────────────────────────────────┐
│  Tin nhắn riêng tư → Local SQLite → encrypted metadata → Cloud  │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐                     │
│  │ Zalo Personal #1 │  │ Zalo Personal #2 │  (nhiều account)    │
│  │ AX API + Script   │  │ AX API + Script   │                    │
│  └────────┬─────────┘  └────────┬─────────┘                     │
│           └────────┬────────────┘                                │
│                    ▼                                             │
│           Agent SQLite (Tầng 3)                                  │
│           Full messages, local only                              │
└───────────┬──────────────────────────────────────────────────────┘
            │ encrypted metadata (Tầng 2)
            ▼
┌─ HAVIZ CLOUD ───────────────────────────────────────────────────┐
│                                                                  │
│  ┌─ Unified Inbox DB ────────────────────────────────────────┐  │
│  │                                                            │  │
│  │  Zalo Personal → encrypted metadata only (từ Agent)       │  │
│  │  Zalo OA      → full messages ✅ (official webhook)       │  │
│  │  Messenger    → full messages ✅ (Graph API webhook)      │  │
│  │  Telegram     → full messages ✅ (Bot API webhook)        │  │
│  │  Phone/SMS    → full messages ✅ (Twilio/VNPT webhook)    │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Cloud Channels (webhook → server):                              │
│    Zalo OA ──── POST /webhooks/zalo-oa ──► message:ingest        │
│    Messenger ── POST /webhooks/messenger ─► message:ingest       │
│    Telegram ─── POST /webhooks/telegram ──► message:ingest       │
│    Phone ────── POST /webhooks/phone ─────► message:ingest       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Unified Inbox View

```
┌─ Unified Inbox ─────────────────────────────────────────────────┐
│                                                                  │
│  ┌──────────┬──────────────────┬───────────┬──────────────────┐ │
│  │ Channel  │ Contact          │ Preview   │ Source            │ │
│  ├──────────┼──────────────────┼───────────┼──────────────────┤ │
│  │ 💬 Zalo  │ Nguyễn Văn A     │ 🔒 E2E   │ Local (Agent #1) │ │
│  │ 📘 OA    │ Trần Thị B       │ Hỏi giá  │ Cloud            │ │
│  │ 💙 Mess  │ Lê Văn C         │ OK ship   │ Cloud            │ │
│  │ ✈️ Tele  │ @customer_d      │ Cảm ơn   │ Cloud            │ │
│  │ 💬 Zalo  │ Phạm E           │ 🔒 E2E   │ Local (Agent #2) │ │
│  │ 📞 Phone │ 0912***678       │ Gọi lại  │ Cloud            │ │
│  └──────────┴──────────────────┴───────────┴──────────────────┘ │
│                                                                  │
│  Click Zalo Personal:                                            │
│  ├─ Agent online + cùng LAN → Full messages từ Agent ✅         │
│  ├─ Agent online + remote   → Full messages qua WSS tunnel ✅   │
│  └─ Agent offline           → Chỉ encrypted preview 🔒         │
│     "Cần kết nối Agent để xem tin nhắn Zalo cá nhân"           │
│                                                                  │
│  Click Cloud channels (OA/Messenger/Telegram/Phone):             │
│  → Full messages từ Cloud DB ✅ (luôn available)                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Multi-Account Zalo (1 Agent nhiều account)

```toml
# agent_config.toml

[[channels]]
type = "zalo_personal"
name = "Zalo chính"
app_identifier = "Zalo"              # AX API target window
account_phone = "0912345678"

[[channels]]
type = "zalo_personal"
name = "Zalo phụ"
app_identifier = "Zalo 2"           # Zalo instance 2
account_phone = "0987654321"

[[channels]]
type = "zalo_oa"
name = "OA Công ty XYZ"
oa_id = "oa_12345"
# → webhook qua Cloud, không qua AX API
```

### Contact Merging (1 khách hàng nhiều kênh)

```
Cùng 1 khách nhắn qua nhiều kênh:
  💬 Zalo cá nhân: "Nguyễn Văn A" (0912345678)
  📘 Zalo OA:      "Nguyễn Văn A" (follower)
  💙 Messenger:    "Nguyen Van A" (FB)
  ✈️ Telegram:     "@nguyenvana"

Merge signals:
  1. Cùng SĐT (exact match)
  2. Cùng tên (fuzzy match, Levenshtein distance < 3)
  3. User xác nhận thủ công (drag & merge trong UI)

Kết quả:
┌──────────────────────────────────────────────────┐
│  Contact: Nguyễn Văn A                            │
│  ├── 💬 Zalo cá nhân → Agent (local)              │
│  ├── 📘 Zalo OA      → Cloud                     │
│  ├── 💙 Messenger    → Cloud                     │
│  └── ✈️ Telegram     → Cloud                     │
│                                                    │
│  Unified Timeline:                                 │
│  15:30 💬 Zalo: "Chào chị" (🔒 cần Agent)         │
│  15:35 📘 OA: "Cho e hỏi giá" (cloud ✅)          │
│  16:00 💙 Mess: "Gửi lại link" (cloud ✅)         │
└──────────────────────────────────────────────────┘
```

### Contact Resolution Flow

```
Tin nhắn mới đến (bất kỳ kênh)
       │
       ▼
┌─ Extract identifiers ─────────────────────────────┐
│  phone, name, email, zalo_id, fb_id, tg_id        │
└──────┬────────────────────────────────────────────┘
       │
       ▼
┌─ Search existing contacts ────────────────────────┐
│  Cloud DB: exact match phone / email / external_id │
│  Agent local: fuzzy match tên (Zalo cá nhân)       │
└──────┬──────────────────┬─────────────────────────┘
       │ Found             │ Not found
       ▼                   ▼
  Link channel mới    Tạo contact mới
  vào contact cũ      (local hoặc cloud
  + contact_channels    tùy channel type)
       │                   │
       └────────┬──────────┘
                ▼
┌─ Duplicate detection ─────────────────────────────┐
│  "Nguyễn Văn A (Zalo) có thể là Nguyen Van A     │
│   (Messenger). Merge?"                             │
│  → User confirm → merge contacts                   │
└───────────────────────────────────────────────────┘
```

---

## 9. Tech Stack Summary

| Layer | Technology |
|---|---|
| **Agent** | Rust, axum 0.7, tokio, SQLite, AX API (Mac), Win32 (Windows), webview |
| **Agent Local DB** | SQLite (Tầng 3 — full messages, encrypted AES-256) |
| **Agent Crypto** | AES-256-GCM (E2E encryption cho Tầng 2) |
| **Backend** | Hono, Node.js, Drizzle ORM |
| **Database** | PostgreSQL (VN cloud — Tầng 1+2 + cloud messages) |
| **Cache/Queue** | Redis + BullMQ |
| **AI** | Groq API (anonymized input), Llama 4 Scout, Whisper |
| **Web** | Next.js 14, Tailwind CSS, Zustand |
| **Mobile** | React Native (Expo) |
| **Zalo Web** | chrome-headless-shell + rust-headless-chrome (CDP) — validated |
| **Cloud Channels** | Zalo OA API, FB Graph API, Telegram Bot API, Twilio/VNPT |
| **Hosting** | VN Cloud (API+DB), Vercel (web), Agent on user's machine |

---

## 9.1 Zalo Web Approach — chrome-headless-shell (Validated 2026-03-19)

Agent dùng **chrome-headless-shell** (binary headless riêng của Google) để đọc/gửi Zalo Web qua CDP, điều khiển từ Rust qua `rust-headless-chrome` crate.

### Validated Results

```
Browser: chrome-headless-shell v146.0.7680.80 (macOS arm64)
Target:  chat.zalo.me
Result:  ✅ Full render — QR code hiện, JS 100%, CDP full control
Load:    2.7s to networkidle
RAM:     ~558MB (tất cả chrome processes)
```

### So sánh approaches (validated)

| | AX API (Desktop) | chrome-headless-shell | Lightpanda | Servo |
|---|---|---|---|---|
| Zalo load | ✅ | **✅ 100%** | Fetch OK, CDP fail | **Crash** |
| JS execution | N/A (OS-level) | **V8 full** | V8 full | Partial |
| RAM | **0MB** (no browser) | 558MB | **24MB** | 510MB |
| CDP support | N/A | **Full** | Beta | WebDriver |
| Cookie persist | N/A | **Yes** | No | N/A |
| Rust integration | Command::new | **rust-headless-chrome** | CDP client | N/A |
| Production ready | **Yes** | **Yes** | No | No |
| Detect risk | Rất thấp | Thấp | Thấp | N/A |

### Architecture: Rust Agent + chrome-headless-shell

```
┌─ Rust Agent ──────────────────────────────────────────────┐
│                                                            │
│  channels/zalo_web.rs                                      │
│  ├── Launch chrome-headless-shell (child process)          │
│  ├── Connect via CDP (rust-headless-chrome crate)          │
│  ├── Navigate chat.zalo.me                                 │
│  ├── Inject cookie (từ lần login trước)                    │
│  ├── Poll DOM: evaluate JS → extract messages              │
│  ├── Send: click input → type message → Enter              │
│  └── Cookie persistence: save/load từ local file           │
│                                                            │
│  First-time login flow:                                    │
│  1. Agent mở chrome-headless-shell KHÔNG headless           │
│     (--headless=false) → hiện window QR code                │
│  2. User scan QR trên điện thoại                           │
│  3. Agent detect login success → save cookies               │
│  4. Đóng window → chuyển sang headless mode                │
│  5. Lần sau: load cookies → login tự động                  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Khi nào dùng channel nào

```
User có Zalo Desktop:     → AX API (ưu tiên, 0MB overhead)
User KHÔNG có Desktop:    → chrome-headless-shell (fallback)
Zalo Desktop bị lỗi:     → auto-fallback sang headless-shell
Zalo OA:                  → Cloud webhook (official API)
```

---

## 10. Production Safety & Data Protection

> **Nguyên tắc #1:** Mọi tin nhắn đã vào Haviz = **vĩnh viễn an toàn**, bất kể Zalo có ban hay không.
> **Nguyên tắc #2:** Zalo chỉ ban **hành vi**, không ban **công cụ**. Haviz giả lập đúng hành vi người thật.

---

### 10.1 Data Protection — Local-first, Zalo 100% không xóa được

Tin nhắn được lưu **trên máy user** ngay khi đọc. Haviz Cloud **không có** nội dung tin nhắn gốc.

```
Tin nhắn trên Zalo ──(AX API đọc)──► Agent Local SQLite (Tầng 3)
                                          │
                     Zalo KHÔNG            ├──► Encrypted metadata → Cloud (Tầng 2)
                     kiểm soát            ├──► Local backup (encrypted)
                     được từ đây          └──► Export (CSV/JSON từ Agent)
```

#### 10.1.1 Data ở đâu?

| Data | Lưu ở đâu | Zalo xóa được? | Haviz đọc được? |
|---|---|---|---|
| Nội dung tin nhắn (text) | **Local SQLite** (máy user) | **Không** | **Không** (chỉ user) |
| Tên người gửi | **Local SQLite** | **Không** | **Không** |
| File đính kèm (ảnh, doc) | **Local disk** | **Không** | **Không** |
| Contact list (PII) | **Local SQLite** | **Không** | **Không** |
| Conversation metadata | Cloud Tầng 2 (**E2E encrypted**) | **Không** | **Không** (encrypted) |
| Templates | Cloud Tầng 1 (plain) | N/A | Có (generic, không chứa PII) |
| Analytics aggregated | Cloud Tầng 1 (plain) | N/A | Có (số liệu, không nội dung) |
| AI drafts | **Local SQLite** + Cloud Tầng 2 (encrypted status) | **Không** | **Không** |

#### 10.1.2 Worst Case Scenarios

| Tình huống | Data mất? | Xử lý |
|---|---|---|
| Zalo ban tài khoản | **Không** — data ở Local SQLite | Tin nhắn cũ vẫn đọc được qua Agent UI |
| Zalo xóa tin nhắn | **Không** — đã copy vào SQLite | Agent giữ bản gốc |
| User xóa Zalo | **Không** — SQLite độc lập | Toàn bộ lịch sử vẫn trong Agent |
| Máy user hỏng | **Rủi ro** — data chỉ ở local | **Cần backup**: Agent tự backup encrypted → Cloud/USB |
| Haviz Cloud down | **Không mất tin nhắn** | Mất templates + metadata tạm, Agent vẫn hoạt động offline |
| Mất mạng | **Không** | Agent đọc/gửi local, sync metadata khi online |

**Lưu ý:** Local-first nghĩa là máy user hỏng = mất data nếu không backup. Agent cần tính năng:
- Auto backup encrypted SQLite → Cloud (opt-in, E2E encrypted)
- Manual backup → USB/external drive
- Restore từ backup khi cài máy mới

#### 10.1.3 Data Export (Agent local API)

```
GET localhost:9999/api/export/conversations?format=csv
GET localhost:9999/api/export/messages?conversation_id=xxx&format=json
GET localhost:9999/api/export/contacts?format=csv
```

User export trực tiếp từ Agent — data không đi qua Cloud.

---

### 10.2 Anti-Ban: Tại sao Haviz an toàn

Haviz **không** reverse-engineer Zalo API hay inject code vào Zalo app. Mọi thao tác ở tầng OS:

| Kỹ thuật | Zalo thấy gì? | Rủi ro |
|---|---|---|
| AX API đọc tin nhắn | Không — đọc ở tầng OS accessibility | 0% |
| AppleScript + Clipboard paste | Như người dùng bình thường | Rất thấp |
| Embedded Webview (chat.zalo.me) | Như browser bình thường | Thấp |
| Zalo OA Official API | Chính thức, được Zalo cho phép | 0% |

#### So sánh với các approach nguy hiểm

| Approach | Rủi ro ban | Haviz dùng? |
|---|---|---|
| Reverse-engineer Zalo API (HTTP trực tiếp) | **Rất cao** | **Không** ❌ |
| Inject code vào Zalo app | **Cao** | **Không** ❌ |
| Selenium/Puppeteer automation | **Trung bình** — detectable | **Không** ❌ |
| OS-level AX API + AppleScript | **Rất thấp** — invisible | **Có** ✅ |
| Embedded Webview + DOM inject | **Thấp** — native browser | **Có** ✅ |
| Zalo OA Official API | **0%** | **Có** ✅ |

---

### 10.3 Safety Engine (agent/src/safety.rs)

```rust
// ═══════════════════════════════════════════════════════
// TẦNG 1: RATE LIMITING — giới hạn tốc độ gửi
// ═══════════════════════════════════════════════════════

/// Per-conversation limits
const MAX_MESSAGES_PER_MINUTE: u32 = 5;
const MAX_MESSAGES_PER_HOUR_PER_CONVERSATION: u32 = 30;

/// Global limits (toàn bộ conversations)
const MAX_MESSAGES_PER_HOUR_GLOBAL: u32 = 60;
const MAX_MESSAGES_PER_DAY_GLOBAL: u32 = 300;

/// Khoảng cách tối thiểu giữa 2 tin
const MIN_BETWEEN_MESSAGES_MS: u64 = 3000;

// ═══════════════════════════════════════════════════════
// TẦNG 2: HUMAN-LIKE BEHAVIOR — giả lập người thật
// ═══════════════════════════════════════════════════════

/// Random delays giữa các thao tác (milliseconds)
const SEARCH_TO_CLICK_DELAY: (u64, u64) = (800, 2000);    // Người thật: ~1-2s
const CLICK_TO_PASTE_DELAY: (u64, u64) = (500, 1200);     // Người thật: ~0.5-1s
const PASTE_TO_SEND_DELAY: (u64, u64) = (300, 800);       // Người thật: ~0.3-0.8s
const TYPING_SIMULATION_MS: (u64, u64) = (2000, 5000);    // "Đang soạn tin..." 2-5s
const BETWEEN_CONVERSATIONS_MS: (u64, u64) = (3000, 8000); // Chuyển conversation: 3-8s

/// Variance — không bao giờ gửi đều đặn
/// Random ±30% cho mỗi delay, tránh pattern detection
const JITTER_PERCENT: f64 = 0.30;

// ═══════════════════════════════════════════════════════
// TẦNG 3: WORKING HOURS — chỉ hoạt động giờ hợp lý
// ═══════════════════════════════════════════════════════

const WEEKDAY_HOURS: (u8, u8) = (7, 22);      // Thứ 2-6: 7:00 - 22:00
const WEEKEND_HOURS: (u8, u8) = (8, 20);       // Thứ 7, CN: 8:00 - 20:00
const LUNCH_BREAK: (u8, u8) = (12, 13);        // Giảm 50% tốc độ giờ trưa

/// Ngoài giờ → queue tin nhắn, gửi sáng hôm sau
/// UI hiện: "Tin nhắn sẽ được gửi lúc 7:00 sáng mai"

// ═══════════════════════════════════════════════════════
// TẦNG 4: CONTENT SAFETY — chặn nội dung nguy hiểm
// ═══════════════════════════════════════════════════════

/// Anti-duplicate: chặn tin giống nhau
const DUPLICATE_WINDOW_SECS: u64 = 300;        // 5 phút
const DUPLICATE_SIMILARITY_THRESHOLD: f64 = 0.85; // Cosine similarity > 85% = duplicate

/// Anti-broadcast: phát hiện gửi hàng loạt
const BROADCAST_DETECTION_WINDOW_MINS: u32 = 30;
const BROADCAST_SAME_CONTENT_LIMIT: u32 = 3;   // Cùng nội dung gửi > 3 người = block
/// UI cảnh báo: "Tin nhắn này giống nhau cho nhiều người. Bạn có chắc?"

/// Blacklist patterns (tự động chặn)
const BLOCKED_PATTERNS: [&str] = [
    "bit.ly/*",          // Link rút gọn (spam signal)
    "click vào link",    // Phishing language
    "chuyển khoản ngay", // Scam language
];

// ═══════════════════════════════════════════════════════
// TẦNG 5: ACCOUNT HEALTH — theo dõi sức khỏe tài khoản
// ═══════════════════════════════════════════════════════

/// Health score: 0-100 (bắt đầu = 100)
/// < 80: cảnh báo vàng, giảm tốc độ 50%
/// < 50: cảnh báo đỏ, chỉ cho gửi thủ công
/// < 20: dừng hoàn toàn, yêu cầu user kiểm tra Zalo

/// Các signal giảm health score:
/// - Gửi tin thất bại liên tiếp (-10/lần)
/// - Tin nhắn bị Zalo ẩn/xóa (-20)
/// - Không nhận được tin nhắn mới > 30 phút (-5) → có thể bị giới hạn
/// - User bị block bởi người nhận (-15)

/// Các signal tăng health score:
/// - Gửi thành công (+1/tin)
/// - Nhận reply từ người nhận (+3) → conversation healthy
/// - Hoạt động bình thường > 24h (+5)
```

---

### 10.4 Rate Limiting Architecture (3 tầng)

```
User approve draft
       │
       ▼
┌─ TẦNG 1: Server Gate ───────────────────────────────┐
│  Redis sliding window:                                │
│    rate:{org}:{conv} → 5/min, 30/hour                │
│    rate:{org}:global → 60/hour, 300/day              │
│  Content check:                                       │
│    Duplicate? Broadcast? Blocked pattern?             │
│  Account health check:                                │
│    health:{account} < 50? → reject                   │
│  > limit? → 429 + UI warning + suggest thủ công      │
└──────┬───────────────────────────────────────────────┘
       │ PASS
       ▼
┌─ TẦNG 2: Agent Safety ──────────────────────────────┐
│  In-memory token bucket (per conversation + global)   │
│  Working hours check (ngoài giờ → queue)              │
│  Human-like delay: random jitter ±30%                 │
│  "Typing simulation": chờ 2-5s trước khi gửi         │
│  Between-conversation delay: 3-8s                     │
└──────┬───────────────────────────────────────────────┘
       │ PASS
       ▼
┌─ TẦNG 3: Execution Safety ──────────────────────────┐
│  Verify Zalo app đang mở + responsive                 │
│  Verify đúng conversation (tên match)                 │
│  Verify tin nhắn đã paste đúng (clipboard check)      │
│  Timeout 10s — nếu không gửi được → fail gracefully  │
│  Screenshot before/after (debug mode)                 │
└──────┬───────────────────────────────────────────────┘
       │ SUCCESS
       ▼
  Log: message_id, sent_at, channel, duration, health_delta
```

---

### 10.5 Account Health Monitoring

```
┌─ Health Dashboard (Web UI) ──────────────────────────┐
│                                                       │
│  Account: sales@company.com                           │
│  ┌──────────────────────────────────────────┐        │
│  │  Health Score: ████████████░░ 85/100     │        │
│  │  Status: 🟢 Healthy                      │        │
│  └──────────────────────────────────────────┘        │
│                                                       │
│  Hôm nay:                                            │
│  ├─ Gửi: 45/300 tin (15%)                            │
│  ├─ Thành công: 44 (97.8%)                            │
│  ├─ Thất bại: 1 (timeout)                             │
│  ├─ Reply nhận: 28 (62% reply rate)                   │
│  └─ Avg response time: 4.2s                           │
│                                                       │
│  Cảnh báo:                                            │
│  ├─ ⚠️ Conversation "Nguyễn Văn A" — 2 tin fail      │
│  └─ ✅ Không có anomaly                               │
│                                                       │
└───────────────────────────────────────────────────────┘
```

#### Health Score Events

| Event | Score Delta | Action |
|---|---|---|
| Gửi thành công | +1 | — |
| Nhận reply | +3 | Conversation marked healthy |
| 24h hoạt động bình thường | +5 | — |
| Gửi thất bại | -10 | Log + retry 1 lần |
| 3 thất bại liên tiếp | -30 | **Dừng gửi** + alert user |
| Tin bị Zalo ẩn/xóa | -20 | Alert + giảm tốc độ 50% |
| Không nhận tin mới > 30 phút | -5 | Check Zalo app status |
| Bị block bởi người nhận | -15 | Đánh dấu contact, không gửi nữa |
| Health < 50 | — | **Chỉ gửi thủ công**, cảnh báo đỏ |
| Health < 20 | — | **Dừng hoàn toàn**, yêu cầu kiểm tra |

---

### 10.6 Anti-Broadcast Protection

Zalo **ban nặng nhất** cho hành vi broadcast (gửi hàng loạt). Haviz chặn triệt để:

```
User approve draft cho Conversation A
       │
       ▼
┌─ Broadcast Detection ──────────────────────────────┐
│                                                     │
│  1. Hash nội dung tin nhắn (SHA-256)                │
│  2. Check Redis: broadcast:{org}:{content_hash}     │
│     Đã gửi cho bao nhiêu conversation trong 30 phút?│
│                                                     │
│  ≤ 3 người: ✅ OK                                   │
│  4-5 người: ⚠️ Warning popup:                       │
│    "Tin nhắn này đã gửi cho 4 người. Tiếp tục?"    │
│  > 5 người: 🚫 BLOCK:                              │
│    "Không thể gửi. Nội dung giống nhau cho quá     │
│     nhiều người. Hãy cá nhân hóa tin nhắn."        │
│                                                     │
│  Similarity check (cosine > 85%):                   │
│    "Chào anh [Tên], em gửi báo giá..."             │
│    ≈ "Chào anh [Tên khác], em gửi báo giá..."      │
│    → Cũng bị detect là broadcast                    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

### 10.7 Fallback Chain (auto-recovery)

```
┌─ Priority 1: Zalo OA (Official API) ─────────────┐
│  Risk: 0% — chính thức                            │
│  Dùng khi: Contact follow OA                      │
│  Fail khi: Contact chưa follow / OA bị giới hạn  │
└──────┬────────────────────────────────────────────┘
       │ fail
       ▼
┌─ Priority 2: Zalo Desktop (AX API + AppleScript) ┐
│  Risk: Rất thấp — OS-level, invisible             │
│  Dùng khi: Zalo Desktop đang mở                   │
│  Fail khi: App không mở / AX API permission       │
└──────┬────────────────────────────────────────────┘
       │ fail
       ▼
┌─ Priority 3: Zalo Web Webview (DOM inject) ───────┐
│  Risk: Thấp — native browser behavior              │
│  Dùng khi: Desktop không available                  │
│  Fail khi: Session hết hạn / DOM thay đổi          │
└──────┬────────────────────────────────────────────┘
       │ fail
       ▼
┌─ Priority 4: Manual Queue ────────────────────────┐
│  Risk: 0% — người gửi thủ công                    │
│  Agent hiện notification:                          │
│    "Không thể tự động gửi. Nhấn để copy tin nhắn  │
│     và gửi thủ công trên Zalo."                   │
│  Tin nhắn status: queued_manual                    │
│  Tự retry lại sau 15 phút                          │
└───────────────────────────────────────────────────┘
```

---

### 10.8 Zalo Update Protection

Khi Zalo cập nhật app/web, UI có thể thay đổi → Agent không đọc/gửi được:

```
Agent detect anomaly:
  - AX API depth thay đổi
  - DOM class names thay đổi
  - Gửi tin thất bại liên tiếp
       │
       ▼
┌─ Self-Healing Protocol ──────────────────────────┐
│                                                   │
│  1. Fallback sang channel khác (10.7)             │
│  2. Agent gửi event: "channel:degraded"           │
│  3. Server log + alert admin                      │
│  4. Queue tất cả tin nhắn (không mất)             │
│  5. Haviz team push Agent update (OTA):           │
│     - Auto-update nếu user cho phép               │
│     - Notification nếu cần manual update           │
│                                                   │
│  Data flow khi Agent lỗi:                         │
│  Agent poll → đọc sai? → KHÔNG sync sai data     │
│  → Confidence check: nếu parsed data bất thường   │
│  → Queue raw data + flag for review                │
│  → Không insert vào DB cho đến khi verified        │
│                                                   │
└───────────────────────────────────────────────────┘

Agent OTA Update:
  - Agent check version mỗi 6 giờ
  - GET https://api.haviz.vn/agent/version
  - Nếu có update: download + verify checksum + restart
  - Rollback nếu update fail
```

---

### 10.9 Database Schema bổ sung (Safety)

```sql
-- Account health tracking
account_health (
  id, user_id, org_id,
  score INT DEFAULT 100,          -- 0-100
  status TEXT DEFAULT 'healthy',  -- healthy | warning | critical | suspended
  messages_sent_today INT,
  messages_failed_today INT,
  last_success_at TIMESTAMP,
  last_failure_at TIMESTAMP,
  last_failure_reason TEXT,
  updated_at TIMESTAMP
)

-- Safety audit log (mọi action đều được log)
safety_audit_log (
  id, org_id, user_id, agent_id,
  event_type TEXT,                -- rate_exceeded | broadcast_blocked | health_warning
                                  -- | duplicate_blocked | outside_hours_queued
                                  -- | fallback_triggered | send_success | send_failed
  details JSONB,                  -- { conversation_id, content_hash, score_delta, ... }
  created_at TIMESTAMP
)

-- Message queue (cho tin nhắn chờ gửi)
message_queue (
  id, message_id, org_id,
  status TEXT DEFAULT 'queued',   -- queued | sending | sent | failed | manual
  channel_attempted TEXT[],       -- ['zalo_oa', 'zalo_desktop'] — đã thử channels nào
  retry_count INT DEFAULT 0,
  max_retries INT DEFAULT 3,
  scheduled_at TIMESTAMP,         -- Gửi lúc nào (nếu ngoài giờ)
  error TEXT,
  created_at TIMESTAMP
)

-- Broadcast detection cache (Redis, backup to PG)
broadcast_log (
  id, org_id,
  content_hash TEXT,              -- SHA-256 of normalized content
  conversation_ids TEXT[],        -- Đã gửi cho ai
  first_sent_at TIMESTAMP,
  window_expires_at TIMESTAMP     -- +30 phút
)
```

---

### 10.10 Session Management

Zalo chỉ cho phép 1 session web + 1 session desktop đồng thời:

```
┌─ Session Monitor (agent/src/session.rs) ─────────────┐
│                                                       │
│  Mỗi 30 giây:                                        │
│  ├─ Zalo Desktop: AX API kiểm tra window tồn tại     │
│  │   → Window không tìm thấy = app đã đóng           │
│  │   → Window tìm thấy nhưng "Đăng nhập" = bị kick   │
│  │                                                    │
│  ├─ Zalo Webview: inject JS kiểm tra                  │
│  │   → document.querySelector('.login-form')          │
│  │     = session hết hạn                              │
│  │   → Kiểm tra chat list có render không              │
│  │     = session còn sống                              │
│  │                                                    │
│  Khi detect session mất:                              │
│  1. Dừng gửi tin ngay lập tức                         │
│  2. Queue tất cả pending messages                     │
│  3. Alert user: "Zalo session hết hạn.                │
│     Vui lòng mở Zalo và đăng nhập lại."              │
│  4. Chuyển sang channel khác (fallback 10.7)          │
│  5. KHÔNG tự re-login (Zalo detect → ban)             │
│                                                       │
│  Khi user login lại:                                  │
│  1. Agent detect session recovered                    │
│  2. Health check: gửi 1 tin test (nếu có)             │
│  3. Resume queue từ từ (rate limited)                 │
│  4. Notification: "Zalo đã kết nối lại."              │
│                                                       │
└───────────────────────────────────────────────────────┘
```

---

### 10.11 Multi-Device Conflict Resolution

User vừa dùng Haviz Agent vừa chat Zalo trên điện thoại:

```
Agent đọc tin nhắn từ Zalo
       │
       ▼
┌─ Sender Detection ──────────────────────────────────┐
│                                                      │
│  So sánh sender name với user's display name:        │
│                                                      │
│  sender == user? → Outbound (user gửi từ Zalo)      │
│    → Mark as: direction=outbound, source=zalo_direct │
│    → KHÔNG tạo AI draft                              │
│    → Sync vào Haviz để giữ lịch sử đầy đủ           │
│                                                      │
│  sender != user? → Inbound (khách gửi đến)           │
│    → Flow bình thường: ingest → AI draft             │
│                                                      │
└──────────────────────────────────────────────────────┘

┌─ Duplicate Send Prevention ─────────────────────────┐
│                                                      │
│  Trước khi Agent gửi tin:                            │
│  1. Đọc tin nhắn mới nhất trong conversation         │
│  2. Nếu tin cuối cùng là outbound + nội dung giống   │
│     + gửi trong 30s → SKIP (user đã gửi thủ công)   │
│  3. Nếu tin cuối cùng là outbound + nội dung khác    │
│     → Gửi bình thường                                │
│                                                      │
│  Edge case: User đang gõ trên điện thoại             │
│  → Agent detect "đang soạn tin" indicator             │
│  → Delay gửi thêm 10s, re-check                     │
│  → Nếu user đã gửi → skip Agent send                │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

### 10.12 Production Checklist

#### Bắt buộc trước khi go-live:
- [ ] `safety.rs` — 5-layer safety engine (rate + human + hours + content + health)
- [ ] `fallback.rs` — 4-priority channel fallback
- [ ] `health.rs` — Account health scoring + auto-throttle
- [ ] `broadcast.rs` — Anti-broadcast detection (hash + similarity)
- [ ] `cache.rs` — Agent local SQLite cache (offline resilience)
- [ ] Server rate limiting (Redis sliding window, 3 tầng)
- [ ] Safety audit log (mọi action đều log)
- [ ] Message queue với retry + scheduled send
- [ ] Data export API (CSV/JSON)
- [ ] Health dashboard UI
- [ ] Broadcast warning UI popup
- [ ] Agent OTA update mechanism
- [ ] Daily encrypted backup (S3/R2)
- [ ] Confidence check cho parsed data (chống sync sai)

#### Monitoring & Alerts:
- [ ] Health score < 50 → Slack/email alert cho admin
- [ ] 3+ send failures liên tiếp → auto-pause + alert
- [ ] Broadcast attempt → log + admin notification
- [ ] Agent disconnect > 5 phút → alert
- [ ] Daily report: messages sent, success rate, health scores

#### Session & Multi-Device:
- [ ] `session.rs` — Session health monitor (30s interval)
- [ ] Session loss detection (AX API + Webview JS check)
- [ ] Auto-queue on session loss + user notification
- [ ] KHÔNG auto re-login (anti-ban)
- [ ] Sender detection (outbound từ user vs inbound từ khách)
- [ ] Duplicate send prevention (30s window)
- [ ] "User đang gõ" detection + delay

---

## 11. Validated (2026-03-18)

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
| Rate limiting ngăn spam | ⬜ Phase 1 |
| Human-like delay giữa các thao tác | ⬜ Phase 1 |
| Account health scoring | ⬜ Phase 1 |
| Anti-broadcast detection | ⬜ Phase 1 |
| Agent local SQLite cache | ⬜ Phase 1 |
| Data export API | ⬜ Phase 1 |
| Daily encrypted backup | ⬜ Phase 1 |
| Agent OTA update | ⬜ Phase 1 |
| Confidence check parsed data | ⬜ Phase 1 |

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

## 12. Execution Roadmap (Thực tế)

> **Nguyên tắc:** Ship nhanh → validate → iterate. Không build thứ chưa cần.
> Mỗi phase kết thúc bằng **validation checkpoint** — nếu fail thì pivot.

### Tổng quan

```
Phase 0: Prototype (4-6 tuần)  → 5-10 beta users, 0đ revenue
Phase 1: MVP (4-6 tuần)       → 50 users, first revenue
Phase 2: Growth (8-12 tuần)   → 200+ users, team features
Phase 3: Scale (ongoing)      → 1000+ users, multi-channel, platform
```

---

### PHASE 0: Working Prototype — "Chạy được trên máy mình"

**Mục tiêu:** Agent đọc Zalo → AI draft → gửi lại. Chạy 100% local, chưa cần cloud.
**Thời gian:** 4-6 tuần
**Team:** 1 dev (bạn)

#### Tuần 1-2: Rust Agent Core

```
□ Khung Agent cơ bản
  ├── Cargo.toml: axum 0.7, tokio, rusqlite, reqwest
  ├── main.rs: tray icon + event loop
  ├── server.rs: Axum HTTP server :9999
  └── tray.rs: system tray (tray-item crate)

□ Zalo Desktop Reader (ĐÃ VALIDATE)
  ├── polling.rs: poll AX API mỗi 3s
  ├── platform/macos/accessibility.rs: traverse hierarchy
  ├── message_parser.rs: depth 18/21/22 → structured message
  └── Detect tin nhắn mới (hash so sánh)

□ Local SQLite
  ├── db.rs: rusqlite connection, WAL mode
  ├── Schema: messages, conversations, contacts (đơn giản)
  └── INSERT message khi detect tin mới

DELIVERABLE: Agent chạy → đọc Zalo → lưu SQLite → log ra terminal
```

#### Tuần 3-4: AI Draft + Send

```
□ AI Draft Engine
  ├── anonymizer.rs: regex xóa tên, SĐT → "Customer"
  ├── ai.rs: call Groq API (reqwest + serde)
  │   System prompt: style cơ bản + org context
  │   Messages: 3 tin gần nhất (anonymized)
  ├── Nhận draft → fill lại tên thật từ SQLite
  └── Lưu draft vào SQLite (status: pending)

□ Template Matching (đơn giản)
  ├── templates table trong SQLite
  ├── Exact + contains match (chưa cần cosine similarity)
  ├── Match → dùng template, skip AI call
  └── 10-20 templates mẫu hard-coded

□ Gửi tin nhắn (ĐÃ VALIDATE)
  ├── channels/zalo_desktop.rs: AppleScript send
  ├── Cmd+F → search tên → paste → Enter
  ├── Random delay 1-3s giữa các bước
  └── Rate limit đơn giản: max 5 tin/phút

DELIVERABLE: Agent đọc tin → AI draft → user approve từ terminal/log → Agent gửi
```

#### Tuần 5-6: Web UI + Polish

```
□ Web UI (served from Agent, localhost:9999)
  ├── Static HTML/JS (hoặc Next.js SSG embed trong Agent)
  ├── Trang Inbox: list conversations, click xem messages
  ├── AI Draft panel: hiện draft, nút Approve / Edit / Reject
  ├── Trang Templates: CRUD templates
  └── Giao diện đơn giản, Tailwind CSS

□ Style Learning (v1 đơn giản)
  ├── Lấy 50 tin outbound từ SQLite
  ├── 1 lần gọi Groq: "Phân tích style viết" → JSON profile
  ├── Cache profile trong SQLite
  └── Inject vào system prompt cho AI draft

□ Polish
  ├── Agent auto-start (LaunchAgent plist)
  ├── Error handling cơ bản (retry, log)
  ├── Setup guide: cách cho Accessibility permission
  └── README cho beta testers

DELIVERABLE: App hoàn chỉnh, chạy local, cho người khác dùng thử được
```

#### Validation Checkpoint Phase 0

```
┌─ PHẢI ĐẠT ĐƯỢC ────────────────────────────────────────────────┐
│                                                                  │
│  □ 5-10 người dùng thử (bạn bè, đồng nghiệp, sales quen biết) │
│  □ Chạy ổn định 1 tuần không crash                              │
│  □ Zalo không ban sau 1 tuần sử dụng bình thường                │
│  □ AI draft quality: > 50% approve without edit                  │
│  □ Feedback: ít nhất 3/5 người nói "có ích, muốn dùng tiếp"    │
│                                                                  │
│  NẾU FAIL:                                                       │
│  - AI quality thấp → cải thiện prompt, thêm templates            │
│  - Zalo ban → pivot sang chỉ Zalo OA (official API)             │
│  - Không ai muốn dùng → pivot features hoặc target audience     │
│  - Crash nhiều → fix bugs trước khi tiếp Phase 1                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

### PHASE 1: MVP — "Người lạ trả tiền"

**Mục tiêu:** Cloud + team features + payment. First paying customers.
**Thời gian:** 4-6 tuần (sau Phase 0 validate OK)
**Điều kiện bắt đầu:** Phase 0 validation pass

#### Tuần 7-8: Cloud Backend

```
□ Haviz Cloud API (Hono + Node.js)
  ├── Auth: Supabase Auth (email + password)
  ├── REST: /auth, /templates, /agents, /analytics
  ├── WebSocket: Agent ↔ Server connection
  └── Deploy: VPS tại VN (1 server đủ)

□ Cloud Database (PostgreSQL)
  ├── Tầng 1 tables: organizations, users, channels, agents
  ├── Templates (shared, team level)
  ├── Daily metrics (nhận từ Agent)
  └── Drizzle ORM + migrations

□ Agent ↔ Cloud sync
  ├── ws_client.rs: WebSocket connect → Cloud
  ├── sync.rs: gửi metrics (chỉ số, không tin nhắn)
  ├── Download templates mới từ Cloud
  └── Agent auth: token từ Cloud, lưu Keychain
```

#### Tuần 9-10: Zalo OA + Mobile

```
□ Zalo OA Integration (Cloud channel, 0% risk)
  ├── POST /webhooks/zalo-oa: nhận tin nhắn
  ├── Zalo OA API: gửi tin nhắn (official)
  ├── AI draft cho OA messages (cloud-side)
  └── Inbox hiển thị cả OA + Desktop

□ Mobile App (React Native / Expo, đơn giản)
  ├── Login → connect Cloud API
  ├── Inbox: list conversations (Tầng 1+2 từ Cloud)
  ├── AI Draft: xem + Approve / Reject
  ├── Push notification: tin nhắn mới
  └── KHÔNG cần đọc full messages (chỉ preview + approve)
```

#### Tuần 11-12: Payment + Launch

```
□ Payment
  ├── Pricing page trên web
  ├── Payment gateway: VNPay / MoMo / Stripe
  ├── Free tier: 50 AI drafts/tháng
  ├── Pro: 199k/tháng (unlimited)
  └── Trial: 14 ngày Pro free

□ Landing Page + Onboarding
  ├── haviz.vn: landing page (benefits, pricing, download)
  ├── Onboarding flow: signup → download Agent → setup
  ├── Video hướng dẫn: 3 phút setup
  └── Support: Zalo group hoặc Telegram

□ Launch
  ├── Post lên các group sales/marketing VN
  ├── Facebook groups cho salesperson
  ├── Tặng 3 tháng Pro cho 20 early adopters
  └── Collect feedback liên tục
```

#### Validation Checkpoint Phase 1

```
┌─ PHẢI ĐẠT ĐƯỢC ────────────────────────────────────────────────┐
│                                                                  │
│  □ 50+ registered users                                          │
│  □ 10+ daily active users                                        │
│  □ 3-5 paying users (chứng minh willingness to pay)              │
│  □ Churn: < 30% sau tháng đầu                                   │
│  □ NPS score: > 30 (hỏi "bạn có giới thiệu cho người khác?")   │
│  □ Zalo vẫn không ban sau 1 tháng multi-user                    │
│                                                                  │
│  NẾU FAIL:                                                       │
│  - 0 paying users → pricing sai, hoặc value chưa đủ             │
│    → Thử: giảm giá, thêm features, đổi target audience          │
│  - Churn cao → product chưa sticky                               │
│    → Thử: cải thiện AI quality, thêm templates, UX tốt hơn      │
│  - Zalo ban → pivot 100% sang Zalo OA only                       │
│                                                                  │
│  NẾU PASS → có REVENUE → Phase 2                                │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

### PHASE 2: Growth — "Team trả tiền"

**Mục tiêu:** Team features, 200+ users, sustainable revenue.
**Thời gian:** 8-12 tuần
**Điều kiện bắt đầu:** Phase 1 có ít nhất 5 paying users

#### Tuần 13-16: Team Features

```
□ Team Dashboard (Cloud)
  ├── Manager view: team performance (metrics từ Agents)
  ├── Conversations list (across team members)
  ├── Response time ranking, AI adoption rate
  └── Basic analytics charts

□ Shared Templates
  ├── Org-level templates (manager tạo, team dùng)
  ├── Template categories + search
  ├── Usage tracking: template nào hiệu quả nhất
  └── Import/export templates

□ Team Pricing
  ├── Team plan: 399k/user
  ├── Billing: per-org, manager trả cho cả team
  ├── Roles: owner, admin, member
  └── Invite flow: email invite → join org
```

#### Tuần 17-20: Multi-channel + Analytics

```
□ Facebook Messenger (Cloud channel)
  ├── FB Graph API integration
  ├── Webhook: nhận tin nhắn từ FB Page
  ├── Gửi reply qua API
  └── Unified inbox: Zalo + OA + Messenger

□ Enhanced Analytics
  ├── AI insights (Agent anonymize → Groq → dashboard)
  ├── Top topics, sentiment trends
  ├── Template effectiveness reports
  └── Weekly email report cho manager

□ Improvements from feedback
  ├── UX improvements (từ user feedback)
  ├── AI draft quality improvements
  ├── Bug fixes, stability
  └── Performance optimization
```

#### Tuần 21-24: Scale preparation

```
□ E2E Encryption (Tầng 2)
  ├── crypto.rs: AES-256-GCM
  ├── Key trong OS Keychain
  ├── Encrypt metadata trước khi sync Cloud
  └── Web UI decrypt in-browser

□ Offline sync
  ├── sync_queue table
  ├── Resume sync khi online
  └── Last-write-wins cho conflicts

□ Agent stability
  ├── Auto-update OTA
  ├── Crash recovery
  ├── SQLite backup daily
  └── Session management (detect Zalo logout)
```

#### Validation Checkpoint Phase 2

```
┌─ PHẢI ĐẠT ĐƯỢC ────────────────────────────────────────────────┐
│                                                                  │
│  □ 200+ total users, 50+ paying                                 │
│  □ 5+ team accounts (mỗi team 5-10 users)                       │
│  □ MRR (Monthly Recurring Revenue): > 10 triệu/tháng            │
│  □ Churn < 10% monthly                                           │
│  □ At least 2 channels hoạt động (Zalo + OA hoặc Messenger)     │
│  □ Positive unit economics (revenue > server cost)               │
│                                                                  │
│  NẾU PASS → Phase 3                                             │
│  NẾU FAIL → iterate Phase 2, đừng thêm features mới             │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

### PHASE 3: Scale — "Platform"

**Mục tiêu:** 1000+ users, multi-channel, revenue > 100 triệu/tháng.
**Thời gian:** Ongoing
**Điều kiện bắt đầu:** Phase 2 MRR > 10 triệu

```
Chỉ build khi có revenue ổn định:

□ Windows Agent
  ├── Win32 UI Automation thay AX API
  ├── SendInput thay AppleScript
  └── MSI installer + Authenticode signing

□ Telegram + Phone/SMS integration

□ REST API + SDK
  ├── Public API cho developers
  ├── npm/pip SDK
  └── API pricing tier

□ Template Marketplace
  ├── Creator upload template packs
  ├── Revenue share 70/30
  └── Review + approval flow

□ Voice Reports (Phase 2 Intelligence)
  ├── Record → Whisper → transcript
  ├── AI extract action items
  └── Auto-create follow-ups

□ Advanced features
  ├── Chatbot semi-auto mode
  ├── MCP server cho AI agents
  ├── White-label
  ├── Enterprise SSO
  └── On-premise option

□ Thuê thêm dev
  ├── 1 Rust dev (Agent)
  ├── 1 Fullstack (Web + API)
  ├── 1 Mobile dev
  └── Khi MRR > 50 triệu
```

---

### Timeline tổng thể (thực tế)

```
2026
Mar  Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec
 │    │    │    │    │    │    │    │    │    │
 ├────┤    │    │    │    │    │    │    │    │
 Phase 0   │    │    │    │    │    │    │    │
 Prototype │    │    │    │    │    │    │    │
 5-10 users│    │    │    │    │    │    │    │
           │    │    │    │    │    │    │    │
           ├────┤    │    │    │    │    │    │
           Phase 1   │    │    │    │    │    │
           MVP       │    │    │    │    │    │
           50 users  │    │    │    │    │    │
           1st rev   │    │    │    │    │    │
                     │    │    │    │    │    │
                     ├─────────┤    │    │    │
                     Phase 2        │    │    │
                     Growth         │    │    │
                     200+ users     │    │    │
                     Team features  │    │    │
                                    │    │    │
                                    ├─────────────
                                    Phase 3
                                    Scale
                                    1000+ users
                                    Platform

Revenue:
  Mar-Apr: 0đ
  May-Jun: 2-5 triệu/tháng (first paying users)
  Jul-Sep: 10-20 triệu/tháng (teams onboard)
  Oct-Dec: 30-50 triệu/tháng (growth)
```

### Tóm tắt: Mỗi Phase build gì

| | Phase 0 | Phase 1 | Phase 2 | Phase 3 |
|---|---|---|---|---|
| **Agent** | AX API + SQLite + AI + Send | + Cloud sync | + E2E + OTA update | + Windows |
| **AI** | Groq draft + templates | + Style learning | + Analytics insights | + Chatbot |
| **Web** | Localhost UI đơn giản | + Cloud web app | + Team dashboard | + API + Marketplace |
| **Mobile** | Không | Basic (approve drafts) | + Full inbox | + Push + Voice |
| **Channels** | Zalo Desktop only | + Zalo OA | + Messenger | + Telegram + Phone |
| **Cloud** | Không | Auth + Templates + Metrics | + E2E + Analytics | + Full platform |
| **Revenue** | 0đ | 2-5 triệu | 10-50 triệu | 100+ triệu |

---

## 13. Legal & Compliance (Pháp lý)

> **CẢNH BÁO:** Haviz hoạt động trong **vùng xám pháp lý**. Phần này phân tích rủi ro thật
> và biện pháp giảm thiểu bắt buộc trước khi go production.

### 13.1 Luật áp dụng

| Luật | Hiệu lực | Liên quan đến Haviz |
|---|---|---|
| **Luật 91/2025/QH15** — Bảo vệ dữ liệu cá nhân (PDPL 2025) | 01/01/2026 | Thu thập, lưu trữ, xử lý tin nhắn |
| **Bộ luật Hình sự — Điều 159** | Hiện hành | Xâm phạm bí mật thư tín, điện tử |
| **Luật An ninh mạng 2018** + Nghị định 53/2022 | Hiện hành | Lưu trữ data tại Việt Nam |
| **Luật Giao dịch điện tử 2023** | 01/07/2024 | Tính xác thực tin nhắn tự động |
| **Điều khoản sử dụng Zalo (ToS v2)** | 2025 | Automation, scraping |

> Lưu ý: Nghị định 13/2023/NĐ-CP đã **hết hiệu lực từ 01/01/2026**, thay thế bởi Luật 91/2025.

---

### 13.2 Phân tích rủi ro pháp lý

#### RỦI RO 1: Thu thập tin nhắn không có sự đồng ý (PDPL 2025) — **CAO**

```
Vấn đề:
  Khách hàng gửi tin nhắn cho salesperson trên Zalo
  → Haviz đọc + lưu tin nhắn đó vào cloud database
  → Khách hàng KHÔNG biết, KHÔNG đồng ý

Luật 91/2025, Điều 18:
  Xử lý dữ liệu cá nhân phải có SỰ ĐỒNG Ý của chủ thể dữ liệu
  (chủ thể = người gửi tin nhắn = khách hàng)

Ngoại lệ (Điều 19) — KHÔNG áp dụng cho Haviz:
  ✗ "Lợi ích chính đáng" — hẹp hơn GDPR, yêu cầu phòng vệ
  ✗ "Thực hiện hợp đồng" — yếu cho liên hệ ban đầu
  ✗ "Bảo vệ tính mạng" — không liên quan
```

#### RỦI RO 2: Bộ luật Hình sự, Điều 159 — **CAO (nghiêm trọng nhất)**

```
Điều 159: Xâm phạm bí mật hoặc an toàn thư tín, điện thoại,
          điện tín hoặc hình thức trao đổi thông tin riêng tư

Tin nhắn Zalo = "trao đổi thông tin riêng tư qua điện tử"

Hành vi: Đọc, lưu trữ, truyền nội dung tin nhắn của người khác
         đến database bên thứ ba (Haviz cloud)

Hình phạt:
  - Phạt tù 1-3 năm
  - Phạt tiền 5-20 triệu VND
  - Tình tiết tăng nặng: tù đến 7 năm

Lưu ý: Khả năng bị truy tố chủ động THẤP,
        nhưng khiếu nại từ khách hàng/đối thủ có thể kích hoạt
```

#### RỦI RO 3: Vi phạm Điều khoản Zalo (ToS) — **CAO**

```
Zalo đã mã hóa AES cho Web API → cấm bên thứ ba truy cập
Haviz dùng AX API đọc UI → bypass kiểm soát truy cập của Zalo
→ Vi phạm ToS về truy cập tự động không được phép

Hậu quả:
  - Khóa tài khoản (gần như chắc chắn nếu bị phát hiện)
  - VNG (công ty mẹ Zalo) có thể kiện dân sự
  - VNG chiếm 85% thị phần, đang bị giám sát cạnh tranh
```

#### RỦI RO 4: Lưu trữ data xuyên biên giới — **TRUNG BÌNH**

```
Nếu dùng cloud ngoài Việt Nam (AWS US, Google Cloud Singapore):
  → Phải lập Đánh giá tác động chuyển dữ liệu (CTIA)
  → Nộp trong vòng 60 ngày kể từ khi bắt đầu chuyển
  → Luật An ninh mạng: lưu trữ data tại VN, tối thiểu 24 tháng
```

#### RỦI RO 5: Tính xác thực tin nhắn — **THẤP-TRUNG BÌNH**

```
Tin nhắn do AI soạn, gửi qua AppleScript → người nhận tưởng
salesperson gõ → vấn đề minh bạch theo Luật GDĐT 2023
```

---

### 13.3 Bảng tổng hợp rủi ro (cập nhật cho Local-first)

| Rủi ro | Cloud cũ | **Local-first** | Lý do giảm |
|---|---|---|---|
| PDPL 2025 — thu thập không consent | **CAO** | **RẤT THẤP** | Haviz không "thu thập" — data ở trên máy user |
| Điều 159 BLHS — bí mật thư tín | **CAO** | **RẤT THẤP** | Không truyền tin nhắn đến bên thứ ba |
| Zalo ToS — automation | **CAO** | **THẤP** | Giống user tự đọc + ghi chú, data không rời device |
| Data xuyên biên giới | TRUNG BÌNH | **KHÔNG ÁP DỤNG** | Data ở local, Cloud chỉ có metadata E2E encrypted |
| Luật An ninh mạng | TRUNG BÌNH | **THẤP** | Không có raw data trên server |
| AI data leak (Groq) | TRUNG BÌNH | **THẤP** | Anonymized trước khi gửi |

---

### 13.4 So sánh với CRM quốc tế (HubSpot, Salesforce)

| Yếu tố | HubSpot / Salesforce | Haviz Cloud (cũ) | **Haviz Local-first (mới)** |
|---|---|---|---|
| Nguồn data | Email gửi ĐẾN business | Zalo scrape → cloud | Zalo scrape → **local SQLite** |
| Data lưu ở đâu | Cloud CRM | Cloud server | **Máy user** |
| Platform hợp tác | API chính thức | Bypass Zalo | **Bypass nhưng data ở local** |
| Consent | Privacy policy, checkbox | Chưa có | Consent mechanism built-in |
| Bên thứ ba nhận data | CRM cloud | Haviz cloud | **Không ai** (chỉ user) |
| AI access | Cloud AI đọc raw | Cloud AI đọc raw | **Anonymized** trước khi gửi AI |

**Với Local-first:** Haviz giống một **app ghi chú thông minh** trên máy user hơn là CRM cloud. User tự đọc tin nhắn Zalo → tự lưu vào SQLite trên máy mình → tự dùng AI hỗ trợ soạn reply. Haviz chỉ cung cấp công cụ.

**Zalo OA** vẫn là kênh an toàn nhất — official API, 0% rủi ro.

---

### 13.5 Vai trò pháp lý (cập nhật cho Local-first)

```
Với LOCAL-FIRST, vai trò thay đổi đáng kể:

Công ty/Salesperson sử dụng Haviz = CHỦ THỂ TỰ XỬ LÝ
  → Tự đọc tin nhắn Zalo trên máy mình
  → Tự lưu vào SQLite trên máy mình
  → Giống như tự ghi chú, screenshot, sao lưu tin nhắn
  → Haviz chỉ là CÔNG CỤ, không "thu thập" data

Haviz = NHÀ CUNG CẤP CÔNG CỤ (Tool Provider)
  → Cung cấp phần mềm Agent + Cloud services
  → KHÔNG truy cập được tin nhắn gốc (Tầng 3 ở local)
  → Chỉ lưu metadata encrypted (Tầng 2, không đọc được)
  → Vai trò tương tự: Microsoft Word, Notion, Obsidian
  → Vẫn nên có DPA cho Tầng 1+2 (templates, metadata)
```

---

### 13.6 Biện pháp bắt buộc trước production

#### A. Pháp lý (Legal)

```
1. DATA PROCESSING AGREEMENT (DPA)
   Hợp đồng giữa Haviz (Processor) và từng công ty khách hàng (Controller)
   Nội dung: phạm vi xử lý, mục đích, bảo mật, thông báo vi phạm
   → Chuyển trách nhiệm consent về phía Controller

2. DATA PROTECTION OFFICER (DPO)
   Bắt buộc theo PDPL 2025
   Bổ nhiệm người có chuyên môn về bảo vệ dữ liệu

3. TERMS OF SERVICE + PRIVACY POLICY
   Rõ ràng: data nào thu thập, xử lý thế nào, lưu ở đâu, bao lâu

4. TƯ VẤN PHÁP LÝ
   Thuê luật sư chuyên về data protection tại VN
   Gợi ý: Tilleke & Gibbins, Baker McKenzie VN, KPMG Legal VN

5. ĐĂNG KÝ VỚI CƠ QUAN CHỨC NĂNG
   PDPL 2025 có thể yêu cầu đăng ký/thông báo với Bộ Công an
```

#### B. Kỹ thuật — Consent Mechanism

```
TIN NHẮN ĐẦU TIÊN cho mỗi contact mới (bắt buộc):

┌──────────────────────────────────────────────────────┐
│  [Auto-message khi contact mới nhắn tin lần đầu]     │
│                                                       │
│  "Cảm ơn bạn đã liên hệ [Tên Công Ty].              │
│   Tin nhắn của bạn có thể được lưu trữ và xử lý     │
│   bởi hệ thống quản lý khách hàng của chúng tôi     │
│   để phục vụ bạn tốt hơn.                            │
│   Xem chính sách bảo mật: [link]                     │
│   Nếu không đồng ý, vui lòng phản hồi 'STOP'."      │
│                                                       │
│  Contact reply "STOP" → opt-out:                      │
│    - Không lưu tin nhắn nữa                           │
│    - Xóa data đã lưu trong 30 ngày                   │
│    - Đánh dấu contact: consent=false                  │
│                                                       │
│  Contact tiếp tục chat → implied consent              │
└──────────────────────────────────────────────────────┘
```

#### C. Kỹ thuật — Data Subject Rights (PDPL 2025)

```
API endpoints cho quyền của chủ thể dữ liệu:

GET  /api/v1/data-subject/lookup?phone=xxx
  → Tìm data liên quan đến số điện thoại/tên

GET  /api/v1/data-subject/:id/export
  → Xuất toàn bộ data của 1 chủ thể (portability)

POST /api/v1/data-subject/:id/correct
  → Chỉnh sửa thông tin sai

DELETE /api/v1/data-subject/:id
  → Xóa toàn bộ data (right to be forgotten)

POST /api/v1/data-subject/:id/restrict
  → Dừng xử lý data nhưng không xóa

POST /api/v1/data-subject/:id/withdraw-consent
  → Rút consent, dừng mọi xử lý
```

#### D. Kỹ thuật — AI Transparency

```
Mỗi tin nhắn AI soạn, trước khi gửi:
  Option 1: Thêm disclaimer nhỏ (configurable)
    "[Tin nhắn này được hỗ trợ bởi AI]"

  Option 2: Không thêm disclaimer (user chọn)
    → User chịu trách nhiệm về nội dung

  Lưu trong DB: message.ai_generated = true
  → Có bằng chứng nếu bị audit
```

#### E. Data Localization

```
BẮT BUỘC: Lưu trữ data tại Việt Nam
  → Supabase region: Singapore → KHÔNG ĐỦ
  → Cần: Supabase self-hosted tại VN hoặc VPS tại VN
  → Hoặc: Cloud VN (VNG Cloud, FPT Cloud, VNPT Cloud)
  → Backup: cũng phải ở VN

Nếu dùng cloud ngoài VN:
  → Lập CTIA (Cross-border Transfer Impact Assessment)
  → Nộp trong 60 ngày
  → Lưu data tối thiểu 24 tháng
```

---

### 13.7 Chiến lược giảm rủi ro tối ưu

```
CHIẾN LƯỢC: Ưu tiên Zalo OA, hạn chế Zalo cá nhân

Mức 1: Zalo OA (Official Account) — AN TOÀN NHẤT
  ✅ API chính thức, được Zalo cho phép
  ✅ Tin nhắn đến OA = business context → consent hợp lý
  ✅ Giống email business → CRM hợp pháp
  → ƯU TIÊN CAO NHẤT cho production

Mức 2: Zalo cá nhân có consent — CHẤP NHẬN ĐƯỢC
  ⚠️ Chỉ dùng cho conversation ĐÃ có consent message
  ⚠️ Contact reply "STOP" → dừng ngay
  ⚠️ Chỉ cho chat 1-1 đã có quan hệ khách hàng
  → Dùng khi OA không áp dụng được

Mức 3: Zalo cá nhân không consent — KHÔNG KHUYẾN KHÍCH
  ❌ Rủi ro pháp lý cao nhất
  ❌ Chỉ nên dùng cho internal team chat
  → Cân nhắc bỏ hoặc chỉ demo
```

---

### 13.8 PII Protection (Bảo vệ thông tin cá nhân)

```
Tự động detect + mask PII trước khi lưu DB:

Loại PII          Ví dụ              Masked
─────────────────  ──────────────────  ──────────────────
Số điện thoại      0912345678         0912***678
CCCD/CMND          012345678901       0123****8901
Email              abc@gmail.com      a***@gmail.com
Số tài khoản       9704xxxxxxxx1234   9704****1234
Địa chỉ            123 Nguyễn Huệ    [ĐỊA CHỈ ĐÃ ẨN]

Lưu 2 bản:
  messages.content          → masked (mặc định)
  messages.content_raw      → encrypted AES-256 (chỉ DPO + admin access)

Tự động xóa content_raw sau X ngày (configurable, mặc định 90 ngày)
```

---

### 13.9 Database Schema bổ sung (Legal)

```sql
-- Consent tracking
contact_consents (
  id, contact_id, org_id,
  consent_type TEXT,          -- 'message_storage' | 'ai_processing' | 'marketing'
  status TEXT,                -- 'granted' | 'withdrawn' | 'pending'
  granted_at TIMESTAMP,
  withdrawn_at TIMESTAMP,
  consent_message_id TEXT,    -- Tin nhắn chứa disclosure
  method TEXT,                -- 'first_message_disclosure' | 'explicit_opt_in' | 'oa_follow'
  evidence_json JSONB         -- Screenshot/log chứng minh consent
)

-- Data subject requests
data_subject_requests (
  id, org_id,
  subject_identifier TEXT,    -- phone, email, hoặc name
  request_type TEXT,          -- 'access' | 'export' | 'delete' | 'correct' | 'restrict'
  status TEXT,                -- 'pending' | 'processing' | 'completed' | 'rejected'
  requested_at TIMESTAMP,
  completed_at TIMESTAMP,
  completed_by TEXT,
  notes TEXT
)

-- PII audit log
pii_access_log (
  id, org_id, user_id,
  contact_id TEXT,
  data_accessed TEXT,         -- 'content_raw' | 'phone' | 'cccd'
  reason TEXT,
  accessed_at TIMESTAMP
)
```

---

### 13.10 Legal Compliance Checklist

#### Bắt buộc trước go-live:
- [ ] Thuê luật sư VN chuyên data protection → legal opinion
- [ ] Soạn DPA (Data Processing Agreement) template
- [ ] Soạn Terms of Service + Privacy Policy
- [ ] Bổ nhiệm DPO (Data Protection Officer)
- [ ] Consent mechanism (first-message disclosure)
- [ ] Data Subject Rights API (access, delete, export, correct)
- [ ] PII detection + masking (phone, CCCD, email, address)
- [ ] Data localization: server tại VN hoặc CTIA
- [ ] Contact consent tracking (DB + audit log)
- [ ] AI transparency flag trên mỗi tin nhắn AI soạn
- [ ] Opt-out mechanism ("STOP" → dừng lưu trữ)
- [ ] Data retention policy (auto-delete sau X ngày)
- [ ] Đăng ký với Bộ Công an (nếu PDPL 2025 yêu cầu)

#### Chiến lược:
- [ ] Ưu tiên Zalo OA (Official API) — 0% rủi ro pháp lý
- [ ] Zalo cá nhân chỉ dùng với consent
- [ ] Tìm hiểu Zalo Partner Program / ZNS
- [ ] Chuẩn bị response plan nếu bị Zalo/VNG liên hệ

### 13.11 Nguồn tham khảo

- [Luật 91/2025/QH15 — Bảo vệ dữ liệu cá nhân](https://thuvienphapluat.vn/van-ban/Bo-may-hanh-chinh/Luat-Bao-ve-du-lieu-ca-nhan-2025-so-91-2025-QH15-625628.aspx)
- [Luật 91/2025 — Bản tiếng Anh](https://thuvienphapluat.vn/van-ban/EN/Bo-may-hanh-chinh/Law-91-2025-QH15-Personal-Data-Protection/665440/tieng-anh.aspx)
- [Tilleke & Gibbins — Vietnam's New PDPL](https://www.tilleke.com/insights/vietnams-new-personal-data-protection-law-a-closer-look/)
- [Baker McKenzie — Decoding Vietnam's PDP Law](https://connectontech.bakermckenzie.com/vietnam-decoding-vietnams-pdp-law-gdpr-inspired-rules-with-local-twists/)
- [KPMG — Exploring Vietnam's PDPL](https://kpmg.com/vn/en/home/insights/2025/06/vietnam-new-personal-data-protection-law.html)
- [IAPP — Vietnam's PDPL in Focus](https://iapp.org/news/a/vietnams-pdpl-in-focus-what-to-know-and-watch-for)
- [Nghị định 53/2022 — Luật An ninh mạng (PwC)](https://www.pwc.com/vn/en/publications/2022/220908-pwc-vietnam-legal-newsbrief-decree-53.pdf)
- [DLA Piper — Data protection laws Vietnam](https://www.dlapiperdataprotection.com/?t=law&c=VN)
- [Zalo ToS Update — DPS Media](https://dps.media/en/update-zalo-terms-of-use-important-changes/)
- [Zalo Compliance — Infobip](https://www.infobip.com/docs/zalo/compliance-guidelines)
- [Điều 159 BLHS — Xâm phạm bí mật thư tín](https://lsvn.vn/mot-so-van-de-ve-toi-xam-pham-bi-mat-hoac-an-toan-thu-tin-dien-thoai-dien-tin-hoac-hinh-thuc-trao-doi-thong-tin-rieng-tu-khac-cua-nguoi-khac-theo-quy-dinh-cua-bo-luat-hinh-su-1677594791-a128129.html)

---

## 14. Business Model

> **Nguyên tắc:** Haviz bán CÔNG CỤ, không bán DATA.
> Giống Notion, Figma, Obsidian — bán phần mềm, user sở hữu data.

### 14.1 Nguồn doanh thu

```
┌─────────────────────────────────────────────────┐
│              NGUỒN DOANH THU                     │
├──────────────┬─────────────────┬────────────────┤
│ 1. SaaS Sub  │ 2. AI Credits   │ 3. Marketplace │
│   (80%)      │   (15%)         │   (5%)         │
├──────────────┼─────────────────┼────────────────┤
│ Pro/Team/    │ Gói Free: 50    │ Template store │
│ Enterprise   │ drafts/tháng    │ Plugin/Ext     │
│ monthly      │ Mua thêm:       │ Integration    │
│              │ 10k/100 drafts  │ API            │
└──────────────┴─────────────────┴────────────────┘
```

### 14.2 Pricing

| Gói | Giá/tháng | Tính năng | Target |
|---|---|---|---|
| **Free** | 0đ | 1 user, 1 Zalo account, 50 AI drafts/tháng, 10 templates | Freelancer |
| **Pro** | 199k/user | Unlimited AI drafts, unlimited templates, mobile, style learning | Cá nhân sales |
| **Team** | 399k/user | Pro + team dashboard, manager analytics, shared templates | Team 5-20 |
| **Enterprise** | Liên hệ | Team + SSO, API, custom AI model, on-premise, SLA | Công ty 50+ |

### 14.3 Tại sao user trả tiền

| Tính năng | Giá trị | User tiết kiệm |
|---|---|---|
| AI Draft unlimited | Không cần gõ từng reply | 2-3 giờ/ngày |
| Style Learning | AI viết giống chính mình | Khách không biết là AI |
| Team Dashboard | Manager biết ai reply nhanh/chậm | Quản lý hiệu quả hơn |
| Shared Templates | Team mới vào dùng ngay | Không cần training |
| Multi-channel | Zalo OA + Desktop + Web trong 1 chỗ | Không tab-switch |
| Mobile access | Approve draft trên điện thoại | Reply khi ra ngoài |

### 14.4 Ước tính doanh thu

```
Năm 1 — MVP:
  100 users Pro × 199k   =  19.9 triệu/tháng
   20 users Team × 399k  =   7.98 triệu/tháng
  ────────────────────────────────────────────
  Tổng:                     ~28 triệu/tháng (~336 triệu/năm)

Năm 2 — Growth:
  500 users Pro × 199k   =  99.5 triệu/tháng
  200 users Team × 399k  =  79.8 triệu/tháng
    5 Enterprise × 5tr   =  25.0 triệu/tháng
  ────────────────────────────────────────────
  Tổng:                    ~204 triệu/tháng (~2.45 tỷ/năm)

Năm 3 — Scale:
  2000 Pro + 1000 Team + 20 Enterprise
  ────────────────────────────────────────────
  Tổng:                    ~800 triệu/tháng (~9.6 tỷ/năm)
```

### 14.5 Chi phí vận hành

```
Chi phí/tháng (100 users):
  AI (Groq):          $5  (~125k VND)  — đã tối ưu 90%
  Cloud VN (VPS):     2 triệu          — Tầng 1+2 (nhẹ, không có messages)
  Supabase Auth:      Free tier
  Domain + SSL:       200k
  ──────────────────────────────
  Tổng:               ~2.5 triệu/tháng

  Doanh thu:          28 triệu
  Chi phí:            2.5 triệu
  ──────────────────────────────
  Gross Margin:       ~91%
```

**Tại sao margin cao:** Local-first = user tự chịu storage (SQLite trên máy). Haviz Cloud chỉ lưu metadata nhẹ → chi phí server rất thấp.

### 14.6 Moat (Lợi thế cạnh tranh)

```
1. LOCK-IN TỰ NHIÊN
   Dùng càng lâu → style profile càng chính xác
   → AI viết càng giống → càng khó bỏ Haviz
   → Churn rate thấp

2. TECHNICAL MOAT
   Rust Agent + AX API = ít developer VN có thể clone
   Local-first + E2E encryption = phức tạp để replicate
   Anonymizer + Safety Engine = domain expertise

3. NETWORK EFFECT (Team)
   1 người dùng → giới thiệu cả team
   → Shared templates → manager dashboard
   → Team gói thay vì Pro

4. MARKETPLACE EFFECT (Phase 3)
   Template packs theo ngành (BĐS, phòng khám, e-commerce)
   Haviz lấy 30% commission
   Nhiều templates → nhiều users → nhiều creators → flywheel

5. DATA ADVANTAGE (ethical)
   Style profiles improve over time (local, user-owned)
   Template usage analytics → better recommendations
   Không bán data → trust = retention
```

### 14.7 Template Marketplace (Phase 3)

```
Các ngành tạo template pack:
  🏠 Bất động sản  — 50 templates  — 299k
  🏥 Phòng khám    — 30 templates  — 199k
  🎓 Giáo dục      — 40 templates  — 249k
  🛒 E-commerce    — 60 templates  — 399k
  💼 B2B Sales     — 45 templates  — 349k

Revenue split: Creator 70% | Haviz 30%
```

---

## 15. Security Architecture

### 15.1 Authentication Flow

```
┌─ User Auth (Web/Mobile) ────────────────────────────────────────┐
│                                                                  │
│  1. User login: email + password → Supabase Auth                 │
│  2. Supabase trả về: access_token (JWT, 1h) + refresh_token     │
│  3. Web/Mobile gửi: Authorization: Bearer <access_token>        │
│  4. Haviz API verify JWT → extract user_id, org_id              │
│  5. Token hết hạn → refresh tự động (refresh_token, 30 ngày)    │
│                                                                  │
│  JWT payload:                                                    │
│  {                                                               │
│    sub: "user_abc",                                              │
│    org_id: "org_123",                                            │
│    role: "member",        // owner | admin | member              │
│    iat: 1711000000,                                              │
│    exp: 1711003600        // +1h                                 │
│  }                                                               │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

┌─ Agent Auth (Machine-to-Server) ────────────────────────────────┐
│                                                                  │
│  1. User cài Agent → login 1 lần qua browser OAuth flow         │
│  2. Server cấp agent_token (random 256-bit, hashed in DB)       │
│  3. Agent lưu token encrypted trong OS keychain:                 │
│     macOS: Keychain Access                                       │
│     Windows: Windows Credential Manager                          │
│  4. Mỗi request: X-Agent-Token: <agent_token>                   │
│  5. Server verify: hash(token) == agents.auth_token_hash         │
│                                                                  │
│  Token rotation:                                                 │
│  - Tự động rotate mỗi 30 ngày                                   │
│  - Agent request new token → server issue + revoke old           │
│  - Nếu token bị compromise: user revoke từ Web UI               │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

┌─ WebSocket Auth ────────────────────────────────────────────────┐
│                                                                  │
│  Agent → Server:                                                 │
│  1. WSS connect: wss://api.haviz.vn/ws?token=<agent_token>      │
│  2. Server verify token → accept/reject                          │
│  3. Heartbeat mỗi 30s để giữ connection                         │
│  4. Token expire → server gửi "auth:expired" → Agent re-auth    │
│                                                                  │
│  Web/Mobile → Server:                                            │
│  1. WSS connect: wss://api.haviz.vn/ws?jwt=<access_token>       │
│  2. JWT expire → client gửi "auth:refresh" với refresh_token    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 15.2 API Security

```
Mọi API endpoint:
  ├── Rate limiting:  100 req/min per user, 1000 req/min per org
  ├── CORS:           Chỉ allow haviz.vn, localhost:9999
  ├── HTTPS only:     HTTP → redirect 301
  ├── Input validation: Zod schema cho mọi request body
  ├── SQL injection:   Drizzle ORM (parameterized queries)
  ├── XSS:            Content-Security-Policy headers
  └── CSRF:           SameSite cookies + CSRF token

Agent local API (localhost:9999):
  ├── Chỉ accept từ: 127.0.0.1, LAN IPs (192.168.x.x, 10.x.x.x)
  ├── Reject public IP access
  ├── Optional: password protect (user set trong Agent settings)
  └── HTTPS với self-signed cert (hoặc HTTP localhost only)
```

### 15.3 Data Encryption

```
┌─ At Rest ────────────────────────────────────────────────────────┐
│                                                                   │
│  Agent SQLite (Tầng 3):                                           │
│    SQLCipher — transparent AES-256-CBC encryption                 │
│    Key derived từ: user password + PBKDF2 (100k iterations)       │
│    Mở DB cần password → nếu ai copy file SQLite = không đọc được │
│                                                                   │
│  Cloud PostgreSQL:                                                │
│    PostgreSQL TDE (Transparent Data Encryption)                   │
│    Disk encryption (LUKS / cloud provider encryption)             │
│                                                                   │
│  Cloud Tầng 2 (E2E):                                             │
│    Đã encrypted trước khi gửi lên → double encryption             │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘

┌─ In Transit ─────────────────────────────────────────────────────┐
│                                                                   │
│  Agent ↔ Cloud:     TLS 1.3 (WSS)                                │
│  Web/Mobile ↔ Cloud: TLS 1.3 (HTTPS)                             │
│  Agent ↔ Groq API:  TLS 1.3 (HTTPS) + anonymized payload        │
│  LAN access:        Optional TLS with self-signed cert            │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

---

## 16. E2E Encryption Protocol (Tầng 2)

### 16.1 Key Management

```
┌─ Key Generation (1 lần khi user đăng ký) ───────────────────────┐
│                                                                   │
│  1. Agent generate: master_key = random 256 bits                  │
│  2. Derive encryption key:                                        │
│     enc_key = HKDF-SHA256(master_key, salt="haviz-e2e-enc")      │
│  3. Lưu master_key vào OS Keychain (encrypted by OS)              │
│     macOS: security add-generic-password -s "haviz" -a "user_id" │
│     Windows: CredWrite("haviz/user_id", master_key)              │
│  4. Backup key: hiện cho user 1 lần → user tự ghi lại            │
│     "Mã khôi phục: XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"               │
│     (BIP39-style mnemonic hoặc base64 encoded)                   │
│                                                                   │
│  master_key KHÔNG BAO GIỜ gửi lên server                         │
│  Haviz Cloud KHÔNG CÓ khả năng decrypt Tầng 2                    │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

### 16.2 Encrypt / Decrypt Flow

```
ENCRYPT (Agent → Cloud):
  1. data = { contact_name: "Nguyễn Văn A", preview: "Chào chị..." }
  2. plaintext = JSON.stringify(data)
  3. nonce = random 12 bytes (unique per message)
  4. ciphertext = AES-256-GCM.encrypt(enc_key, nonce, plaintext)
  5. Upload: { encrypted_blob: ciphertext, nonce: nonce }

DECRYPT (Agent ← Cloud):
  1. Download: { encrypted_blob, nonce }
  2. plaintext = AES-256-GCM.decrypt(enc_key, nonce, encrypted_blob)
  3. data = JSON.parse(plaintext)

Web App (remote access):
  1. Agent online → Web App request key via WSS tunnel
  2. Agent gửi enc_key encrypted với session key (Diffie-Hellman)
  3. Web App decrypt Tầng 2 data trong browser memory
  4. Key KHÔNG persist trong browser (chỉ in-memory)
  5. Tab đóng → key mất → cần Agent lại
```

### 16.3 Key Rotation

```
Mỗi 90 ngày (hoặc user trigger thủ công):
  1. Agent generate: new_master_key
  2. Derive: new_enc_key
  3. Download tất cả encrypted_blob từ Cloud
  4. Re-encrypt: decrypt(old_key) → encrypt(new_key)
  5. Upload lại tất cả + update nonce
  6. Lưu new_master_key vào Keychain, xóa old
  7. Thời gian: ~vài giây cho 1000 records
```

### 16.4 Key Recovery

```
Khi mất máy / cài lại OS / máy mới:

Option 1: Recovery phrase
  User nhập mã khôi phục (đã ghi lại khi đăng ký)
  → Derive master_key từ phrase
  → Decrypt tất cả Tầng 2 data

Option 2: Device-to-device transfer
  Máy cũ còn hoạt động:
  1. Mở Agent trên máy cũ → "Transfer keys"
  2. Hiện QR code chứa master_key (encrypted với OTP)
  3. Agent máy mới scan QR → nhập OTP → nhận key

Option 3: Key mất hoàn toàn (không recovery phrase, không máy cũ)
  → Tầng 2 data mất vĩnh viễn (Haviz KHÔNG thể recover)
  → Tầng 1 data (templates, settings) vẫn còn
  → Tầng 3 data mất cùng máy cũ
  → User bắt đầu lại từ đầu (Agent tạo key mới)
  → ĐÂY LÀ TRADE-OFF của E2E: bảo mật = không recovery
```

---

## 17. Offline-first Sync Protocol

### 17.1 Sync Strategy: Last-Write-Wins + Append-Only

```
Haviz KHÔNG dùng CRDT (quá phức tạp cho use case này).
Thay vào đó, phân loại data theo conflict strategy:

┌─ APPEND-ONLY (không bao giờ conflict) ──────────────────────────┐
│  Messages:     chỉ INSERT, không UPDATE/DELETE                   │
│  AI Drafts:    chỉ INSERT, status chuyển 1 chiều                │
│  Audit logs:   chỉ INSERT                                        │
│  Analytics:    chỉ INSERT (metrics mỗi giờ/ngày)                │
│                                                                   │
│  → Sync đơn giản: Agent gửi records mới → Server INSERT          │
│  → Không bao giờ conflict                                         │
└───────────────────────────────────────────────────────────────────┘

┌─ LAST-WRITE-WINS (hiếm conflict) ──────────────────────────────┐
│  Contacts:      updated_at timestamp → mới nhất thắng           │
│  Conversations: status, tags → mới nhất thắng                   │
│  Templates:     content, patterns → mới nhất thắng              │
│  Settings:      org/user config → mới nhất thắng                │
│                                                                   │
│  → Conflict: 2 nơi edit cùng lúc (rất hiếm)                    │
│  → Giải quyết: updated_at lớn hơn thắng                         │
│  → Edge case: Agent offline 2 ngày, quay lại → so sánh timestamp│
└───────────────────────────────────────────────────────────────────┘
```

### 17.2 Sync Flow khi Agent offline → online

```
Agent mất mạng lúc 14:00, online lại lúc 16:00:

14:00-16:00 (offline):
  ├── Agent vẫn đọc Zalo bình thường (AX API = local)
  ├── Lưu messages vào SQLite (bình thường)
  ├── AI drafts tạo local (nếu có internet cho Groq)
  │   └── Không có internet → queue draft requests
  ├── Metrics tính local (đếm messages, response time)
  └── Tất cả sync events queue trong: sync_queue table

16:00 (online lại):
  1. Agent detect network restored
  2. Đọc sync_queue (FIFO):
     ├── 45 new messages metadata → encrypt → upload Tầng 2
     ├── 3 new contacts → encrypt → upload Tầng 2
     ├── Hourly metrics (14h, 15h) → upload Tầng 1
     └── 5 AI draft statuses → encrypt → upload Tầng 2
  3. Download updates từ Cloud:
     ├── New templates (team member added)
     ├── Updated org settings
     └── Messages từ Cloud channels (OA, Messenger)
  4. Resolve conflicts (last-write-wins trên updated_at)
  5. Clear sync_queue
  6. Log: "Sync completed: 45 msgs, 3 contacts, 2h metrics"

Sync queue table (Agent SQLite):
  sync_queue (
    id INTEGER PRIMARY KEY,
    entity_type TEXT,     -- 'message_meta' | 'contact' | 'metric' | 'draft_status'
    entity_id TEXT,
    action TEXT,          -- 'insert' | 'update'
    payload_encrypted BLOB,
    created_at TIMESTAMP,
    synced_at TIMESTAMP   -- NULL until synced
  )
```

### 17.3 Conflict Resolution UI

```
Rất hiếm, nhưng khi xảy ra:

┌─ Conflict Detected ─────────────────────────────────────────────┐
│                                                                  │
│  Template "Báo giá tour" có 2 phiên bản:                        │
│                                                                  │
│  Local (14:30):                    Cloud (15:00):                │
│  "Giá tour Đà Lạt:                "Giá tour Đà Lạt:            │
│   3.500.000đ/người"                3.200.000đ/người (sale)"     │
│                                                                  │
│  [Giữ Local]  [Giữ Cloud]  [Giữ cả hai]                        │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

Mặc định (auto): Cloud thắng (updated_at mới hơn)
User có thể bật: "Hỏi tôi khi conflict" trong Settings
```

---

## 18. Deployment & DevOps

### 18.1 Infrastructure

```
┌─ Production Stack ──────────────────────────────────────────────┐
│                                                                  │
│  ┌─ VN Cloud (FPT Cloud / VNG Cloud) ────────────────────────┐  │
│  │                                                            │  │
│  │  API Server:                                               │  │
│  │    Docker container (Hono + Node.js)                       │  │
│  │    2 vCPU, 4GB RAM (scale horizontally)                    │  │
│  │    Auto-scaling: 2-10 instances behind load balancer       │  │
│  │                                                            │  │
│  │  PostgreSQL:                                               │  │
│  │    Managed PostgreSQL (VNG Cloud DB / FPT RDS)             │  │
│  │    2 vCPU, 4GB RAM, 50GB SSD                              │  │
│  │    Daily automated backup, point-in-time recovery          │  │
│  │    Read replica cho analytics queries                      │  │
│  │                                                            │  │
│  │  Redis:                                                    │  │
│  │    Managed Redis (1GB, cluster mode)                       │  │
│  │    BullMQ queues + rate limiting + session cache           │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌─ Vercel ──────────────────────────────────────────────────┐  │
│  │  Next.js Web App (auto-deploy from git)                    │  │
│  │  CDN: edge caching cho static assets                       │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌─ User's Machine ─────────────────────────────────────────┐   │
│  │  Rust Agent + SQLite (self-contained, no dependency)       │   │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 18.2 CI/CD Pipeline

```
┌─ Git Push ──► GitHub Actions ──────────────────────────────────┐
│                                                                 │
│  apps/api (push to main):                                       │
│    1. pnpm install + type-check                                 │
│    2. pnpm test (vitest)                                        │
│    3. Docker build + push → VN Cloud Registry                   │
│    4. Deploy staging → smoke test (health check + key endpoints)│
│    5. Manual approve → deploy production (blue-green)           │
│    6. Rollback nếu health check fail trong 5 phút              │
│                                                                 │
│  apps/web (push to main):                                       │
│    1. pnpm install + type-check                                 │
│    2. pnpm test                                                 │
│    3. Vercel auto-deploy (preview → production)                 │
│                                                                 │
│  agent/ (push to main):                                         │
│    1. cargo check + clippy                                      │
│    2. cargo test                                                │
│    3. Cross-compile: x86_64-apple-darwin, aarch64-apple-darwin  │
│       + x86_64-pc-windows-msvc                                  │
│    4. Code signing (Apple Developer ID + Windows Authenticode)  │
│    5. Upload binaries → S3 (versioned)                          │
│    6. Update version manifest: api.haviz.vn/agent/version       │
│    7. Agents auto-update (OTA, xem Section 10.8)                │
│                                                                 │
│  Database migrations:                                           │
│    1. drizzle-kit generate                                      │
│    2. Review SQL diff                                           │
│    3. Apply staging → verify                                    │
│    4. Apply production (trong maintenance window nếu breaking)  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 18.3 Environments

```
┌──────────────┬──────────────────────────────────────────────────┐
│ Environment  │ Mục đích                                        │
├──────────────┼──────────────────────────────────────────────────┤
│ local        │ Dev machine: docker-compose (PG + Redis)        │
│              │ Agent: cargo run                                 │
│              │ Web: next dev                                    │
├──────────────┼──────────────────────────────────────────────────┤
│ staging      │ VN Cloud: staging.api.haviz.vn                  │
│              │ Separate DB + Redis                              │
│              │ Seed data: 5 test users, 100 conversations      │
│              │ Agent connects to staging API                    │
├──────────────┼──────────────────────────────────────────────────┤
│ production   │ VN Cloud: api.haviz.vn                          │
│              │ Blue-green deployment                            │
│              │ DB backup mỗi 6 giờ                             │
│              │ Monitoring active                                │
└──────────────┴──────────────────────────────────────────────────┘
```

### 18.4 Monitoring Stack

```
┌─ Observability ─────────────────────────────────────────────────┐
│                                                                  │
│  Logging:                                                        │
│    API Server → Pino (structured JSON) → Grafana Loki            │
│    Agent → tracing crate → local log file + rotate               │
│                                                                  │
│  Metrics:                                                        │
│    API Server → Prometheus metrics endpoint                      │
│    ├── http_request_duration_seconds                              │
│    ├── ws_connections_active                                      │
│    ├── bullmq_queue_depth                                        │
│    ├── ai_draft_latency_seconds                                  │
│    └── agent_heartbeat_last_seen                                 │
│    → Grafana dashboards                                          │
│                                                                  │
│  Error tracking:                                                 │
│    API + Web → Sentry (error capture + tracing)                  │
│    Agent → Sentry Rust SDK (opt-in, user consent)                │
│                                                                  │
│  Uptime:                                                         │
│    BetterUptime / UptimeRobot                                    │
│    ├── api.haviz.vn/health (API)                                 │
│    ├── haviz.vn (Web)                                            │
│    └── Alert → Slack/Telegram/Email khi down                     │
│                                                                  │
│  Alerts (Grafana):                                               │
│    ├── API error rate > 5% → Slack alert                         │
│    ├── Queue depth > 1000 → Slack alert                          │
│    ├── Agent disconnect > 50% of active → Slack alert            │
│    ├── DB connection pool exhausted → PagerDuty                  │
│    └── Disk usage > 80% → Slack alert                            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 19. Agent Distribution

### 19.1 Packaging

```
┌─ macOS ──────────────────────────────────────────────────────────┐
│                                                                   │
│  Format: .dmg (drag to Applications)                              │
│  Size: ~8-12MB (Rust binary + embedded webview resources)         │
│  Signing: Apple Developer ID (notarized)                          │
│    → Không bị Gatekeeper chặn                                    │
│    → "Haviz.app is from an identified developer"                  │
│                                                                   │
│  Cài đặt:                                                        │
│    1. Download haviz-agent-macos-arm64.dmg (hoặc x64)            │
│    2. Drag Haviz.app → Applications                              │
│    3. First launch: macOS hỏi Accessibility permission            │
│       "Haviz muốn điều khiển máy tính của bạn"                   │
│       → System Preferences → Privacy → Accessibility → Allow     │
│    4. Login qua browser → Agent nhận token                        │
│    5. Tray icon xuất hiện → Agent chạy background                │
│                                                                   │
│  Auto-start: LaunchAgent plist                                    │
│    ~/Library/LaunchAgents/vn.haviz.agent.plist                   │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘

┌─ Windows ────────────────────────────────────────────────────────┐
│                                                                   │
│  Format: .msi installer (hoặc .exe NSIS)                          │
│  Size: ~10-15MB                                                   │
│  Signing: Authenticode certificate (EV recommended)               │
│    → Windows SmartScreen trust                                    │
│                                                                   │
│  Cài đặt:                                                        │
│    1. Download haviz-agent-windows-x64.msi                        │
│    2. Run installer → chọn install path                           │
│    3. Login qua browser                                           │
│    4. Tray icon xuất hiện                                         │
│                                                                   │
│  Auto-start: Registry key                                         │
│    HKCU\Software\Microsoft\Windows\CurrentVersion\Run             │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

### 19.2 Auto-Update (OTA)

```
Mỗi 6 giờ, Agent check:
  GET https://api.haviz.vn/agent/version
  Response: {
    latest: "1.2.0",
    url_macos_arm64: "https://dl.haviz.vn/agent/1.2.0/haviz-macos-arm64.tar.gz",
    url_macos_x64:   "https://dl.haviz.vn/agent/1.2.0/haviz-macos-x64.tar.gz",
    url_windows_x64: "https://dl.haviz.vn/agent/1.2.0/haviz-windows-x64.zip",
    checksum_sha256: "abc123...",
    changelog: "Bug fixes, improved AX API parsing",
    min_version: "1.0.0",     // Force update nếu < min_version
    optional: true             // true = user chọn, false = bắt buộc
  }

Update flow:
  1. Agent so sánh current vs latest version
  2. optional=true → notification: "Phiên bản mới 1.2.0. Cập nhật?"
     optional=false → "Cập nhật bắt buộc. Agent sẽ restart trong 5 phút."
  3. Download binary → verify SHA-256 checksum
  4. macOS: replace binary in .app bundle
     Windows: MSI silent upgrade
  5. Restart Agent
  6. Verify: Agent gửi agent:connect với version mới
  7. Rollback nếu crash trong 60s sau update:
     Giữ bản cũ (backup) → restore → báo cáo lỗi
```

### 19.3 Permissions Required

```
macOS:
  ├── Accessibility (bắt buộc): đọc AX API
  ├── Automation (bắt buộc): AppleScript control Zalo
  ├── Network (tự động): outbound connections
  └── Login Items (optional): auto-start

Windows:
  ├── UI Automation (tự động): Win32 API
  ├── Network (tự động): firewall allow
  └── Startup (optional): auto-start
```

---

## 20. Performance & Limits

### 20.1 Benchmarks (Expected)

```
┌─ Agent Performance ─────────────────────────────────────────────┐
│                                                                  │
│  AX API polling:                                                 │
│    Latency per poll: ~50-100ms (traverse Zalo window hierarchy)  │
│    Poll interval: 3s                                             │
│    CPU usage: ~1-2% (idle), ~5% (during poll)                    │
│    Memory: ~20-30MB RSS                                          │
│                                                                  │
│  SQLite performance:                                             │
│    INSERT message: <1ms                                          │
│    SELECT recent 100 messages: ~5ms                              │
│    Full-text search (FTS5): ~10-20ms per 10k messages            │
│    DB size: ~1MB per 1000 messages (text only)                   │
│    Practical limit: 1M+ messages (SQLite handles billions)       │
│                                                                  │
│  AI Draft generation:                                            │
│    Anonymize: <5ms                                               │
│    Groq API call: ~500-1500ms (network dependent)                │
│    Fill-back names: <1ms                                         │
│    Total: ~0.5-1.5s per draft                                    │
│                                                                  │
│  E2E Encryption:                                                 │
│    Encrypt 1 record: <0.1ms (AES-256-GCM, hardware accelerated) │
│    Encrypt 1000 records (batch sync): ~50ms                      │
│    Key derivation (HKDF): ~1ms                                   │
│                                                                  │
│  Send message (AppleScript):                                     │
│    Search + click + paste + send: 3-8s (includes human delays)   │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘

┌─ Server Performance ────────────────────────────────────────────┐
│                                                                  │
│  API response time (p95):                                        │
│    REST endpoints: <100ms                                        │
│    WebSocket message: <50ms                                      │
│                                                                  │
│  BullMQ throughput:                                              │
│    message:ingest: 100 msg/s per worker                          │
│    ai:draft (cloud): 10 draft/s (Groq rate limit)               │
│                                                                  │
│  WebSocket connections:                                          │
│    Per server: ~10,000 concurrent (Node.js + ws library)         │
│    With 2 servers: ~20,000 concurrent agents + web clients       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 20.2 Scaling Limits

```
┌────────────────────┬──────────────┬──────────────────────────────┐
│ Component          │ Limit        │ Scaling strategy              │
├────────────────────┼──────────────┼──────────────────────────────┤
│ Agent SQLite       │ 1M+ messages │ Archiving: move old → archive│
│                    │ per agent    │ DB after 6 months            │
│ Cloud PostgreSQL   │ 10M rows     │ Partitioning by org_id +     │
│                    │ per table    │ date. Read replica cho query │
│ Redis              │ 1GB          │ Cluster mode, evict old keys │
│ WebSocket          │ 10k/server   │ Horizontal: add servers      │
│ BullMQ             │ 100 msg/s    │ Add workers, separate queues │
│ Groq API           │ 30 req/min   │ Queue + batch, upgrade plan  │
│ Agent binary size  │ 8-15MB       │ Strip symbols, UPX compress  │
│ Concurrent Agents  │ 20k          │ Multiple API servers + LB    │
└────────────────────┴──────────────┴──────────────────────────────┘
```

### 20.3 SQLite Archiving Strategy

```
Khi Agent SQLite > 100MB (khoảng 100k messages):
  1. Agent detect: DB size > threshold
  2. Notification: "Tin nhắn cũ sẽ được archive"
  3. Move messages older than 6 months → archive_YYYY.sqlite
  4. Main DB chỉ giữ 6 tháng gần nhất
  5. Archive DB: read-only, searchable, same encryption
  6. User tìm tin nhắn cũ → Agent search across all DBs
```

---

## 21. Error Handling & Recovery

### 21.1 Agent Crash Recovery

```
┌─ Agent crash (unexpected termination) ──────────────────────────┐
│                                                                  │
│  1. SQLite: WAL mode → auto-recovery trên next start             │
│     Nếu crash giữa lúc write → WAL replay → data intact        │
│     Worst case: mất tin nhắn cuối cùng chưa commit              │
│                                                                  │
│  2. Agent restart:                                               │
│     Auto-start (LaunchAgent/Registry) → restart trong 5s        │
│     Reconnect WebSocket → Cloud                                  │
│     Resume polling Zalo                                          │
│     Process sync_queue (nếu có pending)                          │
│                                                                  │
│  3. Crash loop detection:                                        │
│     > 3 crashes trong 5 phút → dừng auto-restart                │
│     Notification: "Haviz gặp lỗi liên tục. Kiểm tra log."      │
│     Log file: ~/Library/Logs/Haviz/agent.log (macOS)             │
│                %APPDATA%\Haviz\logs\agent.log (Windows)          │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 21.2 SQLite Corruption Recovery

```
Phát hiện:
  Agent startup: PRAGMA integrity_check;
  Nếu fail → DB corrupted

Recovery:
  1. Thử repair: sqlite3 corrupt.db ".recover" | sqlite3 repaired.db
  2. Nếu repair thành công → replace DB, log warning
  3. Nếu repair fail:
     a. Có backup? → restore từ backup gần nhất
        (Agent auto-backup mỗi ngày: db_backup_YYYYMMDD.sqlite.enc)
     b. Không có backup? → tạo DB mới
        → Cloud Tầng 2 data vẫn còn (encrypted metadata)
        → Agent poll Zalo lại → rebuild messages từ Zalo history
        → Notification: "Database đã được khôi phục. Một số tin nhắn
          cũ có thể bị mất."

Prevention:
  ├── WAL mode (default): crash-safe writes
  ├── Daily backup: encrypted copy → ~/Haviz/backups/
  ├── Integrity check: mỗi lần Agent start
  └── Disk space check: cảnh báo khi < 500MB free
```

### 21.3 Mid-Send Failure

```
Agent đang gửi tin nhắn (AppleScript) → crash/disconnect:

Trước khi gửi:
  message.status = 'sending'
  message.send_attempt = attempt + 1

Sau khi restart:
  1. Check messages WHERE status = 'sending'
  2. Mở Zalo → kiểm tra conversation cuối
  3. Tin đã gửi thành công? (AX API đọc outbound message)
     → Mark status = 'sent'
  4. Tin chưa gửi?
     → send_attempt < 3? → retry
     → send_attempt >= 3? → mark 'failed' + notification user
  5. Không chắc chắn? → mark 'uncertain'
     → UI: "Không xác định tin nhắn đã gửi chưa. Kiểm tra Zalo."
```

### 21.4 Data Migration (Agent Version Upgrade)

```
Khi Agent update có DB schema change:

agent/migrations/
├── v1_0_0_initial.sql
├── v1_1_0_add_attachments.sql
├── v1_2_0_add_sync_queue.sql
└── v1_3_0_add_analytics.sql

Migration flow:
  1. Agent start → check current_version vs db_version
  2. current > db → run pending migrations sequentially
  3. Mỗi migration trong transaction:
     BEGIN;
     ALTER TABLE ...;
     UPDATE db_meta SET version = '1.2.0';
     COMMIT;
  4. Migration fail → ROLLBACK → Agent chạy version cũ
     → Alert: "Cập nhật database thất bại. Liên hệ support."
  5. Backup DB trước mỗi migration (tự động)

Rollback:
  Agent detect migration fail → restore backup
  Download Agent version cũ nếu binary cũng incompatible
```

---

## 22. Phase 2-3 Detailed Design

### 22.1 Phase 2: Intelligence (Tháng 3-4 2026)

#### Voice Reports

```
Salesperson ghi âm meeting/gọi điện:
  1. App mobile: record audio → save local
  2. Agent upload encrypted audio → Cloud (opt-in)
     HOẶC: transcribe local (Whisper.cpp trên device)
  3. Groq Whisper API: audio → transcript (anonymized)
  4. Groq LLM: transcript → structured data:
     {
       customer: "[Customer]",
       discussed: ["báo giá tour Đà Lạt", "ngày đi 25/3"],
       action_items: ["gửi báo giá", "book phòng khách sạn"],
       sentiment: "positive",
       follow_up_date: "2026-03-22"
     }
  5. Action items → auto-create tasks / reminders
  6. Transcript + extracted data → lưu local (Tầng 3)
```

#### Chatbot Training

```
Dùng historical messages để train chatbot tự động reply:

  1. Collect training data (local):
     Agent export: question → answer pairs từ SQLite
     Anonymize: xóa PII
     Filter: chỉ lấy successful conversations (có reply + positive outcome)

  2. Fine-tune model (optional, cloud):
     Upload anonymized QA pairs → fine-tune Llama model trên Groq
     HOẶC: dùng few-shot prompting (không cần fine-tune)

  3. Chatbot modes:
     Manual: AI draft → user approve (hiện tại)
     Semi-auto: AI draft → auto-send nếu confidence > 90%
                AI draft → require approval nếu confidence < 90%
     Full-auto: AI reply tự động (chỉ cho Zalo OA, cần consent rõ ràng)

  4. Confidence scoring:
     Template match: 95% confidence
     Similar to past Q&A: 80-95%
     Novel question: <80% → always require approval
```

#### Analytics Dashboard (Enhanced)

```
Ngoài basic metrics (đã có ở 8.5), thêm:

  Revenue tracking:
    - Gắn conversation → deal value (user input)
    - Revenue per salesperson
    - Conversion rate: conversation → sale
    - Pipeline visualization

  Team leaderboard:
    - Response time ranking
    - Customer satisfaction (sentiment analysis)
    - AI adoption rate (ai_drafts_approved / total_replies)

  Predictions (AI):
    - "Conversation X có 75% khả năng chốt deal" (based on sentiment + patterns)
    - "Khách Y chưa reply 3 ngày → suggest follow-up"
    - Weekly forecast: expected messages, peak hours
```

### 22.2 Phase 3: Platform (Tháng 5-6 2026)

#### REST API + SDK

```
Public API cho developers:
  POST /api/v1/public/messages/send
  GET  /api/v1/public/conversations
  GET  /api/v1/public/contacts
  POST /api/v1/public/templates

SDK:
  npm install @haviz/sdk
  pip install haviz-sdk

  const haviz = new Haviz({ apiKey: "hvz_..." });
  await haviz.messages.send({
    channel: "zalo_oa",
    to: "contact_id",
    content: "Hello!"
  });
```

#### MCP (Model Context Protocol)

```
Haviz as MCP Server:
  Cho phép AI agents (Claude, GPT) truy cập Haviz data:

  Tools:
    haviz_list_conversations(status, limit)
    haviz_read_messages(conversation_id, limit)
    haviz_send_message(conversation_id, content)
    haviz_search_contacts(query)
    haviz_get_analytics(date_range)

  Resources:
    haviz://conversations/{id}
    haviz://contacts/{id}
    haviz://templates

  Use case:
    Claude: "Tóm tắt 5 conversation gần nhất"
    → MCP → haviz_list_conversations → haviz_read_messages × 5
    → Claude tóm tắt

  Security:
    MCP chỉ truy cập Cloud channels (OA, Messenger)
    Local channels (Zalo cá nhân) → cần Agent online + user approve
```

#### White-label

```
Cho agency/partner bán dưới brand riêng:

  Config:
    brand_name: "SalesPro by Agency X"
    logo_url: "https://..."
    primary_color: "#FF6B00"
    domain: "inbox.agencyx.vn"

  Revenue:
    Agency trả Haviz: 50% of subscription
    Agency giữ: 50% + markup tùy ý

  Tech:
    Cùng infrastructure, khác UI theme + domain
    Multi-tenant: org.white_label_config JSONB
```
