import { pgTable, uuid, text, jsonb, timestamp } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';

export const channels = pgTable('channels', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  type: text('type').notNull(), // zalo_personal | zalo_oa | messenger | telegram | phone
  name: text('name').notNull(),
  status: text('status').notNull().default('active'),
  config: jsonb('config'),
  source: text('source').notNull(), // local | cloud
  agentId: text('agent_id'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
});

export type Channel = typeof channels.$inferSelect;
export type NewChannel = typeof channels.$inferInsert;
