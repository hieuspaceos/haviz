/// Auth routes — signup, login, current user.
/// POST /auth/signup  { email, password, full_name }
/// POST /auth/login   { email, password }
/// GET  /auth/me      (requires authMiddleware)
import { Hono } from 'hono';
import { supabase } from '../lib/supabase.js';
import { authMiddleware } from '../middleware/auth.js';
import type { AppVariables } from '../lib/app-types.js';

const auth = new Hono<{ Variables: AppVariables }>();

auth.post('/signup', async (c) => {
  const { email, password, full_name } = await c.req.json<{
    email: string;
    password: string;
    full_name?: string;
  }>();

  if (!email || !password) {
    return c.json({ ok: false, error: 'email and password are required' }, 400);
  }

  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: { data: { full_name } },
  });

  if (error) {
    return c.json({ ok: false, error: error.message }, 400);
  }

  return c.json({ ok: true, session: data.session, user: data.user }, 201);
});

auth.post('/login', async (c) => {
  const { email, password } = await c.req.json<{
    email: string;
    password: string;
  }>();

  if (!email || !password) {
    return c.json({ ok: false, error: 'email and password are required' }, 400);
  }

  const { data, error } = await supabase.auth.signInWithPassword({ email, password });

  if (error) {
    return c.json({ ok: false, error: error.message }, 401);
  }

  return c.json({ ok: true, session: data.session, user: data.user });
});

auth.get('/me', authMiddleware, (c) => {
  const user = c.get('user');
  return c.json({ ok: true, user });
});

export default auth;
