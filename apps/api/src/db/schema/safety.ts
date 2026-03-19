import { pgTable, uuid, text, integer, jsonb, timestamp } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';
import { users } from './users.js';

export const accountHealth = pgTable('account_health', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  score: integer('score').notNull().default(100),
  status: text('status').notNull().default('healthy'),
  messagesSentToday: integer('messages_sent_today').notNull().default(0),
  messagesFailedToday: integer('messages_failed_today').notNull().default(0),
});

export const safetyAuditLog = pgTable('safety_audit_log', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').references(() => users.id),
  eventType: text('event_type').notNull(),
  detailsJson: jsonb('details_json'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
});

export type AccountHealth = typeof accountHealth.$inferSelect;
export type SafetyAuditLog = typeof safetyAuditLog.$inferSelect;
