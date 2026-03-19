import { pgTable, uuid, text, jsonb, boolean, timestamp } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';

export const contacts = pgTable('contacts', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  displayName: text('display_name').notNull(),
  phone: text('phone'),
  email: text('email'),
  tags: text('tags').array(),
  metadata: jsonb('metadata'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});

export const contactChannels = pgTable('contact_channels', {
  id: uuid('id').primaryKey().defaultRandom(),
  contactId: uuid('contact_id').notNull().references(() => contacts.id),
  channelType: text('channel_type').notNull(),
  channelSource: text('channel_source').notNull(), // local | cloud
  externalId: text('external_id').notNull(),
  externalName: text('external_name'),
  agentId: text('agent_id'),
  isPrimary: boolean('is_primary').notNull().default(false),
  linkedAt: timestamp('linked_at').notNull().defaultNow(),
});

export type Contact = typeof contacts.$inferSelect;
export type NewContact = typeof contacts.$inferInsert;
export type ContactChannel = typeof contactChannels.$inferSelect;
export type NewContactChannel = typeof contactChannels.$inferInsert;
