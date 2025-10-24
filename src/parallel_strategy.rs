//! Parallel strategy wrapper for existing repair strategies

use crate::error::Result;
use crate::traits::{ParallelRepairStrategy, RepairStrategy};
use std::sync::Arc;

/// Wrapper that makes any RepairStrategy parallel-safe
pub struct ParallelStrategyWrapper {
    strategy: Arc<dyn RepairStrategy + Send + Sync>,
}

impl ParallelStrategyWrapper {
    /// Create a new parallel strategy wrapper
    pub fn new(strategy: Arc<dyn RepairStrategy + Send + Sync>) -> Self {
        Self { strategy }
    }
}

impl RepairStrategy for ParallelStrategyWrapper {
    fn apply(&self, content: &str) -> Result<String> {
        self.strategy.apply(content)
    }
    
    fn priority(&self) -> u8 {
        self.strategy.priority()
    }
}

impl ParallelRepairStrategy for ParallelStrategyWrapper {
    fn apply_parallel(&self, content: &str) -> Result<String> {
        self.strategy.apply(content)
    }
}

/// Helper function to convert strategies to parallel-safe versions
pub fn make_parallel<T>(strategy: T) -> ParallelStrategyWrapper 
where 
    T: RepairStrategy + Send + Sync + 'static,
{
    ParallelStrategyWrapper::new(Arc::new(strategy))
}

/// Helper function to convert multiple strategies to parallel-safe versions
pub fn make_parallel_batch<T>(strategies: Vec<T>) -> Vec<ParallelStrategyWrapper>
where 
    T: RepairStrategy + Send + Sync + 'static,
{
    strategies
        .into_iter()
        .map(|strategy| make_parallel(strategy))
        .collect()
}
