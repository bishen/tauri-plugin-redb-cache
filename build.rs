const COMMANDS: &[&str] = &[
    "cache_get",
    "cache_set",
    "cache_remove",
    "cache_clear",
    "cache_clean_expired",
    "cache_info",
    "cache_list",
    "image_cache_get",
    "image_cache_set",
    "image_cache_remove",
    "image_cache_clean_expired",
    "image_cache_clear",
    "image_cache_info",
    "image_cache_list",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
