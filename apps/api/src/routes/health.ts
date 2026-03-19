import { Hono } from 'hono';
import { ok } from '../lib/response.js';

const health = new Hono();

health.get('/', (c) => {
  return c.json(ok({ status: 'ok', timestamp: new Date().toISOString() }));
});

export default health;
