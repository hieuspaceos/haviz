import { eq } from 'drizzle-orm';
import { db } from '../db/client.js';
import {
  contacts,
  contactChannels,
  type NewContactChannel,
} from '../db/schema/index.js';

export async function listContacts() {
  return db.select().from(contacts).orderBy(contacts.createdAt);
}

export async function getContactById(id: string) {
  const rows = await db
    .select()
    .from(contacts)
    .where(eq(contacts.id, id))
    .limit(1);
  return rows[0] ?? null;
}

export async function updateContact(id: string, data: Partial<typeof contacts.$inferInsert>) {
  const rows = await db
    .update(contacts)
    .set({ ...data, updatedAt: new Date() })
    .where(eq(contacts.id, id))
    .returning();
  return rows[0] ?? null;
}

export async function mergeContacts(primaryId: string, secondaryId: string) {
  // Reassign all channel links from secondary to primary
  await db
    .update(contactChannels)
    .set({ contactId: primaryId })
    .where(eq(contactChannels.contactId, secondaryId));

  // Delete the secondary contact
  await db.delete(contacts).where(eq(contacts.id, secondaryId));

  return getContactById(primaryId);
}

export async function listContactChannels(contactId: string) {
  return db
    .select()
    .from(contactChannels)
    .where(eq(contactChannels.contactId, contactId));
}

export async function linkChannel(data: NewContactChannel) {
  const rows = await db
    .insert(contactChannels)
    .values(data)
    .returning();
  return rows[0];
}
