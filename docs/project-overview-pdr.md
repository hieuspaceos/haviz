# Haviz — Product Development Requirements (PDR)

**Version:** 1.0
**Last Updated:** 2026-03-19
**Status:** Phase 1 - Core MVP (In Progress)

## Product Vision

**Tagline:** *"Biến mỗi cuộc trò chuyện thành doanh thu"* (Turn every conversation into revenue)

Haviz is a Revenue Intelligence Platform that empowers Vietnamese salespeople to sell more effectively through Zalo, Messenger, and SMS by leveraging AI-powered reply suggestions while maintaining complete message privacy.

## Mission Statement

Enable 1000+ Vietnamese salespeople to close more deals faster by providing intelligent conversation assistance that respects user privacy (messages never leave their machine) and builds trust through safety-first automation.

## Target Users

### Primary
- **Vietnamese B2B/B2C salespeople** who manage customer conversations across multiple platforms
- **Sales teams** in e-commerce, real estate, automotive, fintech
- **Account managers** managing 50-500+ active conversations per person
- **Small-to-medium enterprises (SMEs)** across Vietnam

### Secondary
- **International sales teams** (future: English, Chinese, Thai language support)
- **Customer support teams** (future: support module)
- **Marketing teams** (future: campaign automation)

### User Profiles

| Profile | Size | Pain Point | Haviz Solves |
|---------|------|-----------|--------------|
| **Solo Seller** | 5-50 customers | "I reply the same thing 100 times/day, lose customers due to slow response" | Auto-suggest replies, 2s response time |
| **Sales Team Lead** | 5-20 salespeople | "Can't monitor all conversations, no visibility into team quality" | Unified dashboard, AI assist for entire team |
| **Account Manager** | 50-200 accounts | "Switching between apps kills productivity" | Single inbox for Zalo + Messenger + SMS |
| **Enterprise Sales Org** | 50-500 salespeople | "Compliance, consistency, and scale across teams" | Safety engine, audit logs, templates |

## Core Value Proposition

1. **Save Time** — 2-5 hours/day per salesperson (auto-complete 80% of routine replies)
2. **Increase Revenue** — 15-25% higher reply rate → higher conversion
3. **Maintain Privacy** — Messages never leave machine (GDPR/PDPA compliant)
4. **Reduce Risk** — Safety engine prevents spam/account bans
5. **Stay Compliant** — Audit trail of all AI-assisted messages

## Functional Requirements

### Phase 1 (MVP) — Current

#### 1. Message Reading
- **Zalo Personal (Desktop)** — Read messages via macOS Accessibility API (AX API)
- **Zalo Personal (Web)** — Read messages via chrome-headless-shell + CDP
- **Chrome Extension** — Content script reads Zalo Web messages
- **Polling Interval** — 3-second near real-time polling
- **Multi-Account** — Support 2-5 accounts per user

**Success Criteria:**
- ✓ Read 95%+ of Zalo messages without losing data
- ✓ <500ms latency to detect new message
- ✓ Handle conversation history (scroll, pagination)

#### 2. AI Reply Drafts
- **Template Matching** — First check if reply matches existing template (0 cost)
- **Fallback AI** — Call Groq Llama 4 Scout if no template match
- **Anonymized Input** — Never send message content to Groq (hash + metadata only)
- **User Control** — User must approve/edit/reject before sending

**Success Criteria:**
- ✓ Generate drafts in <10s
- ✓ Template match accuracy >80% for common scenarios
- ✓ User approval rate >70% without edits

#### 3. Message Sending
- **Zalo Web** — Send via chrome-headless-shell automation
- **Human-like Behavior** — Typing delays, random waits (not instant)
- **User Confirmation** — UI checkbox "Send" with visual feedback
- **Timing** — Send during business hours only

**Success Criteria:**
- ✓ Send success rate >99%
- ✓ No account flags or warnings
- ✓ Delivery confirmation within 2s

#### 4. Web Dashboard
- **Conversation List** — All conversations with unread count
- **Message Thread** — Full chat history with sender/timestamp
- **Draft Panel** — AI draft suggestions, edit, approve/reject buttons
- **Sidebar Navigation** — Quick access to all accounts
- **Status Indicators** — Online/offline, message delivery status

**Success Criteria:**
- ✓ Dashboard loads in <1s
- ✓ Real-time message updates
- ✓ Mobile-responsive design

#### 5. Chrome Extension
- **Manifest V3** — Latest security standard
- **Content Script** — Read Zalo Web messages via MutationObserver
- **Popup UI** — Show unread count, quick links
- **Background Service Worker** — Periodic message fetch

**Success Criteria:**
- ✓ Extension installs on Chrome 90+
- ✓ Read messages without breaking Zalo Web
- ✓ <5MB extension size

#### 6. Account Management
- **Multi-Account Switch** — Dropdown to select active account
- **Per-Account Settings** — Templates, working hours, style
- **Logout/Login** — Manual account management
- **Account Health** — Show account ban risk score (0-100)

**Success Criteria:**
- ✓ Support 5+ accounts per user
- ✓ Switch accounts in <1s
- ✓ Per-account settings saved locally

### Phase 2 (Cloud Channels) — Q2-Q3 2026

- **Zalo OA (Official Account)** — Cloud API integration
- **Messenger** — Facebook Graph API
- **Telegram** — Telegram Bot API
- **Cloud Database** — PostgreSQL for templates, analytics
- **User Authentication** — Supabase auth

### Phase 3 (Advanced) — Q4 2026+

- **Analytics Dashboard** — Conversations, reply rate, conversion metrics
- **Mobile App** — React Native for on-the-go management
- **SMS/Phone** — Twilio/VNPT integration
- **Advanced AI** — Multi-turn conversations, context understanding
- **Enterprise Features** — Team management, audit logs, compliance

## Non-Functional Requirements

### Performance
| Metric | Target | Acceptable |
|--------|--------|-----------|
| Message read latency | <500ms | <2s |
| Draft generation | <10s | <15s |
| Dashboard load time | <1s | <2s |
| Send confirmation | <2s | <5s |
| Agent memory usage | <200MB | <500MB |

### Reliability
- **Uptime** — 99%+ for cloud services (Phase 2+)
- **Message Loss** — 0% (all messages stored locally)
- **Data Sync** — Eventually consistent (encrypted metadata)

### Security
- **Message Storage** — Local SQLite, no cloud sync
- **Metadata Encryption** — AES-256 for cloud (Phase 2+)
- **API Authentication** — Token-based, rotated monthly
- **Account Health** — Monitor for suspicious activity

### Scalability
- **Conversations per User** — Support 1000+ conversations
- **Messages per Conversation** — Unlimited (streamed from SQLite)
- **Concurrent Users** — Start with 100, scale to 10k+
- **Daily API Calls** — ~5-10 per user (Groq, Zalo OA)

### Compliance
- **GDPR** — Only metadata in EU regions
- **PDPA** — Local storage in Vietnam
- **Anti-Spam** — Built-in rate limiting & safety checks
- **Terms of Service** — Must comply with Zalo, Messenger, Telegram ToS

## Success Metrics (Phase 1)

### User Adoption
- **Target:** 100+ active users in first 3 months
- **Metric:** Daily Active Users (DAU) > 50% of registered users

### Product Quality
- **Message Accuracy:** 95%+ read rate
- **Draft Quality:** 70%+ user approval (without edits)
- **System Uptime:** 99%+

### Business Impact
- **Average Time Saved:** 2-5 hours/day per user
- **Revenue Impact:** Measure via user surveys (future: analytics)
- **Account Safety:** 0% account bans due to Haviz

### User Satisfaction
- **NPS Score:** 50+
- **Feature Requests:** Track top 5 most-requested features
- **Support Tickets:** <5 per 100 active users

## Technical Constraints

### Platform Constraints
- **Zalo Desktop** — Only works on Windows/macOS (not Linux)
- **Zalo Web** — Subject to anti-bot measures (chrome-headless-shell workaround)
- **Chrome Extension** — Manifest V3 only (V2 deprecated)
- **Electron vs wry** — Chose wry for smaller footprint

### API Constraints
- **Groq Rate Limits** — 30 calls/minute (sufficient for MVP)
- **Zalo OA API** — Requires official approval (Phase 2)
- **Supabase** — 500MB free tier (sufficient for MVP)

### Development Constraints
- **Rust Expertise** — Team knows Rust, fewer bugs vs Node.js backend
- **Svelte Learning Curve** — Smaller team, new framework
- **macOS AX API** — Requires Apple review for distribution
- **Browser Automation** — Fragile, may break with Zalo updates

## Architecture Decisions

### Why Local-First?
- **Privacy** — Vietnamese users value message privacy
- **GDPR/PDPA Compliance** — No message sync to cloud
- **Performance** — No network latency for common operations
- **Reliability** — Works offline, no server dependency

### Why Rust Agent?
- **Performance** — 10x faster than Node.js for I/O
- **System Integration** — Easy access to AX API, AppleScript
- **Automation** — Type-safe, fewer runtime errors
- **Deployment** — Single binary, auto-update ready

### Why Svelte 5?
- **Learning Curve** — Simpler than React/Vue
- **Bundle Size** — 15KB vs 40KB (React)
- **Reactivity** — Built-in stores, no Redux complexity
- **Desktop Friendly** — Works well in wry webview

### Why Groq Llama 4 Scout?
- **Cost** — ~$0.05/month per salesperson (vs $5/month for GPT-4)
- **Speed** — Sub-100ms latency
- **Privacy** — API calls anonymized (no user identifiers)
- **Quality** — Sufficient for sales reply drafts

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Zalo updates break message reading | High | High | Maintain browser validation suite, react quickly to updates |
| Account bans due to spam-like behavior | Medium | High | 5-layer safety engine, account health monitoring |
| Data privacy breach | Low | Critical | Local-first, encrypt metadata, no PII in cloud |
| User confusion on draft editing | Medium | Medium | Clear UI, tooltips, onboarding tutorial |
| Groq API outage | Low | Medium | Fall back to template matching, show error message |
| Team scaling bottleneck | Low | High | Hire contractors, clear code documentation |

## Dependencies & Assumptions

### External Dependencies
- **Groq API** — Available, rate limits sufficient
- **Zalo Web** — Remains accessible via chrome-headless-shell
- **Chrome DevTools Protocol** — Stable, no breaking changes
- **macOS Accessibility API** — Continues to work on latest macOS

### Team Assumptions
- **Rust expertise** — Team can maintain Rust agent
- **Svelte expertise** — Team learns Svelte 5 quickly
- **Vietnam market knowledge** — Understanding of local sales practices

### User Assumptions
- **Tech-savvy** — Comfortable installing desktop apps + extensions
- **Trust in Haviz** — Willing to give app access to Zalo conversations
- **Active Zalo users** — Send 5+ messages/day on average

## Success Criteria (End of Phase 1)

### Functional Completeness
- ✓ All features implemented & tested
- ✓ Zero critical bugs
- ✓ Cross-platform builds (Windows, macOS, Linux)

### Code Quality
- ✓ >80% test coverage
- ✓ All code follows standards in `docs/code-standards.md`
- ✓ Comprehensive documentation

### User Experience
- ✓ Onboarding flow <5 minutes
- ✓ All major workflows <3 clicks
- ✓ Help documentation for common issues

### Performance
- ✓ Agent runs on 8GB RAM with <200MB footprint
- ✓ Web UI responds to all actions in <500ms
- ✓ Draft generation <10s average

## Post-Phase-1 Roadmap

1. **Q2 2026** — Zalo OA + Messenger integration
2. **Q3 2026** — Mobile app (React Native)
3. **Q4 2026** — Analytics dashboard + SMS
4. **2027+** — Enterprise features, international expansion

## Appendix

### Glossary

| Term | Definition |
|------|-----------|
| **Local-First** | Messages stay on user's machine, only metadata synced |
| **E2E Encryption** | AES-256 encryption only user can decrypt |
| **AX API** | macOS Accessibility API for reading UI elements |
| **CDP** | Chrome DevTools Protocol for automation |
| **Draft** | AI-suggested reply text user can edit before sending |
| **Template** | Pre-written reply for common customer questions |
| **Account Health** | Risk score (0-100) for account ban likelihood |

### Resources

- `README.md` — Project overview & quick start
- `docs/codebase-summary.md` — File structure & tech stack
- `docs/code-standards.md` — Coding conventions
- `docs/system-architecture.md` — Architecture diagrams
- `ARCHITECTURE.md` — Detailed Vietnamese architecture

### Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-19 | Initial PDR created |

---

**For implementation details, see** `docs/system-architecture.md`
**For code standards, see** `docs/code-standards.md`
