import { pgTable, uuid, text, integer, timestamp, customType } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';
import { contacts } from './contacts.js';

// Custom type for bytea (PostgreSQL binary data)
const bytea = customType<{ data: Buffer }>({
  dataType() {
    return 'bytea';
  },
});

export const unifiedConversations = pgTable('unified_conversations', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  contactId: uuid('contact_id').references(() => contacts.id),
  channelType: text('channel_type').notNull(),
  channelSource: text('channel_source').notNull(), // local | cloud
  agentId: text('agent_id'),
  status: text('status').notNull().default('open'),
  assignedTo: text('assigned_to'),
  lastActivityAt: timestamp('last_activity_at'),
  lastPreviewEncrypted: bytea('last_preview_encrypted'), // E2E if local
  lastPreviewPlain: text('last_preview_plain'),          // plain if cloud
  unreadCount: integer('unread_count').notNull().default(0),
  tags: text('tags').array(),
  priority: text('priority').notNull().default('normal'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});

export type UnifiedConversation = typeof unifiedConversations.$inferSelect;
export type NewUnifiedConversation = typeof unifiedConversations.$inferInsert;
