import type { Config } from "drizzle-kit";
import dotenv from "dotenv";

// Load environment variables
dotenv.config({ path: "../.env" });

export default {
  schema: "./shared/schema.ts",
  out: "./drizzle",
  dialect: "postgresql",
  dbCredentials: {
    url: process.env.DATABASE_URL || "postgresql://localhost:5432/nexus_security",
  },
} satisfies Config;