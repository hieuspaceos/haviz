import { pgTable, uuid, text, timestamp, customType } from 'drizzle-orm/pg-core';
import { organizations } from './organizations.js';
import { users } from './users.js';

// Custom type for bytea (PostgreSQL binary data for E2E encrypted blobs)
const bytea = customType<{ data: Buffer }>({
  dataType() {
    return 'bytea';
  },
});

export const encryptedConversations = pgTable('encrypted_conversations', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  encryptedBlob: bytea('encrypted_blob').notNull(), // AES-256-GCM: { contact_name, last_preview, unread_count, status }
  nonce: bytea('nonce').notNull(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});

export const encryptedContacts = pgTable('encrypted_contacts', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  encryptedBlob: bytea('encrypted_blob').notNull(), // AES-256-GCM: { display_name, phone_masked, tags }
  nonce: bytea('nonce').notNull(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});

export const encryptedDrafts = pgTable('encrypted_drafts', {
  id: uuid('id').primaryKey().defaultRandom(),
  orgId: uuid('org_id').notNull().references(() => organizations.id),
  userId: uuid('user_id').notNull().references(() => users.id),
  conversationRef: text('conversation_ref').notNull(),
  status: text('status').notNull().default('pending'), // pending | approved | rejected
  encryptedContent: bytea('encrypted_content').notNull(),
  nonce: bytea('nonce').notNull(),
  createdAt: timestamp('created_at').notNull().defaultNow(),
});

export type EncryptedConversation = typeof encryptedConversations.$inferSelect;
export type EncryptedContact = typeof encryptedContacts.$inferSelect;
export type EncryptedDraft = typeof encryptedDrafts.$inferSelect;
