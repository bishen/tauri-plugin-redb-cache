//! Namespace-based cache implementation using Redb
//!
//! Each namespace gets its own Redb table: `ns_{namespace}`
//! Stores arbitrary JSON values with optional TTL expiration.

use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::cache::{compress_data, decompress_data, now_timestamp, open_db};
use crate::get_config;

/// Entry stored in the namespace cache
#[derive(Serialize, Deserialize)]
struct NsEntry {
    data: Vec<u8>,
    timestamp: u64,
    compressed: bool,
    ttl_ms: Option<u64>,
}

/// Input for batch set operations
#[derive(Deserialize)]
pub struct NsCacheSetEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub ttl_ms: Option<u64>,
}

/// Output for prefix query operations
#[derive(Serialize)]
pub struct NsCacheResult {
    pub key: String,
    pub value: serde_json::Value,
}

fn table_name(ns: &str) -> String {
    format!("ns_{}", ns)
}

fn is_expired(entry: &NsEntry, now: u64) -> bool {
    match entry.ttl_ms {
        Some(ttl) => now > entry.timestamp + ttl,
        None => false,
    }
}

fn encode_value(value: &serde_json::Value) -> Result<(Vec<u8>, bool), String> {
    let json_bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let threshold = get_config().compress_threshold;
    if json_bytes.len() > threshold {
        match compress_data(&json_bytes) {
            Ok(compressed) => Ok((compressed, true)),
            Err(_) => Ok((json_bytes, false)),
        }
    } else {
        Ok((json_bytes, false))
    }
}

fn decode_value(data: &[u8], compressed: bool) -> Result<serde_json::Value, String> {
    let raw = if compressed {
        decompress_data(data)?
    } else {
        data.to_vec()
    };
    serde_json::from_slice(&raw).map_err(|e| e.to_string())
}

// ==================== Namespace Cache Operations ====================

pub fn ns_cache_get_impl(ns: String, key: String) -> Result<Option<serde_json::Value>, String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;

    let table = match read_txn.open_table(table_def) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    match table.get(key.as_str()).map_err(|e| e.to_string())? {
        Some(value) => {
            let entry: NsEntry =
                rmp_serde::from_slice(value.value()).map_err(|e| e.to_string())?;
            if is_expired(&entry, now_timestamp()) {
                return Ok(None);
            }
            let val = decode_value(&entry.data, entry.compressed)?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

pub fn ns_cache_set_impl(
    ns: String,
    key: String,
    value: serde_json::Value,
    ttl_ms: Option<u64>,
) -> Result<(), String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let (data, compressed) = encode_value(&value)?;
    let timestamp = now_timestamp();

    let entry = NsEntry {
        data,
        timestamp,
        compressed,
        ttl_ms,
    };
    let serialized = rmp_serde::to_vec(&entry).map_err(|e| e.to_string())?;

    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = write_txn.open_table(table_def).map_err(|e| e.to_string())?;
        table
            .insert(key.as_str(), serialized.as_slice())
            .map_err(|e| e.to_string())?;
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn ns_cache_remove_impl(ns: String, key: String) -> Result<(), String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = write_txn.open_table(table_def).map_err(|e| e.to_string())?;
        table.remove(key.as_str()).ok();
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn ns_cache_get_batch_impl(
    ns: String,
    keys: Vec<String>,
) -> Result<Vec<Option<serde_json::Value>>, String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;

    let table = match read_txn.open_table(table_def) {
        Ok(t) => t,
        Err(_) => return Ok(keys.iter().map(|_| None).collect()),
    };

    let now = now_timestamp();
    let mut results = Vec::with_capacity(keys.len());
    for key in &keys {
        match table.get(key.as_str()).map_err(|e| e.to_string())? {
            Some(value) => {
                let entry: NsEntry =
                    rmp_serde::from_slice(value.value()).map_err(|e| e.to_string())?;
                if is_expired(&entry, now) {
                    results.push(None);
                } else {
                    let val = decode_value(&entry.data, entry.compressed)?;
                    results.push(Some(val));
                }
            }
            None => results.push(None),
        }
    }
    Ok(results)
}

pub fn ns_cache_set_batch_impl(
    ns: String,
    entries: Vec<NsCacheSetEntry>,
) -> Result<(), String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let timestamp = now_timestamp();

    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = write_txn.open_table(table_def).map_err(|e| e.to_string())?;
        for e in &entries {
            let (data, compressed) = encode_value(&e.value)?;
            let ns_entry = NsEntry {
                data,
                timestamp,
                compressed,
                ttl_ms: e.ttl_ms,
            };
            let serialized = rmp_serde::to_vec(&ns_entry).map_err(|e| e.to_string())?;
            table
                .insert(e.key.as_str(), serialized.as_slice())
                .map_err(|e| e.to_string())?;
        }
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn ns_cache_get_by_prefix_impl(
    ns: String,
    prefix: String,
) -> Result<Vec<NsCacheResult>, String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let read_txn = db.begin_read().map_err(|e| e.to_string())?;

    let table = match read_txn.open_table(table_def) {
        Ok(t) => t,
        Err(_) => return Ok(vec![]),
    };

    let now = now_timestamp();
    let mut results = Vec::new();
    let iter = table.iter().map_err(|e| e.to_string())?;
    for item in iter {
        let (k, v) = item.map_err(|e| e.to_string())?;
        let key = k.value().to_string();
        if !key.starts_with(&prefix) {
            continue;
        }
        let entry: NsEntry = match rmp_serde::from_slice(v.value()) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if is_expired(&entry, now) {
            continue;
        }
        if let Ok(val) = decode_value(&entry.data, entry.compressed) {
            results.push(NsCacheResult { key, value: val });
        }
    }
    Ok(results)
}

pub fn ns_cache_remove_by_prefix_impl(ns: String, prefix: String) -> Result<u64, String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    let mut removed = 0u64;
    {
        let mut table = write_txn.open_table(table_def).map_err(|e| e.to_string())?;

        // Collect keys first, then remove
        let keys_to_remove: Vec<String> = {
            let iter = table.iter().map_err(|e| e.to_string())?;
            let mut keys = Vec::new();
            for item in iter {
                let (k, _) = item.map_err(|e| e.to_string())?;
                let key = k.value().to_string();
                if key.starts_with(&prefix) {
                    keys.push(key);
                }
            }
            keys
        };

        for key in &keys_to_remove {
            if table.remove(key.as_str()).is_ok() {
                removed += 1;
            }
        }
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(removed)
}

pub fn ns_cache_clear_impl(ns: String) -> Result<u64, String> {
    let tname = table_name(&ns);
    let table_def: TableDefinition<&str, &[u8]> = TableDefinition::new(&tname);
    let db = open_db()?;
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;
    let mut removed = 0u64;
    {
        let mut table = write_txn.open_table(table_def).map_err(|e| e.to_string())?;

        let keys: Vec<String> = {
            let iter = table.iter().map_err(|e| e.to_string())?;
            let mut ks = Vec::new();
            for item in iter {
                let (k, _) = item.map_err(|e| e.to_string())?;
                ks.push(k.value().to_string());
            }
            ks
        };

        for key in &keys {
            if table.remove(key.as_str()).is_ok() {
                removed += 1;
            }
        }
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(removed)
}
