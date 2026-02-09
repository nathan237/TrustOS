//! Block Cache — LRU sector cache for TrustFS
//!
//! Reduces disk I/O by caching recently accessed sectors in memory.
//! Write-back: dirty sectors are flushed on eviction or explicit sync.

use alloc::collections::BTreeMap;
use spin::Mutex;

const CACHE_SIZE: usize = 256; // 256 × 512B = 128KB cache
const SECTOR_SIZE: usize = 512;

struct CacheEntry {
    data: [u8; SECTOR_SIZE],
    dirty: bool,
    access: u64,     // LRU counter
}

pub struct BlockCache {
    entries: BTreeMap<u64, CacheEntry>,
    counter: u64,
    read_fn: fn(u64, &mut [u8; SECTOR_SIZE]) -> Result<(), ()>,
    write_fn: fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
}

impl BlockCache {
    pub fn new(
        read_fn: fn(u64, &mut [u8; SECTOR_SIZE]) -> Result<(), ()>,
        write_fn: fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
    ) -> Self {
        Self {
            entries: BTreeMap::new(),
            counter: 0,
            read_fn,
            write_fn,
        }
    }

    /// Read a sector, serving from cache if available
    pub fn read(&mut self, sector: u64, buf: &mut [u8; SECTOR_SIZE]) -> Result<(), ()> {
        self.counter += 1;
        if let Some(entry) = self.entries.get_mut(&sector) {
            entry.access = self.counter;
            buf.copy_from_slice(&entry.data);
            return Ok(());
        }
        // Cache miss — read from disk
        (self.read_fn)(sector, buf)?;
        self.insert(sector, *buf, false);
        Ok(())
    }

    /// Write a sector (write-back: cached, flushed later)
    pub fn write(&mut self, sector: u64, buf: &[u8; SECTOR_SIZE]) -> Result<(), ()> {
        self.counter += 1;
        if let Some(entry) = self.entries.get_mut(&sector) {
            entry.data.copy_from_slice(buf);
            entry.dirty = true;
            entry.access = self.counter;
            return Ok(());
        }
        self.insert(sector, *buf, true);
        Ok(())
    }

    /// Insert into cache, evicting LRU if full
    fn insert(&mut self, sector: u64, data: [u8; SECTOR_SIZE], dirty: bool) {
        if self.entries.len() >= CACHE_SIZE {
            self.evict_lru();
        }
        self.entries.insert(sector, CacheEntry {
            data,
            dirty,
            access: self.counter,
        });
    }

    /// Evict least recently used entry (flush if dirty)
    fn evict_lru(&mut self) {
        let lru_sector = self.entries.iter()
            .min_by_key(|(_, e)| e.access)
            .map(|(&s, _)| s);
        if let Some(sector) = lru_sector {
            if let Some(entry) = self.entries.remove(&sector) {
                if entry.dirty {
                    let _ = (self.write_fn)(sector, &entry.data);
                }
            }
        }
    }

    /// Flush all dirty entries to disk
    pub fn sync(&mut self) -> Result<(), ()> {
        for (&sector, entry) in self.entries.iter_mut() {
            if entry.dirty {
                (self.write_fn)(sector, &entry.data)?;
                entry.dirty = false;
            }
        }
        Ok(())
    }

    /// Invalidate a specific sector (force re-read next time)
    pub fn invalidate(&mut self, sector: u64) {
        if let Some(entry) = self.entries.remove(&sector) {
            if entry.dirty {
                let _ = (self.write_fn)(sector, &entry.data);
            }
        }
    }

    /// Number of cached entries
    pub fn cached_count(&self) -> usize {
        self.entries.len()
    }

    /// Number of dirty entries
    pub fn dirty_count(&self) -> usize {
        self.entries.values().filter(|e| e.dirty).count()
    }
}

/// Global block cache instance
static BLOCK_CACHE: Mutex<Option<BlockCache>> = Mutex::new(None);

/// Initialize the global block cache
pub fn init(
    read_fn: fn(u64, &mut [u8; SECTOR_SIZE]) -> Result<(), ()>,
    write_fn: fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
) {
    *BLOCK_CACHE.lock() = Some(BlockCache::new(read_fn, write_fn));
}

/// Read through cache
pub fn cached_read(sector: u64, buf: &mut [u8; SECTOR_SIZE]) -> Result<(), ()> {
    if let Some(cache) = BLOCK_CACHE.lock().as_mut() {
        cache.read(sector, buf)
    } else {
        Err(()) // Cache not initialized — caller should fall back to direct I/O
    }
}

/// Write through cache
pub fn cached_write(sector: u64, buf: &[u8; SECTOR_SIZE]) -> Result<(), ()> {
    if let Some(cache) = BLOCK_CACHE.lock().as_mut() {
        cache.write(sector, buf)
    } else {
        Err(())
    }
}

/// Sync all dirty blocks to disk
pub fn sync() -> Result<(), ()> {
    if let Some(cache) = BLOCK_CACHE.lock().as_mut() {
        cache.sync()
    } else {
        Ok(())
    }
}

/// Get cache statistics
pub fn stats() -> (usize, usize) {
    if let Some(cache) = BLOCK_CACHE.lock().as_ref() {
        (cache.cached_count(), cache.dirty_count())
    } else {
        (0, 0)
    }
}
