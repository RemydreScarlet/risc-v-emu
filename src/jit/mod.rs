pub mod cache;
pub mod codegen;
pub mod context;

/// JIT engine for RISC-V emulation
///
/// This engine provides optional Just-In-Time compilation of RISC-V code
/// to JavaScript functions, improving performance for hot code paths.
pub struct JitEngine {
    cache: cache::JitCache,
    codegen: codegen::CodeGenerator,
    enabled: bool,
    hot_threshold: u32,
}

impl JitEngine {
    /// Create a new JIT engine
    pub fn new(hot_threshold: u32) -> Self {
        JitEngine {
            cache: cache::JitCache::new(),
            codegen: codegen::CodeGenerator::new(),
            enabled: false,
            hot_threshold,
        }
    }

    /// Enable JIT compilation
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable JIT compilation
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if JIT is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the hot threshold for compilation
    pub fn hot_threshold(&self) -> u32 {
        self.hot_threshold
    }

    /// Set the hot threshold for compilation
    pub fn set_hot_threshold(&mut self, threshold: u32) {
        self.hot_threshold = threshold;
    }

    /// Check if a PC address is hot (should be compiled)
    pub fn is_hot(&self, pc: u64) -> bool {
        self.cache.is_hot(pc, self.hot_threshold)
    }

    /// Get a mutable reference to the cache
    pub fn cache_mut(&mut self) -> &mut cache::JitCache {
        &mut self.cache
    }

    /// Get a reference to the cache
    pub fn cache(&self) -> &cache::JitCache {
        &self.cache
    }

    /// Get a mutable reference to the code generator
    pub fn codegen_mut(&mut self) -> &mut codegen::CodeGenerator {
        &mut self.codegen
    }

    /// Clear the JIT cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get JIT statistics
    pub fn get_stats(&self) -> cache::JitStats {
        self.cache.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_engine_creation() {
        let engine = JitEngine::new(100);
        assert!(!engine.is_enabled());
        assert_eq!(engine.hot_threshold(), 100);
    }

    #[test]
    fn test_enable_disable() {
        let mut engine = JitEngine::new(100);
        assert!(!engine.is_enabled());
        
        engine.enable();
        assert!(engine.is_enabled());
        
        engine.disable();
        assert!(!engine.is_enabled());
    }

    #[test]
    fn test_hot_threshold() {
        let mut engine = JitEngine::new(100);
        assert_eq!(engine.hot_threshold(), 100);
        
        engine.set_hot_threshold(50);
        assert_eq!(engine.hot_threshold(), 50);
    }
}
