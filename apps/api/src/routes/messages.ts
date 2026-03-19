import { Hono } from 'hono';
import { ok, err } from '../lib/response.js';
import { listMessages, createMessage } from '../services/message-service.js';
import { getConversationById } from '../services/conversation-service.js';

const messages = new Hono<{ Variables: Record<string, unknown> }>();

// GET /conversations/:id/messages — paginated
messages.get('/:id/messages', async (c) => {
  const convId = c.req.param('id');
  const limit = Number(c.req.query('limit') ?? 50);
  const offset = Number(c.req.query('offset') ?? 0);
  const rows = await listMessages(convId, { limit, offset });
  return c.json(ok(rows));
});

// POST /conversations/:id/messages — create cloud message
messages.post('/:id/messages', async (c) => {
  const convId = c.req.param('id');
  const conv = await getConversationById(convId);
  if (!conv) return err('Conversation not found', 404);

  const body = await c.req.json<{
    content: string;
    contentType?: string;
    senderType: string;
    senderId: string;
    direction: string;
  }>();

  const row = await createMessage({
    conversationId: convId,
    orgId: conv.orgId,
    direction: body.direction,
    senderType: body.senderType,
    senderId: body.senderId,
    content: body.content,
    contentType: body.contentType ?? 'text',
    channelType: conv.channelType,
    sentAt: new Date(),
  });

  return c.json(ok(row), 201);
});

export default messages;
