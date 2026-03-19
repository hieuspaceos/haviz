import { eq } from 'drizzle-orm';
import { db } from '../db/client.js';
import { cloudMessages, type NewCloudMessage } from '../db/schema/index.js';

export interface MessageListOptions {
  limit?: number;
  offset?: number;
}

export async function listMessages(conversationId: string, opts: MessageListOptions = {}) {
  const limit = opts.limit ?? 50;
  const offset = opts.offset ?? 0;

  return db
    .select()
    .from(cloudMessages)
    .where(eq(cloudMessages.conversationId, conversationId))
    .orderBy(cloudMessages.createdAt)
    .limit(limit)
    .offset(offset);
}

export async function createMessage(data: NewCloudMessage) {
  const rows = await db
    .insert(cloudMessages)
    .values(data)
    .returning();
  return rows[0];
}
