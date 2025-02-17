import { drizzle } from 'drizzle-orm/postgres-js'
import postgres from 'postgres'
import * as schema from "./schema"

export async function getDB() {
    const client = postgres(process.env.DATABASE_POOLER_URL!, { prepare: false })
    return drizzle({ client, schema });
}
