---
phase: 1
title: "Bootstrap Backend API"
priority: CRITICAL
status: pending
effort: 20h
---

# Phase 1: Bootstrap Backend API

## Context Links

- [ARCHITECTURE.md Section 4.2](../../ARCHITECTURE.md) — Cloud PostgreSQL schema
- [ARCHITECTURE.md Section 6](../../ARCHITECTURE.md) — API endpoints spec
- [ARCHITECTURE.md Section 7](../../ARCHITECTURE.md) — BullMQ queues
- [docker-compose.yml](../../docker-compose.yml) — Postgres 16 + Redis 7 already configured
- [apps/api/](../../apps/api/) — empty directory with .gitkeep scaffolding
- [.env.example](../../.env.example) — DATABASE_URL + REDIS_URL already defined

## Overview

Backend API is 0% implemented. `apps/api/` contains only `.gitkeep` files in scaffolded directories. No `package.json`, no server, no schema. The web app currently calls Agent localhost:9999 directly — there is no cloud layer for multi-device access, cloud channels (OA, Messenger), or team management.

## Key Insights

- Docker Compose already defines Postgres 16 + Redis 7 — infrastructure ready
- ARCHITECTURE.md Section 4.2 has complete PostgreSQL schema (Tier 1 + Tier 2 tables)
- ARCHITECTURE.md Section 6 has full API endpoint spec (auth, conversations, messages, contacts, templates, AI, agents, webhooks, analytics, safety)
- apps/api/ already has directory scaffolding: `src/{config,db/migrations,db/schema,lib,middleware,routes,services,workers,ws}/`
- Phase 1 scope: bootstrap server + core CRUD only. Auth in Phase 2. Webhooks/workers/analytics deferred to later phases.

## Requirements

### Functional
- Hono HTTP server running on configurable port (default 3001)
- Drizzle ORM connected to PostgreSQL
- Database migration system (drizzle-kit)
- Core CRUD routes: conversations, messages, contacts, templates
- Agent registration endpoint (POST /agents/register)
- Health check endpoint (GET /health)
- CORS configured for web UI origins

### Non-Functional
- TypeScript strict mode
- Response time <100ms for CRUD operations
- Structured JSON error responses `{ ok: boolean, error?: string, data?: T }`
- Request validation (zod)
- Structured logging (pino or built-in)

## Architecture

```
apps/api/
├── src/
│   ├── index.ts                    # Entry point: Hono app + listen
│   ├── config/
│   │   └── env.ts                  # Typed env config (DATABASE_URL, REDIS_URL, PORT)
│   ├── db/
│   │   ├── client.ts               # Drizzle client + connection pool
│   │   ├── schema/
│   │   │   ├── organizations.ts    # organizations table
│   │   │   ├── users.ts            # users table (links to Supabase auth_id)
│   │   │   ├── channels.ts         # channels table
│   │   │   ├── agents.ts           # agents table
│   │   │   ├── templates.ts        # templates table
│   │   │   ├── contacts.ts         # contacts + contact_channels
│   │   │   ├── conversations.ts    # unified_conversations
│   │   │   ├── messages.ts         # cloud_messages
│   │   │   ├── analytics.ts        # daily_metrics, daily_insights, template_analytics
│   │   │   ├── safety.ts           # account_health, safety_audit_log, message_queue, broadcast_log
│   │   │   ├── encrypted.ts        # encrypted_conversations, encrypted_contacts, encrypted_drafts
│   │   │   └── index.ts            # Re-export all schemas
│   │   └── migrations/             # Drizzle generated migrations
│   ├── routes/
│   │   ├── health.ts               # GET /health
│   │   ├── conversations.ts        # GET/PATCH /conversations
│   │   ├── messages.ts             # GET/POST /conversations/:id/messages
│   │   ├── contacts.ts             # GET/PATCH /contacts, merge, channels
│   │   ├── templates.ts            # GET/POST/PATCH /templates
│   │   └── agents.ts               # POST /agents/register, GET /agents
│   ├── middleware/
│   │   └── error-handler.ts        # Global error handler
│   ├── lib/
│   │   └── response.ts             # Typed response helpers (ok, err)
│   └── services/                   # Business logic (keep routes thin)
│       ├── conversation-service.ts
│       ├── message-service.ts
│       ├── contact-service.ts
│       └── template-service.ts
├── package.json
├── tsconfig.json
├── drizzle.config.ts
└── Dockerfile
```

## Related Code Files

### Create
- `apps/api/package.json` — deps: hono, drizzle-orm, drizzle-kit, pg (postgres driver), zod, dotenv
- `apps/api/tsconfig.json` — strict TS config
- `apps/api/drizzle.config.ts` — Drizzle Kit config pointing to DATABASE_URL
- `apps/api/Dockerfile` — Node 20 alpine, multi-stage build
- `apps/api/src/index.ts` — Hono entry
- `apps/api/src/config/env.ts` — zod-validated env
- `apps/api/src/db/client.ts` — Drizzle + postgres pool
- All schema files in `apps/api/src/db/schema/`
- All route files in `apps/api/src/routes/`
- All service files in `apps/api/src/services/`
- `apps/api/src/middleware/error-handler.ts`
- `apps/api/src/lib/response.ts`

### Modify
- `docker-compose.yml` — add `api` service (Node container)
- `pnpm-workspace.yaml` — ensure `apps/api` is included
- `turbo.json` — add `api` build/dev/test tasks

### Delete
- All `.gitkeep` files in `apps/api/src/` subdirectories (replaced by real files)

## Implementation Steps

### Step 1: Initialize package (30min)
1. Create `apps/api/package.json` with dependencies:
   - `hono` (HTTP framework)
   - `@hono/node-server` (Node.js adapter)
   - `drizzle-orm` + `postgres` (DB)
   - `drizzle-kit` (migrations)
   - `zod` (validation)
   - `dotenv` (env loading)
   - DevDeps: `typescript`, `tsx`, `@types/node`
2. Create `tsconfig.json` — target ESNext, moduleResolution bundler, strict true
3. Add scripts: `dev` (tsx watch), `build` (tsc), `db:generate`, `db:migrate`, `db:push`
4. Run `pnpm install` from monorepo root

### Step 2: Environment config (30min)
1. Create `src/config/env.ts` — use zod to validate and type:
   - `DATABASE_URL` (required)
   - `REDIS_URL` (optional for now)
   - `PORT` (default 3001)
   - `CORS_ORIGINS` (default `http://localhost:3333,http://localhost:9999`)
2. Load `.env.local` from project root via dotenv

### Step 3: Database schema (3h)
1. Translate ARCHITECTURE.md Section 4.2 PostgreSQL tables to Drizzle schema:
   - Tier 1 tables: organizations, users, channels, agents, templates, style_profiles, daily_metrics, daily_insights, template_analytics, account_health, safety_audit_log
   - Tier 2 tables: encrypted_conversations, encrypted_contacts, encrypted_drafts
   - Multi-channel tables: contact_channels, unified_conversations, cloud_messages
   - Safety tables: message_queue, broadcast_log
2. Use Drizzle schema conventions: `pgTable()`, proper column types, relations
3. Create `src/db/schema/index.ts` re-exporting everything
4. Run `drizzle-kit generate` to create initial migration
5. Run `drizzle-kit push` to apply schema to local Postgres

### Step 4: Database client (30min)
1. Create `src/db/client.ts`:
   - Use `postgres` (pg driver) + `drizzle()` wrapper
   - Connection pool with sensible defaults (max 10 connections)
   - Graceful shutdown hook

### Step 5: Response helpers + error handler (30min)
1. Create `src/lib/response.ts`:
   - `ok(data)` → `{ ok: true, ...data }`
   - `err(message, status)` → `{ ok: false, error: message }`
2. Create `src/middleware/error-handler.ts`:
   - Catch all errors, return structured JSON
   - Log errors with context

### Step 6: Core routes — Conversations (2h)
1. Create `src/services/conversation-service.ts`:
   - `list(filters)` — paginated, filter by status/channel/assigned
   - `getById(id)` — single conversation with metadata
   - `update(id, data)` — assign, tag, change status
2. Create `src/routes/conversations.ts`:
   - `GET /conversations` — list with query params
   - `GET /conversations/:id` — single
   - `PATCH /conversations/:id` — update
   - Input validation with zod

### Step 7: Core routes — Messages (2h)
1. Create `src/services/message-service.ts`:
   - `list(conversationId, pagination)` — paginated messages
   - `create(conversationId, data)` — store new cloud message
2. Create `src/routes/messages.ts`:
   - `GET /conversations/:id/messages` — paginated
   - `POST /conversations/:id/messages` — create (for cloud channels)

### Step 8: Core routes — Contacts (2h)
1. Create `src/services/contact-service.ts`:
   - `list(filters)` — merged across channels
   - `getById(id)` — detail + linked channels
   - `update(id, data)` — update contact
   - `merge(contactIdA, contactIdB)` — merge duplicate contacts
   - `linkChannel(contactId, channelData)` — link new channel
2. Create `src/routes/contacts.ts`:
   - All CRUD + merge + channel linking endpoints

### Step 9: Core routes — Templates (1.5h)
1. Create `src/services/template-service.ts`:
   - `list(orgId)` — all templates for org
   - `create(data)` — new template with match_patterns
   - `update(id, data)` — edit template
2. Create `src/routes/templates.ts`:
   - `GET /templates`
   - `POST /templates`
   - `PATCH /templates/:id`

### Step 10: Core routes — Agents + Health (1h)
1. Create `src/routes/agents.ts`:
   - `POST /agents/register` — register desktop agent, return auth token
   - `GET /agents` — list agents for org
2. Create `src/routes/health.ts`:
   - `GET /health` — server health + DB connectivity check

### Step 11: Wire up Hono app (1h)
1. Create `src/index.ts`:
   - Initialize Hono app
   - Register CORS middleware
   - Register error handler
   - Mount all route groups
   - Start `@hono/node-server` on configured port
   - Graceful shutdown for DB pool

### Step 12: Docker setup (1h)
1. Create `apps/api/Dockerfile`:
   - Multi-stage: build (node:20-alpine + pnpm) → runtime (node:20-alpine)
   - Copy only necessary files
   - CMD: `node dist/index.js`
2. Update `docker-compose.yml`:
   - Add `api` service depending on `postgres` and `redis`
   - Map port 3001
   - Pass DATABASE_URL and REDIS_URL

### Step 13: Turborepo integration (30min)
1. Ensure `pnpm-workspace.yaml` includes `apps/api`
2. Update `turbo.json`:
   - Add `api#dev`, `api#build`, `api#db:migrate` tasks
   - Set proper dependencies

### Step 14: Smoke test (1h)
1. `docker-compose up -d` (Postgres + Redis)
2. `pnpm --filter api db:push` (apply schema)
3. `pnpm --filter api dev` (start server)
4. Test all endpoints with curl/httpie
5. Verify DB tables created correctly

## Todo List

- [ ] Initialize package.json with deps
- [ ] Create tsconfig.json
- [ ] Create env config with zod validation
- [ ] Create Drizzle schema for all ARCHITECTURE.md tables
- [ ] Create drizzle.config.ts
- [ ] Create DB client with connection pool
- [ ] Create response helpers + error handler middleware
- [ ] Implement conversations routes + service
- [ ] Implement messages routes + service
- [ ] Implement contacts routes + service
- [ ] Implement templates routes + service
- [ ] Implement agents registration route
- [ ] Implement health check route
- [ ] Wire up Hono app entry point
- [ ] Create Dockerfile
- [ ] Update docker-compose.yml with api service
- [ ] Update turbo.json
- [ ] Smoke test all endpoints

## Success Criteria

- `pnpm --filter api dev` starts server on port 3001
- All ARCHITECTURE.md Section 4.2 tables exist in PostgreSQL
- All Phase 1 CRUD endpoints return correct JSON responses
- `drizzle-kit generate` produces valid migration files
- Docker build succeeds and container runs
- Response time <100ms for basic CRUD

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Schema mismatch with ARCHITECTURE.md | Low | High | Cross-reference each table carefully |
| Drizzle ORM learning curve | Medium | Low | Well-documented, similar to Prisma |
| Postgres connection issues in Docker | Low | Medium | Use docker-compose healthcheck |
| Schema too large for Phase 1 | Medium | Low | Create all tables but only implement CRUD for core 4 |

## Security Considerations

- No auth in this phase (added in Phase 2) — endpoints are unprotected
- Do NOT expose API publicly until Phase 2 auth is complete
- Database credentials only in .env.local (already gitignored)
- Use parameterized queries (Drizzle does this by default)
- Validate all input with zod schemas
