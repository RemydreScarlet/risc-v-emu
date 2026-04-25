use std::collections::BTreeMap;

/// JIT statistics
#[derive(Debug, Clone, Copy)]
pub struct JitStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub compiled_blocks: u32,
    pub total_instructions: u64,
    pub jit_instructions: u64,
    pub interpreter_instructions: u64,
}

impl Default for JitStats {
    fn default() -> Self {
        JitStats {
            cache_hits: 0,
            cache_misses: 0,
            compiled_blocks: 0,
            total_instructions: 0,
            jit_instructions: 0,
            interpreter_instructions: 0,
        }
    }
}

/// A compiled JIT block
#[derive(Clone)]
pub struct CompiledBlock {
    /// The generated JavaScript code as a string
    pub code: String,
    /// The PC address this block starts at
    pub pc: u64,
    /// Number of times this block has been executed
    pub execution_count: u32,
}

/// JIT cache for managing compiled blocks
pub struct JitCache {
    /// Map from PC address to compiled blocks
    compiled_blocks: BTreeMap<u64, CompiledBlock>,
    /// Map from PC address to execution counts (for hot path detection)
    execution_counts: BTreeMap<u64, u32>,
    /// Statistics
    stats: JitStats,
}

impl JitCache {
    /// Create a new JIT cache
    pub fn new() -> Self {
        JitCache {
            compiled_blocks: BTreeMap::new(),
            execution_counts: BTreeMap::new(),
            stats: JitStats::default(),
        }
    }

    /// Get a compiled block for the given PC address
    pub fn get(&self, pc: u64) -> Option<&CompiledBlock> {
        self.compiled_blocks.get(&pc)
    }

    /// Get a mutable reference to a compiled block
    pub fn get_mut(&mut self, pc: u64) -> Option<&mut CompiledBlock> {
        self.compiled_blocks.get_mut(&pc)
    }

    /// Insert a compiled block into the cache
    pub fn insert(&mut self, block: CompiledBlock) {
        let pc = block.pc;
        self.compiled_blocks.insert(pc, block);
        self.stats.compiled_blocks += 1;
    }

    /// Check if a PC address has a compiled block
    pub fn contains(&self, pc: u64) -> bool {
        self.compiled_blocks.contains_key(&pc)
    }

    /// Increment the execution count for a PC address
    pub fn increment_count(&mut self, pc: u64) {
        *self.execution_counts.entry(pc).or_insert(0) += 1;
    }

    /// Get the execution count for a PC address
    pub fn get_count(&self, pc: u64) -> u32 {
        self.execution_counts.get(&pc).copied().unwrap_or(0)
    }

    /// Check if a PC address is hot (should be compiled)
    pub fn is_hot(&self, pc: u64, threshold: u32) -> bool {
        self.get_count(pc) >= threshold
    }

    /// Invalidate a compiled block (for self-modifying code)
    pub fn invalidate(&mut self, pc: u64) {
        self.compiled_blocks.remove(&pc);
        self.execution_counts.remove(&pc);
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.compiled_blocks.clear();
        self.execution_counts.clear();
        self.stats = JitStats::default();
    }

    /// Get the number of compiled blocks in the cache
    pub fn len(&self) -> usize {
        self.compiled_blocks.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.compiled_blocks.is_empty()
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.stats.cache_hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.stats.cache_misses += 1;
    }

    /// Record JIT instruction execution
    pub fn record_jit_instruction(&mut self) {
        self.stats.jit_instructions += 1;
    }

    /// Record interpreter instruction execution
    pub fn record_interpreter_instruction(&mut self) {
        self.stats.interpreter_instructions += 1;
    }

    /// Record total instruction execution
    pub fn record_instruction(&mut self) {
        self.stats.total_instructions += 1;
    }

    /// Get JIT statistics
    pub fn get_stats(&self) -> JitStats {
        self.stats.clone()
    }

    /// Get all PC addresses in the cache
    pub fn keys(&self) -> impl Iterator<Item = &u64> {
        self.compiled_blocks.keys()
    }

    /// Get an iterator over all compiled blocks
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &CompiledBlock)> {
        self.compiled_blocks.iter()
    }
}

impl Default for JitCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = JitCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = JitCache::new();
        let block = CompiledBlock {
            code: "test code".to_string(),
            pc: 0x1000,
            execution_count: 0,
        };

        cache.insert(block.clone());
        assert_eq!(cache.len(), 1);
        assert!(cache.contains(0x1000));

        let retrieved = cache.get(0x1000);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().code, "test code");
    }

    #[test]
    fn test_execution_counting() {
        let mut cache = JitCache::new();

        assert_eq!(cache.get_count(0x1000), 0);

        cache.increment_count(0x1000);
        cache.increment_count(0x1000);
        cache.increment_count(0x1000);

        assert_eq!(cache.get_count(0x1000), 3);
    }

    #[test]
    fn test_hot_detection() {
        let mut cache = JitCache::new();

        for _ in 0..100 {
            cache.increment_count(0x1000);
        }

        assert!(cache.is_hot(0x1000, 100));
        assert!(!cache.is_hot(0x1000, 101));
        assert!(!cache.is_hot(0x2000, 100));
    }

    #[test]
    fn test_invalidation() {
        let mut cache = JitCache::new();
        let block = CompiledBlock {
            code: "test code".to_string(),
            pc: 0x1000,
            execution_count: 0,
        };

        cache.insert(block);
        cache.increment_count(0x1000);

        assert!(cache.contains(0x1000));
        assert_eq!(cache.get_count(0x1000), 1);

        cache.invalidate(0x1000);

        assert!(!cache.contains(0x1000));
        assert_eq!(cache.get_count(0x1000), 0);
    }

    #[test]
    fn test_clear() {
        let mut cache = JitCache::new();

        let block = CompiledBlock {
            code: "test code".to_string(),
            pc: 0x1000,
            execution_count: 0,
        };

        cache.insert(block);
        cache.increment_count(0x1000);

        assert_eq!(cache.len(), 1);

        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut cache = JitCache::new();

        cache.record_hit();
        cache.record_hit();
        cache.record_miss();
        cache.record_jit_instruction();
        cache.record_jit_instruction();
        cache.record_jit_instruction();
        cache.record_interpreter_instruction();
        cache.record_instruction();

        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.jit_instructions, 3);
        assert_eq!(stats.interpreter_instructions, 1);
        assert_eq!(stats.total_instructions, 1);
    }
}
