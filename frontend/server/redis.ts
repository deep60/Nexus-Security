/**
 * Redis Session Storage Module
 * Handles Redis connection for persistent sessions
 */

import Redis from "ioredis";
import { config } from "./config";

let redis: Redis | null = null;

/**
 * Initialize Redis connection
 * Falls back to in-memory sessions if REDIS_URL is not configured
 */
export function initializeRedis() {
  if (config.redisUrl) {
    console.log("🔴 Connecting to Redis...");

    try {
      redis = new Redis(config.redisUrl, {
        maxRetriesPerRequest: 3,
        retryStrategy(times) {
          const delay = Math.min(times * 50, 2000);
          return delay;
        },
        reconnectOnError(err) {
          console.error("Redis reconnect on error:", err.message);
          return true;
        },
      });

      redis.on("connect", () => {
        console.log("✅ Redis connection established");
      });

      redis.on("error", (error) => {
        console.error("❌ Redis connection error:", error.message);
        console.log("⚠️  Falling back to in-memory sessions");
        redis = null;
      });

      redis.on("close", () => {
        console.log("🔴 Redis connection closed");
      });
    } catch (error) {
      console.error("❌ Failed to initialize Redis:", error);
      console.log("⚠️  Falling back to in-memory sessions");
      redis = null;
    }
  } else {
    console.log("⚠️  REDIS_URL not configured, using in-memory sessions");
  }
}

/**
 * Get the Redis client instance
 * Returns null if not initialized (will use in-memory sessions)
 */
export function getRedis() {
  return redis;
}

/**
 * Session storage interface
 */
export interface SessionStore {
  get(sessionId: string): Promise<{ userId: string; expiresAt: number } | null>;
  set(sessionId: string, data: { userId: string; expiresAt: number }): Promise<void>;
  delete(sessionId: string): Promise<void>;
}

/**
 * Redis-based session store
 */
class RedisSessionStore implements SessionStore {
  private redis: Redis;

  constructor(redisClient: Redis) {
    this.redis = redisClient;
  }

  async get(sessionId: string) {
    const data = await this.redis.get(`session:${sessionId}`);
    if (!data) return null;

    try {
      return JSON.parse(data);
    } catch {
      return null;
    }
  }

  async set(sessionId: string, data: { userId: string; expiresAt: number }) {
    const ttl = Math.floor((data.expiresAt - Date.now()) / 1000);
    if (ttl > 0) {
      await this.redis.setex(
        `session:${sessionId}`,
        ttl,
        JSON.stringify(data)
      );
    }
  }

  async delete(sessionId: string) {
    await this.redis.del(`session:${sessionId}`);
  }
}

/**
 * In-memory session store (fallback)
 */
class MemorySessionStore implements SessionStore {
  private sessions = new Map<string, { userId: string; expiresAt: number }>();

  async get(sessionId: string) {
    const session = this.sessions.get(sessionId);
    if (!session) return null;

    // Check expiration
    if (session.expiresAt < Date.now()) {
      this.sessions.delete(sessionId);
      return null;
    }

    return session;
  }

  async set(sessionId: string, data: { userId: string; expiresAt: number }) {
    this.sessions.set(sessionId, data);
  }

  async delete(sessionId: string) {
    this.sessions.delete(sessionId);
  }
}

/**
 * Get the session store
 * Uses Redis if available, otherwise falls back to in-memory
 */
export function getSessionStore(): SessionStore {
  if (redis) {
    return new RedisSessionStore(redis);
  }
  return new MemorySessionStore();
}

/**
 * Close Redis connection
 */
export async function closeRedis() {
  if (redis) {
    await redis.quit();
    console.log("🔴 Redis connection closed");
  }
}

// Handle process termination
process.on("SIGTERM", async () => {
  await closeRedis();
});

process.on("SIGINT", async () => {
  await closeRedis();
});
