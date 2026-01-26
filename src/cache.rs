//! Core cache implementation using Redb with LRU memory cache and compression

use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use lru::LruCache;
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Runtime};

use crate::get_config;

pub(crate) const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("http_cache");
pub(crate) const IMAGE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("image_cache");

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();
static MEMORY_CACHE: OnceLock<Mutex<LruCache<String, MemoryCacheEntry>>> = OnceLock::new();
static IMAGE_MEMORY_CACHE: OnceLock<Mutex<LruCache<String, MemoryCacheEntry>>> = OnceLock::new();

#[derive(Clone)]
pub(crate) struct MemoryCacheEntry {
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CacheEntry {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub compressed: bool,
}

/// Initialize memory cache with configured size
pub(crate) fn get_memory_cache() -> &'static Mutex<LruCache<String, MemoryCacheEntry>> {
    MEMORY_CACHE.get_or_init(|| {
        let size = get_config().memory_cache_size;
        Mutex::new(LruCache::new(NonZeroUsize::new(size).unwrap_or(NonZeroUsize::new(100).unwrap())))
    })
}

pub(crate) fn get_image_memory_cache() -> &'static Mutex<LruCache<String, MemoryCacheEntry>> {
    IMAGE_MEMORY_CACHE.get_or_init(|| {
        let size = get_config().memory_cache_size;
        Mutex::new(LruCache::new(NonZeroUsize::new(size).unwrap_or(NonZeroUsize::new(100).unwrap())))
    })
}

/// Compress data using Zlib
pub(crate) fn compress_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(data, Compression::fast());
    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).map_err(|e| e.to_string())?;
    Ok(compressed)
}

/// Decompress data using Zlib
pub(crate) fn decompress_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| e.to_string())?;
    Ok(decompressed)
}

/// Initialize cache path and start background cleanup task
pub fn init_cache<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config();
    
    DB_PATH.get_or_init(|| {
        let data_dir = app.path().app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("cache");
        
        std::fs::create_dir_all(&data_dir).ok();
        log::info!("[redb-cache] Database path: {:?}", data_dir);
        data_dir.join(&config.db_filename)
    });
    
    // Start background cleanup task
    start_cleanup_task();
    
    Ok(())
}

/// Start background auto cleanup task
fn start_cleanup_task() {
    let config = get_config();
    let interval = config.cleanup_interval_secs;
    let http_ttl = config.http_ttl_ms;
    let image_ttl = config.image_ttl_ms;
    
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(interval));
            
            // Clean expired HTTP cache
            if let Err(e) = cleanup_expired_sync(CACHE_TABLE, http_ttl) {
                log::warn!("[redb-cache] Auto cleanup HTTP cache failed: {}", e);
            }
            
            // Clean expired image cache
            if let Err(e) = cleanup_expired_sync(IMAGE_TABLE, image_ttl) {
                log::warn!("[redb-cache] Auto cleanup image cache failed: {}", e);
            }
            
            log::info!("[redb-cache] Auto cleanup completed");
        }
    });
}

/// Sync cleanup expired entries (for background task)
pub(crate) fn cleanup_expired_sync(table_def: TableDefinition<&str, &[u8]>, max_age_ms: u64) -> Result<u64, String> {
    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    let now = now_timestamp();
    let mut removed = 0u64;
    
    {
        let mut table = match write_txn.open_table(table_def) {
            Ok(t) => t,
            Err(_) => return Ok(0),
        };
        
        let expired_keys: Vec<String> = table
            .iter()
            .map_err(|e| e.to_string())?
            .filter_map(|r| {
                r.ok().and_then(|(k, v)| {
                    let entry: CacheEntry = rmp_serde::from_slice(v.value()).ok()?;
                    if now - entry.timestamp > max_age_ms {
                        Some(k.value().to_string())
                    } else {
                        None
                    }
                })
            })
            .collect();
        
        for key in expired_keys {
            if table.remove(key.as_str()).is_ok() {
                removed += 1;
            }
        }
    }
    
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(removed)
}

pub(crate) fn get_db_path() -> PathBuf {
    DB_PATH
        .get_or_init(|| {
            // Fallback path if init_cache was not called
            let config = get_config();
            let data_dir = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("tauri-plugin-redb-cache")
                .join("cache");
            
            std::fs::create_dir_all(&data_dir).ok();
            data_dir.join(&config.db_filename)
        })
        .clone()
}

pub(crate) fn open_db() -> Result<Database, String> {
    Database::create(get_db_path()).map_err(|e| format!("Failed to open cache database: {}", e))
}

pub(crate) fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ==================== HTTP Cache Operations ====================

pub async fn cache_get_impl(key: String) -> Result<Option<(Vec<u8>, u64)>, String> {
    let config = get_config();
    let ttl = config.http_ttl_ms;
    
    // 1. Check memory cache first
    {
        let mut cache = get_memory_cache().lock().map_err(|e| e.to_string())?;
        if let Some(entry) = cache.get(&key) {
            let now = now_timestamp();
            if now - entry.timestamp < ttl {
                return Ok(Some((entry.data.clone(), entry.timestamp)));
            }
        }
    }
    
    // 2. Check disk cache
    let key_clone = key.clone();
    let result: Option<(Vec<u8>, u64)> = tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let read_txn = db.begin_read().map_err(|e| e.to_string())?;
        
        let table = match read_txn.open_table(CACHE_TABLE) {
            Ok(t) => t,
            Err(_) => return Result::<_, String>::Ok(None),
        };
        
        match table.get(key_clone.as_str()).map_err(|e| e.to_string())? {
            Some(value) => {
                let entry: CacheEntry = rmp_serde::from_slice(value.value())
                    .map_err(|e| e.to_string())?;
                
                // Decompress if needed
                let data = if entry.compressed {
                    decompress_data(&entry.data)?
                } else {
                    entry.data
                };
                
                Ok(Some((data, entry.timestamp)))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())??;
    
    // 3. Update memory cache if disk hit
    if let Some((ref data, timestamp)) = result {
        let mut cache = get_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.put(key, MemoryCacheEntry {
            data: data.clone(),
            timestamp,
        });
    }
    
    Ok(result)
}

pub async fn cache_set_impl(key: String, data: Vec<u8>) -> Result<(), String> {
    let config = get_config();
    let compress_threshold = config.compress_threshold;
    let timestamp = now_timestamp();
    let data_clone = data.clone();
    let key_clone = key.clone();
    
    // 1. Write to memory cache
    {
        let mut cache = get_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.put(key_clone.clone(), MemoryCacheEntry {
            data: data_clone,
            timestamp,
        });
    }
    
    // 2. Write to disk asynchronously (with compression)
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        {
            let mut table = write_txn.open_table(CACHE_TABLE).map_err(|e| e.to_string())?;
            
            // Compress if exceeds threshold
            let (stored_data, compressed) = if data.len() > compress_threshold {
                match compress_data(&data) {
                    Ok(compressed_data) => (compressed_data, true),
                    Err(_) => (data, false),
                }
            } else {
                (data, false)
            };
            
            let entry = CacheEntry {
                data: stored_data,
                timestamp,
                compressed,
            };
            
            let serialized = rmp_serde::to_vec(&entry).map_err(|e| e.to_string())?;
            table.insert(key_clone.as_str(), serialized.as_slice()).map_err(|e| e.to_string())?;
        }
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn cache_remove_impl(key: String) -> Result<(), String> {
    // Remove from memory cache
    {
        let mut cache = get_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.pop(&key);
    }
    
    // Remove from disk
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        {
            let mut table = write_txn.open_table(CACHE_TABLE).map_err(|e| e.to_string())?;
            table.remove(key.as_str()).map_err(|e| e.to_string())?;
        }
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn cache_clear_impl() -> Result<u64, String> {
    // Clear memory cache
    {
        let mut cache = get_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.clear();
    }
    
    // Clear disk cache
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        let count = {
            let mut table = write_txn.open_table(CACHE_TABLE).map_err(|e| e.to_string())?;
            let count = table.len().map_err(|e| e.to_string())?;
            
            let keys: Vec<String> = table
                .iter()
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok().map(|(k, _)| k.value().to_string()))
                .collect();
            
            for key in keys {
                table.remove(key.as_str()).ok();
            }
            
            count
        };
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(count)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn cache_clean_expired_impl(max_age_ms: u64) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cleanup_expired_sync(CACHE_TABLE, max_age_ms)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ==================== Image Cache Operations ====================

pub async fn image_cache_get_impl(key: String) -> Result<Option<(String, u64)>, String> {
    let config = get_config();
    let ttl = config.image_ttl_ms;
    
    // 1. Check memory cache first
    {
        let mut cache = get_image_memory_cache().lock().map_err(|e| e.to_string())?;
        if let Some(entry) = cache.get(&key) {
            let now = now_timestamp();
            if now - entry.timestamp < ttl {
                let data_url = String::from_utf8(entry.data.clone())
                    .map_err(|e| e.to_string())?;
                return Ok(Some((data_url, entry.timestamp)));
            }
        }
    }
    
    // 2. Check disk cache
    let key_clone = key.clone();
    let result: Option<(String, u64, Vec<u8>)> = tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let read_txn = db.begin_read().map_err(|e| e.to_string())?;
        
        let table = match read_txn.open_table(IMAGE_TABLE) {
            Ok(t) => t,
            Err(_) => return Result::<_, String>::Ok(None),
        };
        
        match table.get(key_clone.as_str()).map_err(|e| e.to_string())? {
            Some(value) => {
                let entry: CacheEntry = rmp_serde::from_slice(value.value())
                    .map_err(|e| e.to_string())?;
                
                let data = if entry.compressed {
                    decompress_data(&entry.data)?
                } else {
                    entry.data
                };
                
                let data_url = String::from_utf8(data.clone())
                    .map_err(|e| e.to_string())?;
                Ok(Some((data_url, entry.timestamp, data)))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())??;
    
    // 3. Update memory cache if disk hit
    if let Some((ref data_url, timestamp, ref data)) = result {
        let mut cache = get_image_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.put(key, MemoryCacheEntry {
            data: data.clone(),
            timestamp,
        });
        return Ok(Some((data_url.clone(), timestamp)));
    }
    
    Ok(None)
}

pub async fn image_cache_set_impl(key: String, data_url: String) -> Result<(), String> {
    let config = get_config();
    let compress_threshold = config.compress_threshold;
    let timestamp = now_timestamp();
    let data = data_url.clone().into_bytes();
    let key_clone = key.clone();
    
    // 1. Write to memory cache
    {
        let mut cache = get_image_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.put(key_clone.clone(), MemoryCacheEntry {
            data: data.clone(),
            timestamp,
        });
    }
    
    // 2. Write to disk asynchronously (with compression)
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        {
            let mut table = write_txn.open_table(IMAGE_TABLE).map_err(|e| e.to_string())?;
            
            let (stored_data, compressed) = if data.len() > compress_threshold {
                match compress_data(&data) {
                    Ok(compressed_data) => (compressed_data, true),
                    Err(_) => (data, false),
                }
            } else {
                (data, false)
            };
            
            let entry = CacheEntry {
                data: stored_data,
                timestamp,
                compressed,
            };
            
            let serialized = rmp_serde::to_vec(&entry).map_err(|e| e.to_string())?;
            table.insert(key_clone.as_str(), serialized.as_slice()).map_err(|e| e.to_string())?;
        }
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn image_cache_remove_impl(key: String) -> Result<(), String> {
    // Remove from memory cache
    {
        let mut cache = get_image_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.pop(&key);
    }
    
    // Remove from disk
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        {
            let mut table = write_txn.open_table(IMAGE_TABLE).map_err(|e| e.to_string())?;
            table.remove(key.as_str()).map_err(|e| e.to_string())?;
        }
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn image_cache_clean_expired_impl(max_age_ms: u64) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cleanup_expired_sync(IMAGE_TABLE, max_age_ms)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn image_cache_clear_impl() -> Result<u64, String> {
    // Clear memory cache
    {
        let mut cache = get_image_memory_cache().lock().map_err(|e| e.to_string())?;
        cache.clear();
    }
    
    // Clear disk cache
    tauri::async_runtime::spawn_blocking(move || {
        let db = open_db()?;
        let write_txn = db.begin_write().map_err(|e| e.to_string())?;
        
        let count = {
            let mut table = write_txn.open_table(IMAGE_TABLE).map_err(|e| e.to_string())?;
            let count = table.len().map_err(|e| e.to_string())?;
            
            let keys: Vec<String> = table
                .iter()
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok().map(|(k, _)| k.value().to_string()))
                .collect();
            
            for key in keys {
                table.remove(key.as_str()).ok();
            }
            
            count
        };
        
        write_txn.commit().map_err(|e| e.to_string())?;
        Ok(count)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ==================== Cache Info ====================

#[derive(Serialize)]
pub struct CacheInfo {
    pub count: u64,
    pub size_bytes: usize,
    pub path: String,
    pub memory_count: usize,
    pub compressed_count: u64,
}

#[derive(Serialize)]
pub struct CacheItem {
    pub key: String,
    pub size: usize,
    pub timestamp: u64,
    pub compressed: bool,
    pub value: Option<String>,
}

pub fn cache_info_impl() -> Result<CacheInfo, String> {
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;
    
    let (count, size, compressed_count) = match read_txn.open_table(CACHE_TABLE) {
        Ok(table) => {
            let count = table.len().unwrap_or(0);
            let mut size = 0usize;
            let mut compressed = 0u64;
            
            for result in table.iter().map_err(|e| e.to_string())? {
                if let Ok((_, v)) = result {
                    size += v.value().len();
                    if let Ok(entry) = rmp_serde::from_slice::<CacheEntry>(v.value()) {
                        if entry.compressed {
                            compressed += 1;
                        }
                    }
                }
            }
            (count, size, compressed)
        }
        Err(_) => (0, 0, 0),
    };
    
    let memory_count = get_memory_cache()
        .lock()
        .map(|c| c.len())
        .unwrap_or(0);
    
    Ok(CacheInfo {
        count,
        size_bytes: size,
        path: get_db_path().to_string_lossy().to_string(),
        memory_count,
        compressed_count,
    })
}

pub fn cache_list_impl(include_value: Option<bool>) -> Result<Vec<CacheItem>, String> {
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;
    let with_value = include_value.unwrap_or(false);
    
    let table = match read_txn.open_table(CACHE_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(vec![]),
    };
    
    let mut items = Vec::new();
    for result in table.iter().map_err(|e| e.to_string())? {
        if let Ok((k, v)) = result {
            let key = k.value().to_string();
            let size = v.value().len();
            
            let (timestamp, compressed, value) = match rmp_serde::from_slice::<CacheEntry>(v.value()) {
                Ok(entry) => {
                    let val = if with_value {
                        let data = if entry.compressed {
                            decompress_data(&entry.data).unwrap_or_default()
                        } else {
                            entry.data
                        };
                        match rmp_serde::from_slice::<serde_json::Value>(&data) {
                            Ok(json_val) => {
                                let json_str = serde_json::to_string_pretty(&json_val)
                                    .unwrap_or_else(|_| String::from_utf8_lossy(&data).to_string());
                                Some(json_str.chars().take(2000).collect::<String>())
                            }
                            Err(_) => {
                                Some(String::from_utf8_lossy(&data).chars().take(2000).collect())
                            }
                        }
                    } else {
                        None
                    };
                    (entry.timestamp, entry.compressed, val)
                }
                Err(_) => (0, false, None),
            };
            
            items.push(CacheItem {
                key,
                size,
                timestamp,
                compressed,
                value,
            });
        }
    }
    
    items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(items)
}

pub fn image_cache_info_impl() -> Result<CacheInfo, String> {
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;
    
    let (count, size, compressed_count) = match read_txn.open_table(IMAGE_TABLE) {
        Ok(table) => {
            let count = table.len().unwrap_or(0);
            let mut size = 0usize;
            let mut compressed = 0u64;
            
            for result in table.iter().map_err(|e| e.to_string())? {
                if let Ok((_, v)) = result {
                    size += v.value().len();
                    if let Ok(entry) = rmp_serde::from_slice::<CacheEntry>(v.value()) {
                        if entry.compressed {
                            compressed += 1;
                        }
                    }
                }
            }
            (count, size, compressed)
        }
        Err(_) => (0, 0, 0),
    };
    
    let memory_count = get_image_memory_cache()
        .lock()
        .map(|c| c.len())
        .unwrap_or(0);
    
    Ok(CacheInfo {
        count,
        size_bytes: size,
        path: get_db_path().to_string_lossy().to_string(),
        memory_count,
        compressed_count,
    })
}

pub fn image_cache_list_impl() -> Result<Vec<CacheItem>, String> {
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;
    
    let table = match read_txn.open_table(IMAGE_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(vec![]),
    };
    
    let mut items = Vec::new();
    for result in table.iter().map_err(|e| e.to_string())? {
        if let Ok((k, v)) = result {
            let key = k.value().to_string();
            let size = v.value().len();
            
            let (timestamp, compressed) = match rmp_serde::from_slice::<CacheEntry>(v.value()) {
                Ok(entry) => (entry.timestamp, entry.compressed),
                Err(_) => (0, false),
            };
            
            items.push(CacheItem {
                key,
                size,
                timestamp,
                compressed,
                value: None,  // Don't include image data preview
            });
        }
    }
    
    items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(items)
}
