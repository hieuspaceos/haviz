import { pgTable, uuid, text, jsonb, timestamp } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';
import { unifiedConversations } from './conversations.js';

export const cloudMessages = pgTable('cloud_messages', {
  id: uuid('id').primaryKey().defaultRandom(),
  conversationId: uuid('conversation_id').notNull().references(() => unifiedConversations.id),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  direction: text('direction').notNull(), // inbound | outbound
  senderType: text('sender_type').notNull(), // user | contact | system
  senderId: text('sender_id').notNull(),
  content: text('content'),
  contentType: text('content_type').notNull().default('text'),
  metadata: jsonb('metadata'),
  status: text('status').notNull().default('received'),
  channelType: text('channel_type').notNull(),
  sentAt: timestamp('sent_at'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
});

export type CloudMessage = typeof cloudMessages.$inferSelect;
export type NewCloudMessage = typeof cloudMessages.$inferInsert;
