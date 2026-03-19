import { pgTable, uuid, text, jsonb, integer, boolean, timestamp } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';

export const templates = pgTable('templates', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  name: text('name').notNull(),
  content: text('content').notNull(),
  category: text('category'),
  variables: jsonb('variables'),
  usageCount: integer('usage_count').notNull().default(0),
  matchPatterns: jsonb('match_patterns'),
  autoMatch: boolean('auto_match').notNull().default(false),
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});

export type Template = typeof templates.$inferSelect;
export type NewTemplate = typeof templates.$inferInsert;
