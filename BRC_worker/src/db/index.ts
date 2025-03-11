import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import * as schema from "./schema.js";
import { DATABASE_POOLER_URL } from "../config/app-config.js";

export async function getDB() {
  const client = postgres(DATABASE_POOLER_URL!, { prepare: false });
  return drizzle({ client, schema });
}
