---
title: "Fix Haviz Project Weaknesses"
description: "Comprehensive plan to address 11 identified weaknesses from security-critical to low priority"
status: pending
priority: P1
effort: 80h
branch: main
tags: [backend, auth, security, agent, testing, ci, safety]
created: 2026-03-19
---

# Fix Haviz Project Weaknesses

## Summary

Address 11 weaknesses across 6 phases. Ordered by dependency: backend API must exist before auth; auth before security hardening; agent modularization enables Windows support; testing/CI validates everything.

## Phase Overview

| # | Phase | Priority | Effort | Status | Deps |
|---|-------|----------|--------|--------|------|
| 1 | [Bootstrap Backend API](phase-01-bootstrap-backend-api.md) | CRITICAL | 20h | pending | none |
| 2 | [Auth & Security](phase-02-auth-and-security.md) | CRITICAL | 16h | pending | P1 |
| 3 | [Modularize Agent](phase-03-modularize-agent.md) | HIGH | 10h | pending | none |
| 4 | [Windows Support](phase-04-windows-support.md) | HIGH | 16h | pending | P3 |
| 5 | [Testing & CI/CD](phase-05-testing-and-ci.md) | MEDIUM | 12h | pending | P1,P3 |
| 6 | [Safety Engine](phase-06-safety-engine.md) | MEDIUM | 8h | pending | P3 |

## Dependency Graph

```
P1 (Backend API) ──> P2 (Auth & Security) ──> P5 (Testing & CI)
P3 (Modularize Agent) ──> P4 (Windows Support)
P3 (Modularize Agent) ──> P5 (Testing & CI)
P3 (Modularize Agent) ──> P6 (Safety Engine)
```

## Parallelization

- P1 + P3 can run in parallel (independent codebases: TS vs Rust)
- P4 + P6 can run in parallel after P3 completes
- P2 depends on P1 completing (needs API routes to protect)
- P5 is last (tests all prior work)

## Key Decisions

1. Backend: Hono + Drizzle + PostgreSQL (per ARCHITECTURE.md)
2. Auth: Supabase Auth (per ARCHITECTURE.md)
3. Agent modularization: extract from haviz_app.rs monolith, keep each file <200 LOC
4. Windows: Win32 UI Automation API (mirrors macOS AX API approach)
5. Testing: Rust `#[cfg(test)]` inline + vitest for TS + GitHub Actions
6. Safety: implement in agent/src/safety/ module directory

## Timeline (sequential)

- Week 1: P1 + P3 (parallel)
- Week 2: P2 + P4 (parallel, P2 after P1, P4 after P3)
- Week 3: P5 + P6 (parallel)
