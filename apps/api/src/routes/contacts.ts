import { Hono } from 'hono';
import { ok, err } from '../lib/response.js';
import {
  listContacts,
  getContactById,
  updateContact,
  mergeContacts,
  listContactChannels,
  linkChannel,
} from '../services/contact-service.js';

const contacts = new Hono();

// GET /contacts — list all
contacts.get('/', async (c) => {
  const rows = await listContacts();
  return c.json(ok(rows));
});

// GET /contacts/:id — single with linked channels
contacts.get('/:id', async (c) => {
  const row = await getContactById(c.req.param('id'));
  if (!row) return err('Contact not found', 404);
  return c.json(ok(row));
});

// PATCH /contacts/:id — update
contacts.patch('/:id', async (c) => {
  const body = await c.req.json<{
    displayName?: string;
    phone?: string;
    email?: string;
    tags?: string[];
  }>();
  const row = await updateContact(c.req.param('id'), body);
  if (!row) return err('Contact not found', 404);
  return c.json(ok(row));
});

// POST /contacts/merge — merge two contacts
contacts.post('/merge', async (c) => {
  const body = await c.req.json<{ primaryId: string; secondaryId: string }>();
  if (!body.primaryId || !body.secondaryId) {
    return err('primaryId and secondaryId are required', 400);
  }
  const result = await mergeContacts(body.primaryId, body.secondaryId);
  return c.json(ok(result));
});

// GET /contacts/:id/channels — list linked channels
contacts.get('/:id/channels', async (c) => {
  const rows = await listContactChannels(c.req.param('id'));
  return c.json(ok(rows));
});

// POST /contacts/:id/channels — link new channel
contacts.post('/:id/channels', async (c) => {
  const contactId = c.req.param('id');
  const body = await c.req.json<{
    channelType: string;
    channelSource: string;
    externalId: string;
    externalName?: string;
    agentId?: string;
    isPrimary?: boolean;
  }>();
  const row = await linkChannel({ contactId, ...body });
  return c.json(ok(row), 201);
});

export default contacts;
