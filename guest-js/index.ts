/**
 * Tauri Plugin Redb Cache - JavaScript/TypeScript bindings
 * 
 * @example
 * ```typescript
 * import { cacheGet, cacheSet, getCacheInfo } from 'tauri-plugin-redb-cache-api';
 * 
 * // Set cache
 * await cacheSet('my-key', new Uint8Array([1, 2, 3]));
 * 
 * // Get cache
 * const result = await cacheGet('my-key');
 * if (result) {
 *   const [data, timestamp] = result;
 *   console.log('Data:', data, 'Cached at:', new Date(timestamp));
 * }
 * ```
 */

import { invoke } from '@tauri-apps/api/core';

// ==================== Types ====================

export interface CacheInfo {
  /** Number of entries in disk cache */
  count: number;
  /** Total size in bytes */
  size_bytes: number;
  /** Database file path */
  path: string;
  /** Number of entries in memory cache */
  memory_count: number;
  /** Number of compressed entries */
  compressed_count: number;
}

export interface CacheItem {
  /** Cache key */
  key: string;
  /** Size in bytes */
  size: number;
  /** Timestamp when cached (ms since epoch) */
  timestamp: number;
  /** Whether data is compressed */
  compressed: boolean;
  /** Value preview (only if requested) */
  value?: string;
}

// ==================== HTTP Cache ====================

/**
 * Get cached data by key
 * @param key Cache key
 * @returns [data, timestamp] or null if not found
 */
export async function cacheGet(key: string): Promise<[Uint8Array, number] | null> {
  const result = await invoke<[number[], number] | null>('plugin:redb-cache|cache_get', { key });
  if (result) {
    return [new Uint8Array(result[0]), result[1]];
  }
  return null;
}

/**
 * Set cache data
 * @param key Cache key
 * @param data Data to cache (Uint8Array or number[])
 */
export async function cacheSet(key: string, data: Uint8Array | number[]): Promise<void> {
  const dataArray = data instanceof Uint8Array ? Array.from(data) : data;
  await invoke('plugin:redb-cache|cache_set', { key, data: dataArray });
}

/**
 * Remove cache entry
 * @param key Cache key
 */
export async function cacheRemove(key: string): Promise<void> {
  await invoke('plugin:redb-cache|cache_remove', { key });
}

/**
 * Clear all HTTP cache entries
 * @returns Number of entries removed
 */
export async function cacheClear(): Promise<number> {
  return invoke<number>('plugin:redb-cache|cache_clear');
}

/**
 * Clean expired cache entries
 * @param maxAgeMs Maximum age in milliseconds
 * @returns Number of entries removed
 */
export async function cacheCleanExpired(maxAgeMs: number): Promise<number> {
  return invoke<number>('plugin:redb-cache|cache_clean_expired', { maxAgeMs });
}

/**
 * Get HTTP cache statistics
 */
export async function getCacheInfo(): Promise<CacheInfo> {
  return invoke<CacheInfo>('plugin:redb-cache|cache_info');
}

/**
 * List all HTTP cache entries
 * @param includeValue Whether to include value preview
 */
export async function getCacheList(includeValue = false): Promise<CacheItem[]> {
  return invoke<CacheItem[]>('plugin:redb-cache|cache_list', { includeValue });
}

// ==================== Image Cache ====================

/**
 * Get cached image by key
 * @param key Cache key
 * @returns [dataUrl, timestamp] or null if not found
 */
export async function imageCacheGet(key: string): Promise<[string, number] | null> {
  return invoke<[string, number] | null>('plugin:redb-cache|image_cache_get', { key });
}

/**
 * Set image cache
 * @param key Cache key
 * @param dataUrl Image data URL (e.g., "data:image/png;base64,...")
 */
export async function imageCacheSet(key: string, dataUrl: string): Promise<void> {
  await invoke('plugin:redb-cache|image_cache_set', { key, dataUrl });
}

/**
 * Remove image cache entry
 * @param key Cache key
 */
export async function imageCacheRemove(key: string): Promise<void> {
  await invoke('plugin:redb-cache|image_cache_remove', { key });
}

/**
 * Clean expired image cache entries
 * @param maxAgeMs Maximum age in milliseconds
 * @returns Number of entries removed
 */
export async function imageCacheCleanExpired(maxAgeMs: number): Promise<number> {
  return invoke<number>('plugin:redb-cache|image_cache_clean_expired', { maxAgeMs });
}

/**
 * Clear all image cache entries
 * @returns Number of entries removed
 */
export async function imageCacheClear(): Promise<number> {
  return invoke<number>('plugin:redb-cache|image_cache_clear');
}

/**
 * Get image cache statistics
 */
export async function getImageCacheInfo(): Promise<CacheInfo> {
  return invoke<CacheInfo>('plugin:redb-cache|image_cache_info');
}

/**
 * List all image cache entries
 */
export async function getImageCacheList(): Promise<CacheItem[]> {
  return invoke<CacheItem[]>('plugin:redb-cache|image_cache_list');
}
