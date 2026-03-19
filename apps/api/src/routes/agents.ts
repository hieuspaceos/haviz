import { Hono } from 'hono';
import { ok, err } from '../lib/response.js';
import { db } from '../db/client.js';
import { agents } from '../db/schema/index.js';
import { randomUUID } from 'crypto';

const agentsRouter = new Hono();

// POST /agents/register — register a new desktop agent, returns auth_token
agentsRouter.post('/register', async (c) => {
  const body = await c.req.json<{
    orgId: string;
    userId: string;
    name: string;
    platform: string;
    version: string;
  }>();

  if (!body.orgId || !body.userId || !body.name || !body.platform || !body.version) {
    return err('orgId, userId, name, platform, version are required', 400);
  }

  const authToken = randomUUID(); // simple token; replace with signed JWT in production

  const rows = await db
    .insert(agents)
    .values({ ...body, authToken, status: 'offline' })
    .returning();

  return c.json(ok(rows[0]), 201);
});

// GET /agents — list all agents
agentsRouter.get('/', async (c) => {
  const rows = await db.select().from(agents).orderBy(agents.createdAt);
  return c.json(ok(rows));
});

export default agentsRouter;
