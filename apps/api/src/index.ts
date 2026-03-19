import 'dotenv/config';
import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { serve } from '@hono/node-server';
import { env } from './config/env.js';
import { errorHandler } from './middleware/error-handler.js';
import { authMiddleware } from './middleware/auth.js';
import health from './routes/health.js';
import authRoutes from './routes/auth.js';
import conversations from './routes/conversations.js';
import messages from './routes/messages.js';
import contacts from './routes/contacts.js';
import templates from './routes/templates.js';
import agents from './routes/agents.js';

const app = new Hono();

// CORS — allow configured origins
app.use(
  '*',
  cors({
    origin: env.CORS_ORIGINS.split(',').map((o) => o.trim()),
    allowMethods: ['GET', 'POST', 'PATCH', 'DELETE', 'OPTIONS'],
    allowHeaders: ['Content-Type', 'Authorization'],
  })
);

// Global error handler
app.use('*', errorHandler);

// Public routes — no auth required
app.route('/api/health', health);
app.route('/api/auth', authRoutes);

// Protected routes — require valid Supabase session
app.use('/api/conversations/*', authMiddleware);
app.use('/api/contacts/*', authMiddleware);
app.use('/api/templates/*', authMiddleware);
app.use('/api/agents/*', authMiddleware);

app.route('/api/conversations', conversations);
app.route('/api/conversations', messages); // nested: /api/conversations/:id/messages
app.route('/api/contacts', contacts);
app.route('/api/templates', templates);
app.route('/api/agents', agents);

// Start server
serve({ fetch: app.fetch, port: env.PORT }, (info) => {
  console.log(`[api] Haviz API running on http://localhost:${info.port}`);
});
