# Haviz Development Roadmap

**Version:** 1.0
**Last Updated:** 2026-03-19
**Current Status:** Phase 1 - Core MVP (In Progress)

## Executive Summary

Haviz is a revenue intelligence platform transforming Vietnamese sales conversations into actionable insights. This roadmap outlines our path from MVP (Phase 1) to enterprise platform (Phase 3+).

**Timeline:**
- **Phase 0 (PoC):** ✓ Complete — Browser validation, agent prototype
- **Phase 1 (MVP):** 🔄 In Progress (Q1-Q2 2026) — Agent + Web UI + Extension
- **Phase 2 (Cloud Channels):** ⏳ Q2-Q3 2026 — Zalo OA, Messenger, Telegram, Mobile
- **Phase 3 (Analytics & Scale):** ⏳ Q4 2026+ — Advanced AI, SMS, Enterprise

## Phase 0: Proof of Concept (COMPLETE ✓)

**Timeline:** Q4 2025 - Q1 2026
**Status:** ✓ COMPLETE

**Objective:** Validate technical approaches for message reading and AI integration

### Deliverables

| Component | Status | Notes |
|-----------|--------|-------|
| Browser validation scripts | ✓ | Tested: Chrome, Servo, Lightpanda, Playwright |
| chrome-headless-shell CDP approach | ✓ CHOSEN | 558MB RAM, full Chrome support |
| macOS AX API prototype | ✓ | Working for Zalo Desktop |
| Groq Llama 4 Scout validation | ✓ | Cost-effective, fast inference |
| Agent prototype (Rust) | ✓ | Basic polling, message parsing |
| Extension prototype | ✓ | MutationObserver reading Zalo Web |

### Key Findings

1. **chrome-headless-shell** is best for cross-platform Zalo Web (vs Servo/Lightpanda)
2. **macOS AX API** works for Zalo Desktop (lower CPU than headless)
3. **Groq Llama 4 Scout** suitable for sales drafts (~200 tokens, $0.05/month per user)
4. **Local-first architecture** feasible with SQLite + cloud sync

## Phase 1: Core MVP (IN PROGRESS 🔄)

**Timeline:** Q1-Q2 2026 (9 weeks remaining)
**Status:** 🔄 IN PROGRESS

**Objective:** Deliver production-ready desktop agent + web UI for single-channel (Zalo Personal) multi-account support

### 1.1 Desktop Agent Finalization

**Current Status:** 80% complete
**LOC:** 2,139 (complete)

| Component | Status | Acceptance Criteria |
|-----------|--------|-------------------|
| `src/server.rs` | ✓ | ✓ HTTP server serving UI + REST API |
| `src/db.rs` | ✓ | ✓ SQLite schema complete, migrations working |
| `src/polling.rs` | 🔄 | Handle edge cases (network timeouts, Zalo updates) |
| `src/ai.rs` | 🔄 | Groq integration + error handling |
| `src/channels/zalo_web.rs` | 🔄 | CDP message reading + sending |
| `src/channels/zalo_desktop.rs` | 🔄 | AX API polling stable |
| `src/message_parser.rs` | ✓ | ✓ Parse 95%+ of message formats correctly |
| `src/bin/haviz_app.rs` | 🔄 | Multi-account UI, tray icon, auto-start |
| Safety engine (5 layers) | ⏳ | Rate limiting, human-like delays, working hours |
| Account health monitoring | ⏳ | Health score 0-100, warn at <20 |

**Deliverables:**

```
haviz-agent-1.0.0/
├── haviz-app.exe / haviz-app.dmg / haviz-app.AppImage
├── INSTALL.md
├── CHANGELOG.md
└── config/
    └── .env.example
```

**Success Criteria:**
- ✓ Cross-platform builds (Windows, macOS, Linux)
- ✓ Read 95%+ of Zalo messages without data loss
- ✓ <500ms message detection latency
- ✓ Support 5+ accounts per user
- ✓ Zero memory leaks over 24-hour run
- ✓ <200MB memory footprint typical

### 1.2 Web UI Implementation

**Current Status:** 70% complete
**LOC:** 574 (complete)

| Component | Status | Acceptance Criteria |
|-----------|--------|-------------------|
| InboxView | 🔄 | Virtual scroll 1000+ messages, real-time updates |
| Sidebar | ✓ | ✓ Conversation list, account switcher |
| Topbar | ✓ | ✓ Status indicator, settings |
| Draft panel | 🔄 | Edit, approve, reject, regenerate buttons |
| API client | ✓ | ✓ REST communication with agent |
| Stores | ✓ | ✓ Svelte reactive state management |
| Styling | 🔄 | Dark mode, responsive mobile view |
| Keyboard shortcuts | ⏳ | Cmd/Ctrl+Enter send, arrow keys navigate |

**Deliverables:**
- Embedded in `haviz_app.rs` webview
- Static files in `agent/static/`
- Build: `pnpm build` → `dist/` folder

**Success Criteria:**
- ✓ Dashboard loads in <1s
- ✓ Real-time message updates (WebSocket or polling)
- ✓ Edit draft without lag
- ✓ Mobile-responsive (90% viewport width)
- ✓ Accessible (WCAG AA)
- ✓ Support 500+ conversations without slowdown

### 1.3 Chrome Extension

**Current Status:** 60% complete
**LOC:** 106 (complete)

| Component | Status | Acceptance Criteria |
|-----------|--------|-------------------|
| manifest.json | ✓ | ✓ Manifest V3 valid, minimal permissions |
| content script | ✓ | ✓ MutationObserver detects new messages |
| background worker | 🔄 | Aggregate messages, send to agent API |
| popup UI | ⏳ | Quick actions, unread count |
| message batching | ⏳ | Batch 50 messages per API call |

**Deliverables:**
```
haviz-extension-1.0.0.zip
├── manifest.json
├── src/
│   ├── content/zalo-reader.js
│   ├── background/service-worker.js
│   └── popup/popup.html
└── README.md
```

**Success Criteria:**
- ✓ Install on Chrome 90+ without warnings
- ✓ Read 95%+ of Zalo Web messages
- ✓ <5MB extension size
- ✓ No memory leaks over 8-hour session

### 1.4 Testing & Quality Assurance

**Current Status:** 20% complete

| Item | Status | Target |
|------|--------|--------|
| Unit tests (Rust) | ⏳ | >80% coverage |
| Integration tests | ⏳ | Agent + DB + API |
| E2E tests (Cypress) | ⏳ | Critical user flows |
| Performance tests | ⏳ | Message read <500ms, send <2s |
| Security review | ⏳ | Credentials, API keys, data encryption |
| User testing (5-10 beta users) | ⏳ | Feedback on UX, feature prioritization |

**Deliverables:**
- Test suite: `cargo test`, `pnpm test`
- Performance report
- Security audit report

**Success Criteria:**
- ✓ All critical tests passing
- ✓ Zero security vulnerabilities (P0/P1)
- ✓ Performance metrics within targets
- ✓ Beta users >70% satisfaction (NPS 50+)

### 1.5 Documentation & Onboarding

**Current Status:** 50% complete

| Document | Status | Purpose |
|----------|--------|---------|
| README.md | ✓ | Quick start (root) |
| Installation guide | 🔄 | Step-by-step setup |
| User manual | ⏳ | Feature walkthroughs |
| API documentation | ⏳ | REST endpoints, auth, examples |
| Developer guide | ✓ | Code standards, architecture |
| Troubleshooting FAQ | ⏳ | Common issues & solutions |

**Deliverables:**
- All docs in `docs/` directory
- Help within app (tooltips, onboarding)
- Video tutorials (3-5 min each)

**Success Criteria:**
- ✓ New user setup <5 minutes
- ✓ All features documented
- ✓ FAQ covers 80% of support tickets

### 1.6 Deployment & Release

**Timeline:** End of Q2 2026

**Deliverables:**
- Windows installer (MSIX)
- macOS .dmg (signed & notarized)
- Linux AppImage
- Chrome extension (Web Store)
- Release notes & changelog
- Update mechanism (Tauri or similar)

**Success Criteria:**
- ✓ <5MB total install size (agent only)
- ✓ Auto-update working
- ✓ Code signing valid (no warnings)
- ✓ Crash reporting integrated

---

## Phase 2: Cloud Channels (Q2-Q3 2026)

**Timeline:** 8-12 weeks (after Phase 1 MVP launch)
**Objective:** Multi-channel support (Zalo OA, Messenger, Telegram) + mobile app + cloud backend

### 2.1 Backend API (Hono + Drizzle)

**Scope:**
- User authentication (Supabase)
- Template management API
- Webhook receiver for cloud channels
- Analytics aggregation
- Rate limiting & auth middleware
- WebSocket for real-time sync

**Key Endpoints:**
```
POST   /api/auth/signup
POST   /api/auth/login
POST   /api/templates
GET    /api/templates
PUT    /api/templates/:id
DELETE /api/templates/:id

POST   /api/channels/:id/webhook  (Zalo OA, Messenger)
GET    /api/analytics/daily
GET    /api/analytics/summary

WebSocket /ws/:userId (real-time updates)
```

**Success Criteria:**
- ✓ API response <200ms (p95)
- ✓ Support 1000 concurrent users
- ✓ 99.9% uptime
- ✓ Secure auth (no credential leaks)

### 2.2 Zalo OA Channel

**Scope:**
- Official Account webhook integration
- Multi-OA support
- Message delivery confirmation
- Conversation sync to local agent

**Flow:**
```
Customer → Zalo OA → Webhook → Haviz Backend → Agent → SQLite
↑                                                           ↓
←───────────────── User approves draft ←────────────────←
```

**Success Criteria:**
- ✓ Read OA messages in <3s
- ✓ Send with <2s latency
- ✓ Support 100+ OAs per user
- ✓ Webhook delivery 99.99% reliability

### 2.3 Messenger & Telegram Channels

**Scope:**
- Facebook Graph API integration
- Telegram Bot API integration
- Unified inbox across channels
- Channel-specific templates

**Success Criteria:**
- ✓ Messenger working end-to-end
- ✓ Telegram working end-to-end
- ✓ Channel switching seamless
- ✓ Message ordering consistent across channels

### 2.4 Mobile App (React Native/Expo)

**Scope:**
- Review conversations on the go
- Approve/reject drafts
- Quick reply via mobile
- Push notifications
- Offline message caching

**Tech Stack:**
- React Native + Expo
- React Query for server sync
- Zustand for state
- native-base or Tamagui for UI

**Success Criteria:**
- ✓ iOS + Android support
- ✓ App store ready
- ✓ <100MB app size
- ✓ 99%+ uptime
- ✓ Offline capability

### 2.5 Cloud Metadata Sync

**Scope:**
- Encrypted sync of conversation metadata
- Multi-device access (future)
- AES-256 encryption (user holds key)
- PostgreSQL backend

**Success Criteria:**
- ✓ Metadata synced within 10s
- ✓ E2E encryption unbreakable
- ✓ Support 5+ devices per user
- ✓ <10ms data access latency

---

## Phase 3: Advanced Features (Q4 2026+)

**Timeline:** 12+ weeks
**Objective:** Enterprise-ready analytics, advanced AI, SMS, scale to 10k+ users

### 3.1 Analytics Dashboard

**Features:**
- Conversation metrics (count, avg response time)
- Reply rate tracking
- Conversion funnel (view → purchase)
- Team performance leaderboard (if team feature)
- Export reports (CSV, PDF)
- Real-time dashboard

**Tech Stack:**
- Svelte or React for web UI
- PostreSQL + Grafana for analytics
- Metabase for self-service BI

**Success Criteria:**
- ✓ Dashboard load <1s
- ✓ Real-time updates (streaming)
- ✓ Export in <5s
- ✓ Support 100+ metrics

### 3.2 Advanced AI Features

**Features:**
- Multi-turn conversation context
- Customer sentiment analysis
- Product recommendation engine
- Churn prediction
- Custom model fine-tuning (per-user)

**Tech Stack:**
- Fine-tuned Llama model
- Vector embeddings (Qdrant/Weaviate)
- RAG pipeline for knowledge base

**Success Criteria:**
- ✓ Drafts 90%+ accurate
- ✓ Sentiment detection >85% F1
- ✓ Recommendation CTR 5%+

### 3.3 SMS & Phone Channel

**Features:**
- Twilio/VNPT SMS integration
- IVR voice support (future)
- Phone number management

**Success Criteria:**
- ✓ SMS read/send working
- ✓ Sub-1s message latency
- ✓ Support 10k+ phone numbers

### 3.4 Team Management (Enterprise)

**Features:**
- Multi-user accounts
- Role-based access control (RBAC)
- Audit logs
- Custom workflows
- SLA monitoring

**Success Criteria:**
- ✓ Support 10-100 team members
- ✓ Audit trail 100% accurate
- ✓ SLA tracking functional

### 3.5 Scale to 10k+ Users

**Infrastructure:**
- Multi-region deployment (Asia, Americas)
- Database sharding
- Load balancing
- CDN for static assets
- Kubernetes orchestration

**Success Criteria:**
- ✓ 99.99% uptime
- ✓ <200ms API latency (global)
- ✓ 1M messages/day throughput
- ✓ <5s report generation time

---

## Success Metrics & KPIs

### User Adoption (Phase 1 → 3)

| Metric | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|
| DAU | 50+ | 500+ | 5,000+ |
| MAU | 100+ | 2,000+ | 20,000+ |
| Retention (30d) | 60%+ | 70%+ | 75%+ |
| NPS Score | 50+ | 55+ | 60+ |

### Product Quality

| Metric | Target | Monitoring |
|--------|--------|-----------|
| Uptime | 99.9%+ | StatusPage |
| Message read accuracy | 95%+ | Manual testing, user reports |
| Draft approval rate | 70%+ | Analytics |
| Crash-free hours | 99%+ | Sentry |
| Support response time | <2h | Slack/Email |

### Business Impact

| Metric | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|
| Revenue/user/month | Free beta | $5-50 | $50-500 |
| Customer acquisition cost | — | <$10 | <$20 |
| Lifetime value | — | $500+ | $5,000+ |
| Churn rate | — | <5%/month | <2%/month |

### User Satisfaction

| Metric | Target | Method |
|--------|--------|--------|
| CSAT | 80%+ | Post-interaction survey |
| Feature completeness | 90%+ | Feature request tracking |
| Documentation clarity | 85%+ | Help content feedback |
| Support satisfaction | 85%+ | Support ticket survey |

---

## Risk Management

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Zalo updates break reading | High | High | Maintain validation scripts, rapid response team |
| Account bans due to spam behavior | Medium | High | Safety engine, beta testing, compliance review |
| Performance degradation at scale | Low | Medium | Load testing, database optimization, caching |
| Data breach of encrypted metadata | Low | Critical | Security audit, penetration testing, incident response |

### Market Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Competitor entry | Medium | High | Focus on product quality, build moat (UX, integrations) |
| Zalo policy changes | Medium | High | Maintain relationships, legal review, diversify channels |
| Low user adoption | Low | High | User research, marketing strategy, PMF validation |

### Execution Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Team scaling bottleneck | Medium | Medium | Documentation, code quality, hiring contractors |
| Dependency vulnerabilities | Low | Medium | Regular audits, dependency updates, security scanning |
| Scope creep in Phase 1 | High | Medium | Strict feature freeze, MoSCoW prioritization |

---

## Dependencies & Constraints

### External Dependencies

| Dependency | Phase | Risk | Fallback |
|-----------|-------|------|----------|
| Groq API availability | 1+ | Low | Template-only mode, manual replies |
| Zalo Web accessibility | 1+ | High | AX API (macOS only), manual updates |
| Chrome DevTools Protocol | 1+ | Low | Revert to Playwright if CDP breaks |
| Supabase (auth) | 2+ | Low | Self-hosted auth, Firebase backup |

### Team Constraints

- **Rust expertise:** Required for agent (current: 1-2 engineers)
- **Svelte expertise:** Nice-to-have (Tauri course planned)
- **Backend expertise:** Needed Phase 2 (hire if needed)
- **Mobile expertise:** Needed Phase 2 (hire or partner)

### Budget Constraints

| Phase | Cost | Source |
|-------|------|--------|
| Phase 1 | $50k | (salaries, infra, API) |
| Phase 2 | $150k | Seed funding |
| Phase 3 | $500k | Series A |

---

## Milestone Timeline

```
Q1 2026 (Jan-Mar):
  Week 1-2:   Phase 1 design finalization
  Week 3-6:   Agent development
  Week 7-10:  Web UI + Extension
  Week 11-13: Testing & bug fixes

Q2 2026 (Apr-Jun):
  Week 1-4:   Phase 1 stabilization & release
  Week 5-13:  Phase 2 planning & backend setup

Q3 2026 (Jul-Sep):
  Week 1-8:   Cloud channels (Zalo OA, Messenger, Telegram)
  Week 9-13:  Mobile app development

Q4 2026+ (Oct+):
  Month 1-3:  Analytics dashboard & advanced AI
  Month 4+:   Scale, enterprise features, international
```

---

## Decision Log

| Date | Decision | Rationale |
|------|----------|----------|
| 2026-03-19 | Svelte 5 for web UI | Simpler than React, smaller bundle, good for desktop |
| 2026-03-19 | chrome-headless-shell for Zalo Web | Validates better than Servo/Lightpanda, cross-platform |
| 2026-03-19 | Groq Llama 4 Scout for AI | Cost-effective ($0.05/month), sufficient quality |
| 2026-03-19 | Local-first architecture | Privacy, compliance, offline capability |
| 2026-03-19 | Monorepo (pnpm + Turborepo) | Shared dependencies, easier development |

---

## Review Cadence

- **Weekly:** Team standups (Phase status, blockers)
- **Bi-weekly:** Product review (feature progress vs roadmap)
- **Monthly:** Stakeholder update (metrics, risks, next month plan)
- **Quarterly:** Roadmap review (update timeline, adjust scope)

---

## References

- `README.md` — Project overview
- `docs/project-overview-pdr.md` — Product requirements
- `docs/codebase-summary.md` — Current codebase state
- `docs/system-architecture.md` — Technical architecture
- ARCHITECTURE.md — Detailed Vietnamese architecture

---

**Last Updated:** 2026-03-19
**Next Review:** 2026-04-19 (monthly)
**Revision History:** See git log for changes
