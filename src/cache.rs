use bytes::Bytes;
use dashmap::DashMap;
use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, time::Instant};

pub struct CacheEntry {
    pub data: Arc<Bytes>,
    pub content_type: &'static str,
    pub size_bytes: usize,
    pub last_accessed: Instant
}

pub struct FileCache {
    entries: DashMap<String, CacheEntry>,
    max_size_bytes: usize,
    current_size_bytes: AtomicUsize
}

impl FileCache {
    // Make a new instance of the FileCache
    pub fn new(max_size_mb: u16) -> Self {
        Self {
            entries: DashMap::new(),
            max_size_bytes: max_size_mb as usize * 1024 * 1024,
            current_size_bytes: AtomicUsize::new(0)
        }
    }

    // Tells the maximum size an entry can be (no more than 20% of the maximum cache size)
    pub fn get_max_size_per_entry(&self) -> usize {
        (self.max_size_bytes as f64 * 0.2) as usize
    }

    // Get an entry from the FileCache
    pub fn get(&self, key: &str) -> Option<(Arc<Bytes>, &'static str)> {
        let mut entry = self.entries.get_mut(key)?;
        entry.last_accessed = Instant::now();
        Some((Arc::clone(&entry.data), entry.content_type))
    }

    // Add an entry to the FileCache
    pub fn insert(&self, key: String, data: Vec<u8>, content_type: &'static str) {
        let entry_size = data.len();

        // Remove the oldest entries until there is enough space for the new one
        while self.current_size_bytes.load(Ordering::Relaxed) + entry_size > self.max_size_bytes {
            // Find the least recently accessed entry
            let oldest_key = self.entries
                .iter()
                .min_by_key(|entry| entry.last_accessed)
                .map(|entry| entry.key().clone());

            match oldest_key {
                Some(key) => self.remove(&key),
                None => break // If the cache is empty but the file still cannot be added (somehow) just stop there
            }
        }

        // Add the entry and update the cache size
        self.current_size_bytes.fetch_add(entry_size, Ordering::Relaxed);
        self.entries.insert(key, CacheEntry {
            data: Arc::new(Bytes::from(data)),
            content_type,
            size_bytes: entry_size,
            last_accessed: Instant::now()
        });
    }

    // Remove an entry from the FileCache
    fn remove(&self, key: &str) {
        if let Some((_, entry)) = self.entries.remove(key) {
            self.current_size_bytes.fetch_sub(entry.size_bytes, Ordering::Relaxed);
        }
    }
}