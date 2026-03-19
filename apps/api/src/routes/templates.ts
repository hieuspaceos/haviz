import { Hono } from 'hono';
import { ok, err } from '../lib/response.js';
import {
  listTemplates,
  createTemplate,
  updateTemplate,
} from '../services/template-service.js';

const templates = new Hono();

// GET /templates — list all for org (org_id from query for now; auth middleware later)
templates.get('/', async (c) => {
  const orgId = c.req.query('org_id') ?? '';
  if (!orgId) return err('org_id query param is required', 400);
  const rows = await listTemplates(orgId);
  return c.json(ok(rows));
});

// POST /templates — create
templates.post('/', async (c) => {
  const body = await c.req.json<{
    orgId: string;
    name: string;
    content: string;
    category?: string;
    variables?: unknown;
    matchPatterns?: unknown;
    autoMatch?: boolean;
  }>();
  const row = await createTemplate(body);
  return c.json(ok(row), 201);
});

// PATCH /templates/:id — update
templates.patch('/:id', async (c) => {
  const body = await c.req.json<{
    name?: string;
    content?: string;
    category?: string;
    variables?: unknown;
    matchPatterns?: unknown;
    autoMatch?: boolean;
  }>();
  const row = await updateTemplate(c.req.param('id'), body);
  if (!row) return err('Template not found', 404);
  return c.json(ok(row));
});

export default templates;
