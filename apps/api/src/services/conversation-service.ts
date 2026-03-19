import { eq, and, type SQL } from 'drizzle-orm';
import { db } from '../db/client.js';
import { unifiedConversations } from '../db/schema/index.js';

export interface ConversationFilters {
  status?: string;
  channelType?: string;
}

export interface ConversationUpdate {
  status?: string;
  assignedTo?: string;
  tags?: string[];
  priority?: string;
}

export async function listConversations(filters: ConversationFilters = {}) {
  const conditions: SQL[] = [];

  if (filters.status) {
    conditions.push(eq(unifiedConversations.status, filters.status));
  }
  if (filters.channelType) {
    conditions.push(eq(unifiedConversations.channelType, filters.channelType));
  }

  return db
    .select()
    .from(unifiedConversations)
    .where(conditions.length > 0 ? and(...conditions) : undefined)
    .orderBy(unifiedConversations.lastActivityAt);
}

export async function getConversationById(id: string) {
  const rows = await db
    .select()
    .from(unifiedConversations)
    .where(eq(unifiedConversations.id, id))
    .limit(1);
  return rows[0] ?? null;
}

export async function updateConversation(id: string, data: ConversationUpdate) {
  const rows = await db
    .update(unifiedConversations)
    .set({ ...data, updatedAt: new Date() })
    .where(eq(unifiedConversations.id, id))
    .returning();
  return rows[0] ?? null;
}
