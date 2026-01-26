# tauri-plugin-redb-cache

[English](#english) | [中文](#中文)

---

## English

A Tauri plugin for HTTP and image caching using [Redb](https://github.com/cberner/redb) with LRU memory cache and compression.

### Features

- **Two-tier caching**: LRU memory cache + Redb persistent storage
- **Automatic compression**: Zlib compression for data >1KB
- **Separate tables**: HTTP responses and images stored separately
- **Configurable**: TTL, memory size, compression threshold
- **Background cleanup**: Automatic expired entry removal
- **Offline-first**: Perfect for field applications with unstable network

### Installation

#### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-redb-cache = { path = "../tauri-plugin-redb-cache" }
```

#### JavaScript/TypeScript

```bash
npm install tauri-plugin-redb-cache-api
# or
pnpm add tauri-plugin-redb-cache-api
```

### Usage

#### Rust Setup

```rust
fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_redb_cache::Builder::new()
                .http_ttl_ms(15 * 24 * 60 * 60 * 1000)  // 15 days
                .image_ttl_ms(7 * 24 * 60 * 60 * 1000)  // 7 days
                .memory_cache_size(100)
                .compress_threshold(1024)
                .build()
        )
        .run(tauri::generate_context!())
        .unwrap();
}
```

#### JavaScript/TypeScript

```typescript
import { 
    cacheGet, cacheSet, getCacheInfo,
    imageCacheGet, imageCacheSet 
} from 'tauri-plugin-redb-cache-api';

// HTTP Cache
await cacheSet('api/users', new Uint8Array([...]));
const result = await cacheGet('api/users');
if (result) {
    const [data, timestamp] = result;
    console.log('Cached at:', new Date(timestamp));
}

// Image Cache
await imageCacheSet('avatar-123', 'data:image/png;base64,...');
const imageResult = await imageCacheGet('avatar-123');
if (imageResult) {
    const [dataUrl, timestamp] = imageResult;
    img.src = dataUrl;
}

// Get cache info
const info = await getCacheInfo();
console.log(`${info.count} entries, ${info.size_bytes} bytes`);
```

### Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `http_ttl_ms` | 15 days | HTTP cache TTL |
| `image_ttl_ms` | 15 days | Image cache TTL |
| `memory_cache_size` | 100 | LRU memory cache entries |
| `compress_threshold` | 1024 | Compress data larger than this (bytes) |
| `cleanup_interval_secs` | 3600 | Auto cleanup interval |
| `db_filename` | "cache.redb" | Database filename |

### API Reference

#### HTTP Cache

| Function | Description |
|----------|-------------|
| `cacheGet(key)` | Get cached data |
| `cacheSet(key, data)` | Set cache |
| `cacheRemove(key)` | Remove entry |
| `cacheClear()` | Clear all HTTP cache |
| `cacheCleanExpired(maxAgeMs)` | Clean expired entries |
| `getCacheInfo()` | Get statistics |
| `getCacheList(includeValue?)` | List all entries |

#### Image Cache

| Function | Description |
|----------|-------------|
| `imageCacheGet(key)` | Get cached image (data URL) |
| `imageCacheSet(key, dataUrl)` | Set image cache |
| `imageCacheRemove(key)` | Remove entry |
| `imageCacheClear()` | Clear all image cache |
| `imageCacheCleanExpired(maxAgeMs)` | Clean expired entries |
| `getImageCacheInfo()` | Get statistics |
| `getImageCacheList()` | List all entries |

### Architecture

```
┌─────────────────────────────────────────┐
│           JavaScript API                 │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Tauri Commands (IPC)             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   LRU Memory Cache (100 entries)         │
│   - Fast access for hot data             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   Redb Persistent Storage                │
│   - Zlib compression (>1KB)              │
│   - http_cache table                     │
│   - image_cache table                    │
└─────────────────────────────────────────┘
```

---

## 中文

基于 [Redb](https://github.com/cberner/redb) 的 Tauri 缓存插件，支持 LRU 内存缓存和数据压缩。

### 特性

- **双层缓存架构**：LRU 内存缓存 + Redb 持久化存储
- **自动压缩**：大于 1KB 的数据自动 Zlib 压缩
- **独立存储表**：HTTP 响应和图片分开存储
- **灵活配置**：TTL、内存大小、压缩阈值均可配置
- **后台清理**：自动清理过期缓存条目
- **离线优先**：适合网络不稳定的外业调查场景

### 安装

#### Rust

在 `Cargo.toml` 中添加：

```toml
[dependencies]
tauri-plugin-redb-cache = { path = "../tauri-plugin-redb-cache" }
```

#### JavaScript/TypeScript

```bash
npm install tauri-plugin-redb-cache-api
# 或
pnpm add tauri-plugin-redb-cache-api
```

### 使用方式

#### Rust 配置

```rust
fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_redb_cache::Builder::new()
                .http_ttl_ms(15 * 24 * 60 * 60 * 1000)  // 15 天
                .image_ttl_ms(7 * 24 * 60 * 60 * 1000)  // 7 天
                .memory_cache_size(100)
                .compress_threshold(1024)
                .build()
        )
        .run(tauri::generate_context!())
        .unwrap();
}
```

#### JavaScript/TypeScript

```typescript
import { 
    cacheGet, cacheSet, getCacheInfo,
    imageCacheGet, imageCacheSet 
} from 'tauri-plugin-redb-cache-api';

// HTTP 缓存
await cacheSet('api/users', new Uint8Array([...]));
const result = await cacheGet('api/users');
if (result) {
    const [data, timestamp] = result;
    console.log('缓存时间:', new Date(timestamp));
}

// 图片缓存
await imageCacheSet('avatar-123', 'data:image/png;base64,...');
const imageResult = await imageCacheGet('avatar-123');
if (imageResult) {
    const [dataUrl, timestamp] = imageResult;
    img.src = dataUrl;
}

// 获取缓存统计
const info = await getCacheInfo();
console.log(`${info.count} 条缓存，${info.size_bytes} 字节`);
```

### 配置选项

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `http_ttl_ms` | 15 天 | HTTP 缓存有效期 |
| `image_ttl_ms` | 15 天 | 图片缓存有效期 |
| `memory_cache_size` | 100 | LRU 内存缓存条目数 |
| `compress_threshold` | 1024 | 压缩阈值（字节） |
| `cleanup_interval_secs` | 3600 | 自动清理间隔（秒） |
| `db_filename` | "cache.redb" | 数据库文件名 |

### API 参考

#### HTTP 缓存

| 函数 | 说明 |
|------|------|
| `cacheGet(key)` | 获取缓存数据 |
| `cacheSet(key, data)` | 设置缓存 |
| `cacheRemove(key)` | 删除缓存 |
| `cacheClear()` | 清空所有 HTTP 缓存 |
| `cacheCleanExpired(maxAgeMs)` | 清理过期缓存 |
| `getCacheInfo()` | 获取统计信息 |
| `getCacheList(includeValue?)` | 列出所有缓存 |

#### 图片缓存

| 函数 | 说明 |
|------|------|
| `imageCacheGet(key)` | 获取缓存图片 (data URL) |
| `imageCacheSet(key, dataUrl)` | 设置图片缓存 |
| `imageCacheRemove(key)` | 删除图片缓存 |
| `imageCacheClear()` | 清空所有图片缓存 |
| `imageCacheCleanExpired(maxAgeMs)` | 清理过期图片 |
| `getImageCacheInfo()` | 获取统计信息 |
| `getImageCacheList()` | 列出所有图片缓存 |

### 架构

```
┌─────────────────────────────────────────┐
│           JavaScript API                 │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Tauri 命令 (IPC)                 │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   LRU 内存缓存 (100 条)                  │
│   - 热点数据快速访问                      │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   Redb 持久化存储                        │
│   - Zlib 压缩 (>1KB)                    │
│   - http_cache 表                       │
│   - image_cache 表                      │
└─────────────────────────────────────────┘
```

---

## License / 许可证

MIT
