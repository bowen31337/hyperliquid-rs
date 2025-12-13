//! Memory allocation optimizations for high-performance trading
//!
//! This module provides various memory optimization strategies:
//! - Arena allocator for short-lived allocations
//! - String interning for symbol names and common strings
//! - Zero-copy parsing with raw JSON values
//! - Object pooling for frequently allocated types
//! - Memory tracking and profiling

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

/// Arena allocator for short-lived allocations
/// Provides fast allocation/deallocation for objects with similar lifetimes
pub struct ArenaAllocator {
    /// Pre-allocated memory chunks
    chunks: Vec<MemoryChunk>,
    /// Current chunk being allocated from
    current_chunk: Option<usize>,
    /// Total allocated bytes
    total_allocated: usize,
    /// Allocation statistics
    stats: Arc<Mutex<AllocationStats>>,
}

/// Memory chunk for arena allocation
struct MemoryChunk {
    /// Raw memory buffer
    buffer: Box<[u8]>,
    /// Current allocation offset
    offset: usize,
}

/// Allocation statistics for monitoring memory usage
#[derive(Debug, Clone)]
pub struct AllocationStats {
    /// Total number of allocations
    total_allocations: usize,
    /// Total bytes allocated
    total_bytes_allocated: usize,
    /// Peak memory usage
    peak_memory_usage: usize,
    /// Current memory usage
    current_memory_usage: usize,
    /// Number of arena allocations
    arena_allocations: usize,
    /// Number of string intern operations
    string_intern_ops: usize,
    /// Number of string cache hits
    string_cache_hits: usize,
    /// Number of string cache misses
    string_cache_misses: usize,
}

/// String interner for deduplicating frequently used strings
/// Especially useful for symbol names (BTC, ETH, etc.)
#[derive(Debug)]
pub struct StringInterner {
    /// String to ID mapping
    string_to_id: HashMap<String, u32>,
    /// ID to string mapping
    id_to_string: Vec<String>,
    /// Statistics
    stats: Arc<Mutex<StringInternStats>>,
}

#[derive(Debug, Clone)]
pub struct StringInternStats {
    pub total_interns: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub unique_strings: usize,
}

/// Zero-copy JSON value wrapper that avoids allocations during parsing
#[derive(Debug, Clone)]
pub enum ZeroCopyValue<'a> {
    /// Raw JSON string (zero-copy)
    Raw(&'a str),
    /// Owned string (when conversion is necessary)
    Owned(String),
    /// Number as f64
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
    /// Array of values
    Array(Vec<ZeroCopyValue<'a>>),
    /// Object with string keys
    Object(HashMap<String, ZeroCopyValue<'a>>),
}

/// Object pool for frequently allocated types
pub struct ObjectPool<T>
where
    T: Default + Clone,
{
    /// Available objects
    pool: Mutex<Vec<T>>,
    /// Maximum pool size
    max_size: usize,
    /// Statistics
    stats: Arc<Mutex<PoolStats>>,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_allocations: usize,
    pub total_releases: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub max_pool_size: usize,
}

impl ArenaAllocator {
    /// Create a new arena allocator with specified chunk size
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            current_chunk: None,
            total_allocated: 0,
            stats: Arc::new(Mutex::new(AllocationStats {
                total_allocations: 0,
                total_bytes_allocated: 0,
                peak_memory_usage: 0,
                current_memory_usage: 0,
                arena_allocations: 0,
                string_intern_ops: 0,
                string_cache_hits: 0,
                string_cache_misses: 0,
            })),
        }
        .allocate_chunk(chunk_size)
    }

    /// Allocate memory from the current chunk or create a new one
    pub fn allocate<T>(&mut self, value: T) -> *mut T
    where
        T: Sized,
    {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Find or create a suitable chunk
        let chunk_idx = self.get_or_create_chunk(size);

        // Allocate from the chunk
        let ptr = self.chunks[chunk_idx].allocate_aligned(size, align);

        if !ptr.is_null() {
            unsafe {
                std::ptr::write(ptr as *mut T, value);
            }

            // Update statistics
            {
                let mut stats = self.stats.lock().unwrap();
                stats.total_allocations += 1;
                stats.total_bytes_allocated += size;
                stats.current_memory_usage += size;
                stats.arena_allocations += 1;
                stats.peak_memory_usage = stats.peak_memory_usage.max(stats.current_memory_usage);
            }
        }

        ptr as *mut T
    }

    /// Allocate a string in the arena (zero-copy when possible)
    pub fn allocate_string(&mut self, s: &str) -> *mut str {
        let bytes = s.as_bytes();
        let ptr = self.allocate_bytes(bytes);

        if !ptr.is_null() {
            // Update statistics
            {
                let mut stats = self.stats.lock().unwrap();
                stats.string_intern_ops += 1;
                stats.current_memory_usage += bytes.len();
                stats.peak_memory_usage = stats.peak_memory_usage.max(stats.current_memory_usage);
            }

            ptr as *mut str
        } else {
            std::ptr::null_mut()
        }
    }

    /// Allocate raw bytes in the arena
    pub fn allocate_bytes(&mut self, bytes: &[u8]) -> *mut u8 {
        let ptr = self.allocate_aligned(bytes.len(), 1);

        if !ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            }
        }

        ptr
    }

    /// Allocate aligned memory from the current chunk
    fn allocate_aligned(&mut self, size: usize, align: usize) -> *mut u8 {
        if let Some(chunk_idx) = self.current_chunk {
            let chunk = &mut self.chunks[chunk_idx];
            let ptr = chunk.allocate_aligned(size, align);

            if !ptr.is_null() {
                return ptr;
            }
        }

        // Current chunk doesn't have space, create a new one
        let chunk_size = size.max(self.estimate_chunk_size());
        self.allocate_chunk(chunk_size);

        if let Some(chunk_idx) = self.current_chunk {
            self.chunks[chunk_idx].allocate_aligned(size, align)
        } else {
            std::ptr::null_mut()
        }
    }

    /// Allocate a new memory chunk
    fn allocate_chunk(&mut self, size: usize) -> &mut MemoryChunk {
        let chunk = MemoryChunk::new(size);
        let chunk_idx = self.chunks.len();
        self.chunks.push(chunk);
        self.current_chunk = Some(chunk_idx);
        &mut self.chunks[chunk_idx]
    }

    /// Get or create a chunk that can accommodate the requested size
    fn get_or_create_chunk(&mut self, size: usize) -> usize {
        if let Some(chunk_idx) = self.current_chunk {
            if self.chunks[chunk_idx].has_space(size) {
                return chunk_idx;
            }
        }

        // Need a new chunk
        let chunk_size = size.max(self.estimate_chunk_size());
        self.allocate_chunk(chunk_size);
        self.current_chunk.unwrap()
    }

    /// Estimate optimal chunk size based on allocation patterns
    fn estimate_chunk_size(&self) -> usize {
        // Simple heuristic: use 64KB chunks for trading data
        64 * 1024
    }

    /// Reset the arena, making all allocated memory available again
    /// This is much faster than deallocating individual objects
    pub fn reset(&mut self) {
        for chunk in &mut self.chunks {
            chunk.reset();
        }
        self.current_chunk = None;
        self.total_allocated = 0;

        // Reset current memory usage but keep peak
        {
            let mut stats = self.stats.lock().unwrap();
            stats.current_memory_usage = 0;
        }
    }

    /// Get allocation statistics
    pub fn get_stats(&self) -> AllocationStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get total allocated bytes
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }
}

impl MemoryChunk {
    /// Create a new memory chunk with the specified size
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size].into_boxed_slice(),
            offset: 0,
        }
    }

    /// Allocate aligned memory from this chunk
    fn allocate_aligned(&mut self, size: usize, align: usize) -> *mut u8 {
        let aligned_offset = self.align_offset(self.offset, align);
        let end_offset = aligned_offset + size;

        if end_offset <= self.buffer.len() {
            let ptr = unsafe {
                self.buffer.as_mut_ptr().add(aligned_offset)
            };
            self.offset = end_offset;
            ptr
        } else {
            std::ptr::null_mut()
        }
    }

    /// Check if the chunk has space for the requested size
    fn has_space(&self, size: usize) -> bool {
        // Account for potential alignment padding
        self.offset + size <= self.buffer.len()
    }

    /// Reset chunk to initial state
    fn reset(&mut self) {
        self.offset = 0;
    }

    /// Align offset to the specified boundary
    fn align_offset(&self, offset: usize, align: usize) -> usize {
        (offset + align - 1) & !(align - 1)
    }
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            string_to_id: HashMap::new(),
            id_to_string: Vec::new(),
            stats: Arc::new(Mutex::new(StringInternStats {
                total_interns: 0,
                cache_hits: 0,
                cache_misses: 0,
                unique_strings: 0,
            })),
        }
    }

    /// Intern a string, returning a stable ID
    pub fn intern(&mut self, s: &str) -> u32 {
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_interns += 1;
        }

        if let Some(&id) = self.string_to_id.get(s) {
            {
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
            }
            return id;
        }

        let id = self.id_to_string.len() as u32;
        self.string_to_id.insert(s.to_string(), id);
        self.id_to_string.push(s.to_string());

        {
            let mut stats = self.stats.lock().unwrap();
            stats.cache_misses += 1;
            stats.unique_strings = self.id_to_string.len();
        }

        id
    }

    /// Get a string by its ID
    pub fn get(&self, id: u32) -> Option<&str> {
        self.id_to_string.get(id as usize).map(|s| s.as_str())
    }

    /// Get statistics
    pub fn get_stats(&self) -> StringInternStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get the number of unique strings interned
    pub fn len(&self) -> usize {
        self.id_to_string.len()
    }

    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.id_to_string.is_empty()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ZeroCopyValue<'a> {
    /// Parse a JSON string into a zero-copy value
    pub fn from_json_str(s: &'a str) -> Result<Self, serde_json::Error> {
        // Use serde_json's raw value support for zero-copy parsing
        let raw_value: Box<serde_json::value::RawValue> = serde_json::from_str(s)?;
        Self::from_raw_value(&raw_value)
    }

    /// Convert from serde_json RawValue
    fn from_raw_value(raw: &Box<serde_json::value::RawValue>) -> Result<Self, serde_json::Error> {
        let s = raw.get();

        // Try to parse as different JSON types
        if s == "null" {
            Ok(Self::Null)
        } else if s == "true" {
            Ok(Self::Bool(true))
        } else if s == "false" {
            Ok(Self::Bool(false))
        } else if let Ok(n) = s.parse::<f64>() {
            Ok(Self::Number(n))
        } else if s.starts_with('"') && s.ends_with('"') {
            // Extract string content without quotes
            let content = &s[1..s.len()-1];
            Ok(Self::Raw(content))
        } else if s.starts_with('[') {
            // Parse as array
            let array: Vec<Box<serde_json::value::RawValue>> = serde_json::from_str(s)?;
            let values = array.into_iter()
                .map(|v| Self::from_raw_value(&v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self::Array(values))
        } else if s.starts_with('{') {
            // Parse as object
            let obj: HashMap<String, Box<serde_json::value::RawValue>> = serde_json::from_str(s)?;
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, Self::from_raw_value(&v)?);
            }
            Ok(Self::Object(map))
        } else {
            Err(serde_json::Error::syntax(
                serde_json::error::Category::Data,
                0,
                "Invalid JSON value".to_string(),
            ))
        }
    }

    /// Get string value if this is a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Raw(s) => Some(s),
            Self::Owned(s) => Some(s),
            _ => None,
        }
    }

    /// Get number value if this is a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get boolean value if this is a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl<T> ObjectPool<T>
where
    T: Default + Clone,
{
    /// Create a new object pool with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::new()),
            max_size,
            stats: Arc::new(Mutex::new(PoolStats {
                total_allocations: 0,
                total_releases: 0,
                pool_hits: 0,
                pool_misses: 0,
                max_pool_size: 0,
            })),
        }
    }

    /// Get an object from the pool or create a new one
    pub fn get(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        stats.total_allocations += 1;

        if let Some(obj) = pool.pop() {
            stats.pool_hits += 1;
            stats.max_pool_size = stats.max_pool_size.max(pool.len());
            PooledObject {
                obj: Some(obj),
                pool: Arc::downgrade(&self.pool),
                stats: Arc::downgrade(&self.stats),
            }
        } else {
            stats.pool_misses += 1;
            PooledObject {
                obj: Some(T::default()),
                pool: Arc::downgrade(&self.pool),
                stats: Arc::downgrade(&self.stats),
            }
        }
    }

    /// Return an object to the pool
    fn return_to_pool(&self, mut obj: T) {
        let mut pool = self.pool.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        stats.total_releases += 1;

        // Reset the object to default state if it implements Default
        obj = T::default();

        if pool.len() < self.max_size {
            pool.push(obj);
            stats.max_pool_size = stats.max_pool_size.max(pool.len());
        }
        // If pool is full, just drop the object
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get current pool size
    pub fn pool_size(&self) -> usize {
        let pool = self.pool.lock().unwrap();
        pool.len()
    }
}

/// RAII wrapper for pooled objects
/// Automatically returns the object to the pool when dropped
pub struct PooledObject<T>
where
    T: Default + Clone,
{
    obj: Option<T>,
    pool: std::sync::Weak<Mutex<Vec<T>>>,
    stats: std::sync::Weak<Mutex<PoolStats>>,
}

impl<T> std::ops::Deref for PooledObject<T>
where
    T: Default + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T>
where
    T: Default + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T>
where
    T: Default + Clone,
{
    fn drop(&mut self) {
        if let (Some(obj), Some(pool), Some(stats)) = (
            self.obj.take(),
            self.pool.upgrade(),
            self.stats.upgrade(),
        ) {
            let pool = pool.lock().unwrap();
            let mut stats = stats.lock().unwrap();
            stats.total_releases += 1;

            // Reset object to default state
            let reset_obj = T::default();

            if pool.len() < pool.capacity() {
                pool.push(reset_obj);
                stats.max_pool_size = stats.max_pool_size.max(pool.len());
            }
            // If pool is full, just drop the object
        }
    }
}

/// Memory profiler for tracking allocations in real-time
pub struct MemoryProfiler {
    /// Start time for the profiler
    start_time: Instant,
    /// Memory usage samples
    samples: Mutex<Vec<MemorySample>>,
    /// Sample interval
    sample_interval: Duration,
    /// Running flag
    running: std::sync::atomic::AtomicBool,
    /// Profiling thread handle
    profiler_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Duration,
    pub allocated_bytes: usize,
    pub peak_bytes: usize,
    pub active_objects: usize,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new(sample_interval: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            samples: Mutex::new(Vec::new()),
            sample_interval,
            running: std::sync::atomic::AtomicBool::new(false),
            profiler_handle: Mutex::new(None),
        }
    }

    /// Start profiling memory usage
    pub fn start(&self) {
        if self.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return; // Already running
        }

        let sample_interval = self.sample_interval;
        let samples = self.samples.clone();
        let running = self.running.clone();

        let handle = std::thread::spawn(move || {
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                // Sample memory usage (this is a simplified implementation)
                // In a real implementation, you'd integrate with system memory APIs
                let sample = MemorySample {
                    timestamp: Instant::now() - sample_interval,
                    allocated_bytes: 0, // Would need integration with allocator
                    peak_bytes: 0,
                    active_objects: 0,
                };

                samples.lock().unwrap().push(sample);
                std::thread::sleep(sample_interval);
            }
        });

        *self.profiler_handle.lock().unwrap() = Some(handle);
    }

    /// Stop profiling and return samples
    pub fn stop(&self) -> Vec<MemorySample> {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(handle) = self.profiler_handle.lock().unwrap().take() {
            let _ = handle.join();
        }

        std::mem::take(&mut *self.samples.lock().unwrap())
    }

    /// Get current samples
    pub fn get_samples(&self) -> Vec<MemorySample> {
        self.samples.lock().unwrap().clone()
    }

    /// Get profiling duration
    pub fn duration(&self) -> Duration {
        Instant::now() - self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_arena_allocator() {
        let mut allocator = ArenaAllocator::new(1024);

        // Allocate some objects
        let ptr1 = allocator.allocate(42i32);
        let ptr2 = allocator.allocate(3.14f64);
        let ptr3 = allocator.allocate_string("test");

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());

        unsafe {
            assert_eq!(*ptr1, 42);
            assert_eq!(*ptr2, 3.14);
            assert_eq!(*ptr3, "test");
        }

        // Get stats
        let stats = allocator.get_stats();
        assert_eq!(stats.total_allocations, 3);
        assert!(stats.arena_allocations > 0);

        // Reset and reuse
        allocator.reset();
        let stats_after_reset = allocator.get_stats();
        assert_eq!(stats_after_reset.current_memory_usage, 0);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("BTC");
        let id2 = interner.intern("ETH");
        let id3 = interner.intern("BTC"); // Should be cached

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);

        assert_eq!(interner.get(id1), Some("BTC"));
        assert_eq!(interner.get(id2), Some("ETH"));

        let stats = interner.get_stats();
        assert_eq!(stats.total_interns, 3);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.unique_strings, 2);
    }

    #[test]
    fn test_zero_copy_value() {
        let json_str = r#"{"price": "100.50", "size": "1.0", "side": "buy"}"#;
        let value = ZeroCopyValue::from_json_str(json_str).unwrap();

        if let ZeroCopyValue::Object(map) = value {
            assert_eq!(map.get("side").unwrap().as_str(), Some("buy"));
            assert_eq!(map.get("price").unwrap().as_str(), Some("100.50"));
        }
    }

    #[test]
    fn test_object_pool() {
        let pool: ObjectPool<Vec<i32>> = ObjectPool::new(5);

        {
            let mut obj = pool.get();
            obj.push(1);
            obj.push(2);
            // Object automatically returned to pool when dropped
        }

        let stats = pool.get_stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_releases, 1);
        assert_eq!(stats.pool_hits, 0); // First allocation is a miss
        assert_eq!(stats.pool_misses, 1);

        assert_eq!(pool.pool_size(), 1);
    }

    #[test]
    fn test_memory_profiler() {
        let profiler = MemoryProfiler::new(Duration::from_millis(100));

        profiler.start();
        std::thread::sleep(Duration::from_millis(250));
        let samples = profiler.stop();

        assert!(samples.len() > 0);
        assert!(profiler.duration() >= Duration::from_millis(250));
    }
}