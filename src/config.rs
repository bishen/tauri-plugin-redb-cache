//! Plugin configuration

use serde::{Deserialize, Serialize};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// HTTP cache TTL in milliseconds (default: 15 days)
    pub http_ttl_ms: u64,
    /// Image cache TTL in milliseconds (default: 15 days)
    pub image_ttl_ms: u64,
    /// Memory cache size (default: 100 entries)
    pub memory_cache_size: usize,
    /// Compression threshold in bytes (default: 1024)
    pub compress_threshold: usize,
    /// Auto cleanup interval in seconds (default: 3600 = 1 hour)
    pub cleanup_interval_secs: u64,
    /// Database file name (default: "cache.redb")
    pub db_filename: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            http_ttl_ms: 15 * 24 * 60 * 60 * 1000,      // 15 days
            image_ttl_ms: 15 * 24 * 60 * 60 * 1000,     // 15 days
            memory_cache_size: 100,
            compress_threshold: 1024,                    // 1KB
            cleanup_interval_secs: 3600,                 // 1 hour
            db_filename: "cache.redb".to_string(),
        }
    }
}

/// Plugin builder
#[derive(Default)]
pub struct Builder {
    pub(crate) config: CacheConfig,
}

impl Builder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set HTTP cache TTL in milliseconds
    pub fn http_ttl_ms(mut self, ttl: u64) -> Self {
        self.config.http_ttl_ms = ttl;
        self
    }

    /// Set image cache TTL in milliseconds
    pub fn image_ttl_ms(mut self, ttl: u64) -> Self {
        self.config.image_ttl_ms = ttl;
        self
    }

    /// Set memory cache size
    pub fn memory_cache_size(mut self, size: usize) -> Self {
        self.config.memory_cache_size = size;
        self
    }

    /// Set compression threshold in bytes
    pub fn compress_threshold(mut self, threshold: usize) -> Self {
        self.config.compress_threshold = threshold;
        self
    }

    /// Set auto cleanup interval in seconds
    pub fn cleanup_interval_secs(mut self, interval: u64) -> Self {
        self.config.cleanup_interval_secs = interval;
        self
    }

    /// Set database filename
    pub fn db_filename(mut self, filename: impl Into<String>) -> Self {
        self.config.db_filename = filename.into();
        self
    }
}
