/**
 * Database Connection Module
 * Handles PostgreSQL connection using Drizzle ORM
 */

import { drizzle, type NodePgDatabase } from "drizzle-orm/node-postgres";
import { Pool } from "pg";
import * as schema from "@shared/schema";
import { config } from "./config";

let db: NodePgDatabase<typeof schema> | null = null;
let pool: Pool | null = null;

/**
 * Initialize database connection
 * Uses in-memory storage if DATABASE_URL is not configured
 */
export function initializeDatabase() {
  if (config.databaseUrl) {
    console.log("📦 Connecting to PostgreSQL database...");

    try {
      pool = new Pool({
        connectionString: config.databaseUrl,
        max: 20, // Maximum number of clients in the pool
        idleTimeoutMillis: 30000,
        connectionTimeoutMillis: 2000,
      });

      // Test the connection
      pool.query("SELECT NOW()").then(() => {
        console.log("✅ PostgreSQL connection established");
      }).catch((error) => {
        console.error("❌ PostgreSQL connection failed:", error.message);
        console.log("⚠️  Falling back to in-memory storage");
        pool = null;
        db = null;
      });

      db = drizzle(pool, { schema });
    } catch (error) {
      console.error("❌ Failed to initialize database:", error);
      console.log("⚠️  Falling back to in-memory storage");
      pool = null;
      db = null;
    }
  } else {
    console.log("⚠️  DATABASE_URL not configured, using in-memory storage");
  }
}

/**
 * Get the database instance
 * Returns null if not initialized (will use in-memory storage)
 */
export function getDatabase() {
  return db;
}

/**
 * Close database connection
 */
export async function closeDatabase() {
  if (pool) {
    await pool.end();
    console.log("📦 PostgreSQL connection closed");
  }
}

// Handle process termination
process.on("SIGTERM", async () => {
  await closeDatabase();
  process.exit(0);
});

process.on("SIGINT", async () => {
  await closeDatabase();
  process.exit(0);
});
