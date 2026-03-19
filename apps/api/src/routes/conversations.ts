import { Hono } from 'hono';
import { ok, err } from '../lib/response.js';
import {
  listConversations,
  getConversationById,
  updateConversation,
} from '../services/conversation-service.js';

const conversations = new Hono();

// GET /conversations — list with optional filters
conversations.get('/', async (c) => {
  const status = c.req.query('status');
  const channelType = c.req.query('channel_type');
  const rows = await listConversations({ status, channelType });
  return c.json(ok(rows));
});

// GET /conversations/:id — single conversation
conversations.get('/:id', async (c) => {
  const row = await getConversationById(c.req.param('id'));
  if (!row) return err('Conversation not found', 404);
  return c.json(ok(row));
});

// PATCH /conversations/:id — assign, tag, status
conversations.patch('/:id', async (c) => {
  const body = await c.req.json<{
    status?: string;
    assignedTo?: string;
    tags?: string[];
    priority?: string;
  }>();
  const row = await updateConversation(c.req.param('id'), body);
  if (!row) return err('Conversation not found', 404);
  return c.json(ok(row));
});

export default conversations;
