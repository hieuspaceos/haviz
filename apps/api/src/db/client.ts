import postgres from 'postgres';
import { drizzle } from 'drizzle-orm/postgres-js';
import { env } from '../config/env.js';
import * as schema from './schema/index.js';

// Raw postgres client (used for drizzle and direct queries)
export const sql = postgres(env.DATABASE_URL, { max: 10 });

// Drizzle ORM instance with full schema
export const db = drizzle(sql, { schema });

// Graceful shutdown — close pool on process exit
process.on('SIGINT', async () => {
  await sql.end();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await sql.end();
  process.exit(0);
});
