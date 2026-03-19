/**
 * Tests for /api/templates routes — auth guard & route structure only.
 * DB/Supabase calls are stubbed to isolate middleware behavior.
 */
import { describe, it, expect, vi } from 'vitest';
import { Hono } from 'hono';
import type { AppVariables } from '../lib/app-types.js';

// ── Stubs ────────────────────────────────────────────────────────────────────
vi.mock('../lib/supabase.js', () => ({
  supabase: {
    auth: {
      getUser: vi.fn().mockResolvedValue({ data: { user: null }, error: { message: 'Unauthorized' } }),
    },
  },
}));

vi.mock('../services/template-service.js', () => ({
  listTemplates: vi.fn().mockResolvedValue([]),
  createTemplate: vi.fn().mockResolvedValue({}),
  updateTemplate: vi.fn().mockResolvedValue(null),
}));

const { authMiddleware } = await import('../middleware/auth.js');
const { default: templates } = await import('./templates.js');

const app = new Hono<{ Variables: AppVariables }>();
app.use('/api/templates/*', authMiddleware);
app.route('/api/templates', templates);

describe('GET /api/templates', () => {
  it('returns 401 without Authorization header', async () => {
    const res = await app.request('/api/templates');
    expect(res.status).toBe(401);
    const body = await res.json() as { ok: boolean; error: string };
    expect(body.ok).toBe(false);
  });

  it('returns 401 with malformed Bearer token', async () => {
    const res = await app.request('/api/templates', {
      headers: { Authorization: 'Bearer fake-token' },
    });
    expect(res.status).toBe(401);
  });
});
