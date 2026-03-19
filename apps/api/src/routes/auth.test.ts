/**
 * Tests for /api/auth routes — validation & middleware logic only.
 * Supabase calls are NOT tested here (require live credentials).
 * We mount a minimal Hono app that mirrors index.ts route structure.
 */
import { describe, it, expect, vi } from 'vitest';
import { Hono } from 'hono';

// ── Stub supabase before auth route imports it ──────────────────────────────
vi.mock('../lib/supabase.js', () => ({
  supabase: {
    auth: {
      signUp: vi.fn().mockResolvedValue({ data: {}, error: { message: 'stub' } }),
      signInWithPassword: vi.fn().mockResolvedValue({ data: {}, error: { message: 'stub' } }),
      getUser: vi.fn().mockResolvedValue({ data: { user: null }, error: { message: 'Unauthorized' } }),
    },
  },
}));

const { default: authRoutes } = await import('./auth.js');

const app = new Hono();
app.route('/api/auth', authRoutes);

describe('POST /api/auth/signup', () => {
  it('returns 400 when email is missing', async () => {
    const res = await app.request('/api/auth/signup', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password: 'secret123' }),
    });
    expect(res.status).toBe(400);
    const body = await res.json() as { ok: boolean };
    expect(body.ok).toBe(false);
  });

  it('returns 400 when password is missing', async () => {
    const res = await app.request('/api/auth/signup', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: 'test@example.com' }),
    });
    expect(res.status).toBe(400);
    const body = await res.json() as { ok: boolean };
    expect(body.ok).toBe(false);
  });
});

describe('POST /api/auth/login', () => {
  it('returns 400 when email is missing', async () => {
    const res = await app.request('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password: 'secret123' }),
    });
    expect(res.status).toBe(400);
    const body = await res.json() as { ok: boolean };
    expect(body.ok).toBe(false);
  });

  it('returns 400 when password is missing', async () => {
    const res = await app.request('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: 'test@example.com' }),
    });
    expect(res.status).toBe(400);
    const body = await res.json() as { ok: boolean };
    expect(body.ok).toBe(false);
  });
});

describe('GET /api/auth/me', () => {
  it('returns 401 when Authorization header is absent', async () => {
    const res = await app.request('/api/auth/me');
    // authMiddleware on /me checks for Bearer token; stub getUser returns error
    expect(res.status).toBe(401);
  });
});
