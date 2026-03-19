/// Hono auth middleware — validates Supabase Bearer token.
/// Sets `user` in context on success; returns 401 JSON on failure.
import type { Context, Next } from 'hono';
import { supabase } from '../lib/supabase.js';
import type { AppVariables } from '../lib/app-types.js';

export async function authMiddleware(
  c: Context<{ Variables: AppVariables }>,
  next: Next,
) {
  const authorization = c.req.header('Authorization');

  if (!authorization || !authorization.startsWith('Bearer ')) {
    return c.json({ ok: false, error: 'Missing or invalid Authorization header' }, 401);
  }

  const token = authorization.slice(7); // strip "Bearer "

  const { data, error } = await supabase.auth.getUser(token);

  if (error || !data.user) {
    return c.json({ ok: false, error: 'Unauthorized' }, 401);
  }

  c.set('user', data.user);
  await next();
}
