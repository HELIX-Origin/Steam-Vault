// Serverless In-Memory & Persistent Redis-compatible Client Interface.
//
// Native to Tauri v2: Backed by Rust in-memory RwLock (RAM, <0.01ms latency)
// and SQLite disk persistence (app_kv table with WAL mode).
// Eliminates unnecessary storefront API calls and provides instant loading
// of cached game metadata, session state, and platform assets.

import { api } from "./api";

export interface RedisClient {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, modeOrTtl?: "EX" | number, ttlSecs?: number): Promise<boolean>;
  del(...keys: string[]): Promise<number>;
  exists(...keys: string[]): Promise<number>;
  keys(pattern?: string): Promise<string[]>;
  mget(...keys: string[]): Promise<(string | null)[]>;
  mset(pairs: Record<string, string> | [string, string][]): Promise<boolean>;
  clearExpired(): Promise<number>;
  getJson<T>(key: string): Promise<T | null>;
  setJson<T>(key: string, value: T, ttlSecs?: number): Promise<boolean>;
}

export const redis: RedisClient = {
  /**
   * Retrieve a value by key. Returns cached string or null if absent/expired.
   */
  async get(key: string): Promise<string | null> {
    try {
      return await api.kvGet(key);
    } catch {
      return null;
    }
  },

  /**
   * Set a key-value pair with optional TTL.
   * Supports `set(key, value)`, `set(key, value, 3600)`, and `set(key, value, 'EX', 3600)`.
   */
  async set(
    key: string,
    value: string,
    modeOrTtl?: "EX" | number,
    ttlSecs?: number
  ): Promise<boolean> {
    let ttl: number | undefined;
    if (typeof modeOrTtl === "number") {
      ttl = modeOrTtl;
    } else if (modeOrTtl === "EX" && typeof ttlSecs === "number") {
      ttl = ttlSecs;
    }

    try {
      return await api.kvSet(key, value, ttl ?? null);
    } catch {
      return false;
    }
  },

  /**
   * Delete one or more keys. Returns number of keys requested to delete.
   */
  async del(...keys: string[]): Promise<number> {
    let deleted = 0;
    for (const key of keys) {
      try {
        await api.kvDelete(key);
        deleted++;
      } catch {
        // ignore
      }
    }
    return deleted;
  },

  /**
   * Check if one or more keys exist. Returns count of existing keys.
   */
  async exists(...keys: string[]): Promise<number> {
    let count = 0;
    for (const key of keys) {
      try {
        if (await api.kvHas(key)) {
          count++;
        }
      } catch {
        // ignore
      }
    }
    return count;
  },

  /**
   * Find keys matching an optional wildcard pattern (e.g. `meta:*`, `game:*`).
   */
  async keys(pattern?: string): Promise<string[]> {
    try {
      return await api.kvKeys(pattern ?? null);
    } catch {
      return [];
    }
  },

  /**
   * Multi-get: retrieve multiple keys in order.
   */
  async mget(...keys: string[]): Promise<(string | null)[]> {
    try {
      const map = await api.kvMget(keys);
      return keys.map((k) => map[k] ?? null);
    } catch {
      return keys.map(() => null);
    }
  },

  /**
   * Multi-set: store multiple key-value pairs atomically.
   */
  async mset(pairs: Record<string, string> | [string, string][]): Promise<boolean> {
    const list: [string, string][] = Array.isArray(pairs)
      ? pairs
      : Object.entries(pairs);

    try {
      return await api.kvMset(list);
    } catch {
      return false;
    }
  },

  /**
   * Purge expired entries from cache.
   */
  async clearExpired(): Promise<number> {
    try {
      return await api.kvClearExpired();
    } catch {
      return 0;
    }
  },

  /**
   * Retrieve and deserialize a structured JSON value.
   */
  async getJson<T>(key: string): Promise<T | null> {
    return api.kvGetJson<T>(key);
  },

  /**
   * Serialize and store a structured JSON value with optional TTL.
   */
  async setJson<T>(key: string, value: T, ttlSecs?: number): Promise<boolean> {
    return api.kvSetJson<T>(key, value, ttlSecs ?? null);
  },
};
