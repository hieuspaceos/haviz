import { pgTable, uuid, text, integer, jsonb, date, real } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';
import { users } from './users.js';
import { templates } from './templates.js';

export const dailyMetrics = pgTable('daily_metrics', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  date: date('date').notNull(),
  messagesInbound: integer('messages_inbound').notNull().default(0),
  messagesOutbound: integer('messages_outbound').notNull().default(0),
  avgResponseTimeSeconds: integer('avg_response_time_seconds'),
  firstResponseTimeSeconds: integer('first_response_time_seconds'),
  aiDraftsTotal: integer('ai_drafts_total').notNull().default(0),
  aiDraftsApproved: integer('ai_drafts_approved').notNull().default(0),
  aiDraftsEdited: integer('ai_drafts_edited').notNull().default(0),
  aiDraftsRejected: integer('ai_drafts_rejected').notNull().default(0),
  templatesUsed: integer('templates_used').notNull().default(0),
  conversationsActive: integer('conversations_active').notNull().default(0),
  conversationsNew: integer('conversations_new').notNull().default(0),
  busiestHour: integer('busiest_hour'),
});

export const dailyInsights = pgTable('daily_insights', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  date: date('date').notNull(),
  topTopics: jsonb('top_topics'),
  sentiment: jsonb('sentiment'),
  commonQuestions: jsonb('common_questions'),
  suggestedActions: jsonb('suggested_actions'),
  qualityScore: integer('quality_score'),
});

export const templateAnalytics = pgTable('template_analytics', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  templateId: uuid('template_id').notNull().references(() => templates.id),
  date: date('date').notNull(),
  timesUsed: integer('times_used').notNull().default(0),
  timesEdited: integer('times_edited').notNull().default(0),
  avgResponseRate: real('avg_response_rate'),
});

export type DailyMetrics = typeof dailyMetrics.$inferSelect;
export type DailyInsights = typeof dailyInsights.$inferSelect;
export type TemplateAnalytics = typeof templateAnalytics.$inferSelect;
