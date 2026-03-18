---
name: multi_account_requirement
description: Multi-account multi-channel requirement — each salesperson has multiple Zalo, OA, Messenger, Telegram accounts, each with hundreds of conversations
type: project
---

Multi-account, multi-channel is a core requirement, not Phase 3 nice-to-have.

**Why:** A single salesperson may have 2-3 Zalo personal accounts, 1-2 Zalo OAs, 1 Messenger page, 1 Telegram bot. Each account has hundreds of conversations. Total could be 500-1000+ conversations per user.

**How to apply:**
- Phase 0: 1 Zalo WebView (current, for validation)
- Phase 1: + Zalo OA API (cloud, supports multi-OA natively)
- Phase 2: + Multi Zalo personal (multiple WebViews or browser profiles) + Messenger + Telegram
- Architecture Section 8.6 already covers this — ensure implementation follows hybrid local+cloud design
- Dashboard must support account switcher / unified inbox across all accounts
