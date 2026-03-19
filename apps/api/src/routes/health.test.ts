/**
 * Tests for GET /api/health
 * Uses Hono's built-in request() test helper — no server needed.
 */
import { describe, it, expect } from 'vitest';
import { Hono } from 'hono';
import health from './health.js';

const app = new Hono();
app.route('/api/health', health);

describe('GET /api/health', () => {
  it('returns 200 with ok: true', async () => {
    const res = await app.request('/api/health');
    expect(res.status).toBe(200);
    const body = await res.json() as { ok: boolean; data: { status: string } };
    expect(body.ok).toBe(true);
    expect(body.data.status).toBe('ok');
  });

  it('includes a timestamp in response', async () => {
    const res = await app.request('/api/health');
    const body = await res.json() as { data: { timestamp: string } };
    expect(body.data.timestamp).toBeDefined();
    expect(new Date(body.data.timestamp).getTime()).toBeGreaterThan(0);
  });
});
