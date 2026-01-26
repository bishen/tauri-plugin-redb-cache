# tauri-plugin-redb-cache-api

[English](#english) | [中文](#中文)

---

## English

JavaScript/TypeScript bindings for [tauri-plugin-redb-cache](https://crates.io/crates/tauri-plugin-redb-cache).

A Tauri plugin for HTTP and image caching using Redb with LRU memory cache and compression.

### Installation

```bash
npm install @bishen/tauri-plugin-redb-cache-api
# or
pnpm add @bishen/tauri-plugin-redb-cache-api
```

### Requirements

- Tauri 2.x
- `@tauri-apps/api` ^2.0.0

### Usage

```typescript
import { 
    cacheGet, cacheSet, cacheRemove, cacheClear,
    getCacheInfo, getCacheList,
    imageCacheGet, imageCacheSet, imageCacheClear,
    getImageCacheInfo
} from '@bishen/tauri-plugin-redb-cache-api';

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

// Get cache statistics
const info = await getCacheInfo();
console.log(`${info.count} entries, ${info.size_bytes} bytes`);
```

### API Reference

#### HTTP Cache

| Function | Description |
|----------|-------------|
| `cacheGet(key)` | Get cached data, returns `[Uint8Array, timestamp]` or `null` |
| `cacheSet(key, data)` | Set cache data |
| `cacheRemove(key)` | Remove a cache entry |
| `cacheClear()` | Clear all HTTP cache |
| `cacheCleanExpired(maxAgeMs)` | Clean expired entries |
| `getCacheInfo()` | Get cache statistics |
| `getCacheList(includeValue?)` | List all cache entries |

#### Image Cache

| Function | Description |
|----------|-------------|
| `imageCacheGet(key)` | Get cached image, returns `[dataUrl, timestamp]` or `null` |
| `imageCacheSet(key, dataUrl)` | Set image cache |
| `imageCacheRemove(key)` | Remove an image cache entry |
| `imageCacheClear()` | Clear all image cache |
| `imageCacheCleanExpired(maxAgeMs)` | Clean expired images |
| `getImageCacheInfo()` | Get image cache statistics |
| `getImageCacheList()` | List all image cache entries |

### License

MIT © 2026 BiShen <bishen@live.com> 算金山™ (https://www.suanjinshan.com/)

---

## 中文

[tauri-plugin-redb-cache](https://crates.io/crates/tauri-plugin-redb-cache) 的 JavaScript/TypeScript 绑定。

基于 Redb 的 Tauri 缓存插件，支持 LRU 内存缓存和数据压缩。

### 安装

```bash
npm install @bishen/tauri-plugin-redb-cache-api
# 或
pnpm add @bishen/tauri-plugin-redb-cache-api
```

### 依赖要求

- Tauri 2.x
- `@tauri-apps/api` ^2.0.0

### 使用方式

```typescript
import { 
    cacheGet, cacheSet, cacheRemove, cacheClear,
    getCacheInfo, getCacheList,
    imageCacheGet, imageCacheSet, imageCacheClear,
    getImageCacheInfo
} from '@bishen/tauri-plugin-redb-cache-api';

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

### API 参考

#### HTTP 缓存

| 函数 | 说明 |
|------|------|
| `cacheGet(key)` | 获取缓存数据，返回 `[Uint8Array, timestamp]` 或 `null` |
| `cacheSet(key, data)` | 设置缓存数据 |
| `cacheRemove(key)` | 删除缓存条目 |
| `cacheClear()` | 清空所有 HTTP 缓存 |
| `cacheCleanExpired(maxAgeMs)` | 清理过期缓存 |
| `getCacheInfo()` | 获取缓存统计信息 |
| `getCacheList(includeValue?)` | 列出所有缓存条目 |

#### 图片缓存

| 函数 | 说明 |
|------|------|
| `imageCacheGet(key)` | 获取缓存图片，返回 `[dataUrl, timestamp]` 或 `null` |
| `imageCacheSet(key, dataUrl)` | 设置图片缓存 |
| `imageCacheRemove(key)` | 删除图片缓存条目 |
| `imageCacheClear()` | 清空所有图片缓存 |
| `imageCacheCleanExpired(maxAgeMs)` | 清理过期图片 |
| `getImageCacheInfo()` | 获取图片缓存统计信息 |
| `getImageCacheList()` | 列出所有图片缓存条目 |

### 许可证

MIT © 2026 BiShen <bishen@live.com> 算金山™ (https://www.suanjinshan.com/)
