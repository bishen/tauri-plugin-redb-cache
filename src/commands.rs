//! Tauri commands for cache operations

use crate::cache::{
    cache_get_impl, cache_set_impl, cache_remove_impl, cache_clear_impl, cache_clean_expired_impl,
    cache_info_impl, cache_list_impl,
    image_cache_get_impl, image_cache_set_impl, image_cache_remove_impl,
    image_cache_clean_expired_impl, image_cache_clear_impl, image_cache_info_impl, image_cache_list_impl,
    CacheInfo, CacheItem,
};

// ==================== HTTP Cache Commands ====================

#[tauri::command]
pub async fn cache_get(key: String) -> Result<Option<(Vec<u8>, u64)>, String> {
    cache_get_impl(key).await
}

#[tauri::command]
pub async fn cache_set(key: String, data: Vec<u8>) -> Result<(), String> {
    cache_set_impl(key, data).await
}

#[tauri::command]
pub async fn cache_remove(key: String) -> Result<(), String> {
    cache_remove_impl(key).await
}

#[tauri::command]
pub async fn cache_clear() -> Result<u64, String> {
    cache_clear_impl().await
}

#[tauri::command]
pub async fn cache_clean_expired(max_age_ms: u64) -> Result<u64, String> {
    cache_clean_expired_impl(max_age_ms).await
}

#[tauri::command]
pub fn cache_info() -> Result<CacheInfo, String> {
    cache_info_impl()
}

#[tauri::command]
pub fn cache_list(include_value: Option<bool>) -> Result<Vec<CacheItem>, String> {
    cache_list_impl(include_value)
}

// ==================== Image Cache Commands ====================

#[tauri::command]
pub async fn image_cache_get(key: String) -> Result<Option<(String, u64)>, String> {
    image_cache_get_impl(key).await
}

#[tauri::command]
pub async fn image_cache_set(key: String, data_url: String) -> Result<(), String> {
    image_cache_set_impl(key, data_url).await
}

#[tauri::command]
pub async fn image_cache_remove(key: String) -> Result<(), String> {
    image_cache_remove_impl(key).await
}

#[tauri::command]
pub async fn image_cache_clean_expired(max_age_ms: u64) -> Result<u64, String> {
    image_cache_clean_expired_impl(max_age_ms).await
}

#[tauri::command]
pub async fn image_cache_clear() -> Result<u64, String> {
    image_cache_clear_impl().await
}

#[tauri::command]
pub fn image_cache_info() -> Result<CacheInfo, String> {
    image_cache_info_impl()
}

#[tauri::command]
pub fn image_cache_list() -> Result<Vec<CacheItem>, String> {
    image_cache_list_impl()
}
