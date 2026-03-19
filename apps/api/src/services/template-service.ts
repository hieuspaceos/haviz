import { eq } from 'drizzle-orm';
import { db } from '../db/client.js';
import { templates, type NewTemplate } from '../db/schema/index.js';

export async function listTemplates(orgId: string) {
  return db
    .select()
    .from(templates)
    .where(eq(templates.orgId, orgId))
    .orderBy(templates.createdAt);
}

export async function createTemplate(data: NewTemplate) {
  const rows = await db.insert(templates).values(data).returning();
  return rows[0];
}

export async function updateTemplate(
  id: string,
  data: Partial<typeof templates.$inferInsert>
) {
  const rows = await db
    .update(templates)
    .set({ ...data, updatedAt: new Date() })
    .where(eq(templates.id, id))
    .returning();
  return rows[0] ?? null;
}
