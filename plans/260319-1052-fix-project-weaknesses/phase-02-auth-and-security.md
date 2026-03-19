---
phase: 2
title: "Auth & Security"
priority: CRITICAL
status: pending
effort: 16h
depends_on: [1]
---

# Phase 2: Auth & Security

## Context Links

- [ARCHITECTURE.md Section 6 Auth](../../ARCHITECTURE.md) — POST /auth/signup, /auth/login, GET /auth/me
- [.env.example](../../.env.example) — SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_SERVICE_KEY, AGENT_AUTH_TOKEN
- [agent/src/server.rs](../../agent/src/server.rs) — Agent HTTP server (no auth, CorsLayer::permissive)
- [agent/src/bin/haviz_app.rs](../../agent/src/bin/haviz_app.rs) — .env.local loading, port 9999 binding
- [apps/web/src/lib/api/client.ts](../../apps/web/src/lib/api/client.ts) — API client (no auth headers)
- [extensions/chrome/](../../extensions/chrome/) — Chrome extension (no auth)

## Overview

Three critical security gaps:
1. **No Auth** — no Supabase integration, no login flow, no token validation anywhere
2. **Agent :9999 unprotected** — anyone on the network can read messages, send commands
3. **Chrome extension calls localhost without token** — no verification of identity

This phase adds Supabase Auth to the cloud API, bearer token auth to the Agent, and login UI to the web app.

## Key Insights

- Supabase Auth handles signup/login/session — we just validate JWTs on our API
- Agent auth is simpler: static bearer token (AGENT_AUTH_TOKEN in .env.local) — Agent is local-first, no cloud users
- Chrome extension needs the Agent auth token to call localhost:9999
- Web UI needs two modes: (1) embedded in Agent webview (use Agent token) and (2) cloud dashboard (use Supabase JWT)
- `CorsLayer::permissive()` in server.rs must be replaced with explicit origin allowlist

## Requirements

### Functional
- Supabase Auth integration in cloud API (signup, login, session)
- JWT validation middleware for all cloud API routes
- Bearer token auth for Agent HTTP server (:9999)
- Login/signup page in Web UI (Svelte)
- Auth-aware API client in Web UI
- Chrome extension stores and sends Agent auth token
- Protected routes: all API endpoints except /health

### Non-Functional
- JWT validation <5ms overhead per request
- Token refresh handled automatically (Supabase client)
- Auth failures return 401 with clear error message
- No credentials in git (already ensured: .env.local gitignored)

## Architecture

### Auth Flow — Cloud API

```
User → Login Page → Supabase Auth → JWT
                                      ↓
                    Cloud API ← Bearer JWT → Supabase verify → 200/401
```

### Auth Flow — Agent (Local)

```
User → .env.local AGENT_AUTH_TOKEN
         ↓
Web UI / Chrome Ext → Authorization: Bearer <token> → Agent :9999
                                                        ↓
                                          Check token matches config → 200/401
```

## Related Code Files

### Create
- `apps/api/src/middleware/auth.ts` — Supabase JWT validation middleware
- `apps/api/src/routes/auth.ts` — POST /auth/signup, /auth/login, GET /auth/me
- `apps/api/src/lib/supabase.ts` — Supabase client initialization
- `apps/web/src/lib/auth/supabase-client.ts` — Supabase browser client
- `apps/web/src/lib/auth/auth-guard.ts` — Auth state management
- `apps/web/src/lib/components/LoginPage.svelte` — Login/signup form
- `apps/web/src/lib/components/AuthLayout.svelte` — Layout wrapper with auth check
- `agent/src/auth.rs` — Bearer token validation for Agent server

### Modify
- `agent/src/server.rs` — add auth middleware, replace CorsLayer::permissive
- `agent/src/config.rs` — add `agent_auth_token` field
- `agent/src/bin/haviz_app.rs` — pass token to server config
- `apps/web/src/lib/api/client.ts` — add Authorization header to all requests
- `apps/web/src/App.svelte` — add routing (login vs dashboard)
- `apps/web/src/main.ts` — add svelte-routing setup
- `extensions/chrome/src/content/zalo-reader.js` — add auth token to Agent calls
- `extensions/chrome/manifest.json` — add storage permission for token

### Delete
- None

## Implementation Steps

### Step 1: Agent auth middleware (2h)
1. Create `agent/src/auth.rs`:
   - Extract `Authorization: Bearer <token>` from request headers
   - Compare with `Config.agent_auth_token`
   - Return 401 JSON if missing/invalid
   - Implement as axum middleware (tower Layer or extractor)
2. Update `agent/src/config.rs`:
   - Add `agent_auth_token: Option<String>` field
   - Load from `AGENT_AUTH_TOKEN` env var
3. Update `agent/src/server.rs`:
   - Apply auth middleware to all `/api/*` routes
   - Keep `/api/status` and static file serving unprotected (needed for webview initial load)
   - Replace `CorsLayer::permissive()` with explicit origins: `localhost:3333`, `localhost:3000`, `localhost:9999`
4. Update `agent/src/bin/haviz_app.rs`:
   - Ensure AGENT_AUTH_TOKEN is loaded from .env.local

### Step 2: Supabase integration in cloud API (3h)
1. Install `@supabase/supabase-js` in apps/api
2. Create `apps/api/src/lib/supabase.ts`:
   - Server-side Supabase client with service key
   - Helper to verify JWT from Authorization header
3. Create `apps/api/src/middleware/auth.ts`:
   - Hono middleware that extracts Bearer token
   - Verifies JWT via Supabase `auth.getUser(token)`
   - Attaches `user` to Hono context (`c.set('user', user)`)
   - Returns 401 if invalid/expired
4. Create `apps/api/src/routes/auth.ts`:
   - `POST /auth/signup` — email + password → Supabase signUp → return session
   - `POST /auth/login` — email + password → Supabase signInWithPassword → return session
   - `GET /auth/me` — return current user from context
5. Apply auth middleware to all route groups except `/health` and `/auth/*`

### Step 3: Web UI login page (3h)
1. Install `@supabase/supabase-js` in apps/web
2. Create `apps/web/src/lib/auth/supabase-client.ts`:
   - Browser Supabase client (SUPABASE_URL + SUPABASE_ANON_KEY from env)
   - Export `supabase` instance
3. Create `apps/web/src/lib/auth/auth-guard.ts`:
   - Svelte store wrapping Supabase auth state
   - `$authStore` — `{ user, session, loading }`
   - `onAuthStateChange` listener
4. Create `apps/web/src/lib/components/LoginPage.svelte`:
   - Email + password form
   - Login / Sign up toggle
   - Error display
   - Redirect to dashboard on success
   - Clean, minimal UI with Tailwind
5. Create `apps/web/src/lib/components/AuthLayout.svelte`:
   - Checks auth state
   - Shows LoginPage if not authenticated
   - Shows children (dashboard) if authenticated
6. Update `apps/web/src/App.svelte`:
   - Wrap dashboard in AuthLayout
   - Use svelte-routing (already installed) for /login vs /dashboard routes

### Step 4: Auth-aware API client (1.5h)
1. Update `apps/web/src/lib/api/client.ts`:
   - Detect mode: embedded (Agent webview) vs cloud (Supabase)
   - Embedded mode: use AGENT_AUTH_TOKEN from URL param or injected global
   - Cloud mode: use Supabase session JWT
   - Add `Authorization: Bearer <token>` to all fetch calls
   - Add token refresh logic for Supabase sessions
   - Handle 401 responses: redirect to login

### Step 5: Chrome extension auth (2h)
1. Update `extensions/chrome/manifest.json`:
   - Add `storage` permission
2. Create `extensions/chrome/src/popup/settings.js`:
   - Simple form to enter/save Agent auth token
   - Store in `chrome.storage.local`
3. Update `extensions/chrome/src/content/zalo-reader.js`:
   - Load token from `chrome.storage.local` on init
   - Add `Authorization: Bearer <token>` to all localhost:9999 fetch calls
   - Show warning if no token configured

### Step 6: Agent CORS hardening (1h)
1. In `agent/src/server.rs`, replace `CorsLayer::permissive()`:
   ```rust
   CorsLayer::new()
       .allow_origin([
           "http://localhost:3333".parse().unwrap(),
           "http://localhost:3000".parse().unwrap(),
           "http://localhost:9999".parse().unwrap(),
       ])
       .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
       .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
   ```
2. Add configurable CORS origins from env var `HAVIZ_CORS_ORIGINS`

### Step 7: Environment cleanup (1h)
1. Verify `.env.local` is in `.gitignore` (confirmed: it is)
2. Verify no `.env.local` file is tracked in git (confirmed: not tracked)
3. Update `.env.example` with clear placeholder values
4. Add `SUPABASE_URL`, `SUPABASE_ANON_KEY` to web app's env (Vite exposes `VITE_` prefixed vars)
5. Document required env vars in README

### Step 8: Integration test (2.5h)
1. Test Agent auth:
   - Request without token → 401
   - Request with wrong token → 401
   - Request with correct token → 200
   - Static file serving works without token
2. Test cloud API auth:
   - Signup → login → access protected route
   - Expired token → 401
   - Invalid token → 401
3. Test Web UI:
   - Login form displays
   - Successful login redirects to dashboard
   - Logout clears session
   - Protected routes redirect to login
4. Test Chrome extension:
   - Token stored and sent correctly
   - Missing token shows warning

## Todo List

- [ ] Create agent/src/auth.rs with bearer token validation
- [ ] Update agent/src/config.rs with auth_token field
- [ ] Add auth middleware to agent/src/server.rs
- [ ] Replace CorsLayer::permissive with explicit origins
- [ ] Install @supabase/supabase-js in apps/api
- [ ] Create Supabase server client + JWT verification middleware
- [ ] Create auth routes (signup, login, me)
- [ ] Apply auth middleware to all cloud API routes
- [ ] Install @supabase/supabase-js in apps/web
- [ ] Create Supabase browser client
- [ ] Create auth store (Svelte)
- [ ] Create LoginPage.svelte
- [ ] Create AuthLayout.svelte
- [ ] Add routing to App.svelte (login vs dashboard)
- [ ] Update API client with auth headers
- [ ] Update Chrome extension with auth token storage
- [ ] Update Chrome extension fetch calls with auth header
- [ ] Harden Agent CORS
- [ ] Update .env.example documentation
- [ ] Integration test all auth flows

## Success Criteria

- Agent :9999 returns 401 without valid AGENT_AUTH_TOKEN
- Cloud API returns 401 without valid Supabase JWT
- Web UI shows login page, authenticates via Supabase, displays dashboard
- Chrome extension sends auth token with all Agent API calls
- No `CorsLayer::permissive()` in production code
- All env secrets in .env.local only (never committed)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Supabase free tier limits | Low | Medium | 50k monthly active users — sufficient for MVP |
| Agent token rotation complexity | Low | Low | Static token for Phase 1; rotate via settings later |
| WebView auth injection | Medium | Medium | Use URL param or window.__haviz_token for embedded mode |
| CORS breaking existing flows | Medium | High | Test thoroughly; keep permissive in dev mode only |

## Security Considerations

- Agent auth token: min 32 chars, generated randomly (document in README)
- Supabase JWT: RS256, auto-expires, refresh handled by client SDK
- Never log tokens in plaintext
- Rate limit auth endpoints (5 attempts/min) to prevent brute force
- HTTPS required for cloud API (enforced at deployment level, not in code)
- Chrome extension token stored in `chrome.storage.local` (encrypted by browser)
