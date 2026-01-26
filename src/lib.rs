//! # Tauri Plugin Redb Cache
//!
//! A Tauri plugin for HTTP and image caching using Redb with LRU memory cache and compression.
//!
//! ## Features
//! - Two-tier caching: LRU memory cache + Redb persistent storage
//! - Automatic Zlib compression for large data (>1KB)
//! - Separate tables for HTTP responses and images
//! - Configurable TTL, memory cache size, and compression threshold
//! - Background cleanup task for expired entries
//!
//! ## Usage
//! ```rust
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(tauri_plugin_redb_cache::Builder::new().build())
//!         .run(tauri::generate_context!())
//!         .unwrap();
//! }
//! ```

mod cache;
mod commands;
mod config;

pub use config::{CacheConfig, Builder};

use std::sync::OnceLock;
use tauri::{
    plugin::{self, TauriPlugin},
    Runtime,
};

pub(crate) static CONFIG: OnceLock<CacheConfig> = OnceLock::new();

/// Get the plugin configuration
pub fn get_config() -> &'static CacheConfig {
    CONFIG.get_or_init(CacheConfig::default)
}

/// Initialize the plugin with the given app handle
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new().build()
}

impl Builder {
    /// Build the plugin
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        // Store config
        let _ = CONFIG.set(self.config.clone());

        plugin::Builder::new("redb-cache")
            .invoke_handler(tauri::generate_handler![
                commands::cache_get,
                commands::cache_set,
                commands::cache_remove,
                commands::cache_clear,
                commands::cache_clean_expired,
                commands::cache_info,
                commands::cache_list,
                commands::image_cache_get,
                commands::image_cache_set,
                commands::image_cache_remove,
                commands::image_cache_clean_expired,
                commands::image_cache_clear,
                commands::image_cache_info,
                commands::image_cache_list,
            ])
            .setup(|app, _api| {
                cache::init_cache(app)?;
                Ok(())
            })
            .build()
    }
}
