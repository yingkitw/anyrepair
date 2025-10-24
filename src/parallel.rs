//! Parallel strategy application for improved performance

use crate::error::Result;
use crate::traits::RepairStrategy;
use rayon::prelude::*;
use std::sync::Arc;

/// Parallel strategy applicator that can run multiple strategies concurrently
pub struct ParallelStrategyApplicator {
    strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>>,
}

impl ParallelStrategyApplicator {
    /// Create a new parallel strategy applicator
    pub fn new(strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>>) -> Self {
        Self { strategies }
    }
    
    /// Apply all strategies in parallel and return the best result
    pub fn apply_parallel(&self, content: &str) -> Result<String> {
        if self.strategies.is_empty() {
            return Ok(content.to_string());
        }
        
        // Apply all strategies in parallel
        let results: Vec<Result<String>> = self.strategies
            .par_iter()
            .map(|strategy| strategy.apply(content))
            .collect();
        
        // Find the best result (first successful one, or return original if all fail)
        for result in results {
            if let Ok(repaired) = result {
                return Ok(repaired);
            }
        }
        
        // If all strategies failed, return original content
        Ok(content.to_string())
    }
    
    /// Apply strategies in parallel and return all successful results
    pub fn apply_all_parallel(&self, content: &str) -> Vec<String> {
        if self.strategies.is_empty() {
            return vec![content.to_string()];
        }
        
        self.strategies
            .par_iter()
            .filter_map(|strategy| strategy.apply(content).ok())
            .collect()
    }
    
    /// Apply strategies in parallel and return the result with highest confidence
    pub fn apply_with_confidence(&self, content: &str) -> Result<String> {
        if self.strategies.is_empty() {
            return Ok(content.to_string());
        }
        
        // Apply all strategies in parallel
        let results: Vec<(Result<String>, u8)> = self.strategies
            .par_iter()
            .map(|strategy| (strategy.apply(content), strategy.priority()))
            .collect();
        
        // Find the result with highest priority among successful ones
        let mut best_result = content.to_string();
        let mut best_priority = 0u8;
        
        for (result, priority) in results {
            if let Ok(repaired) = result {
                if priority > best_priority {
                    best_result = repaired;
                    best_priority = priority;
                }
            }
        }
        
        Ok(best_result)
    }
}

/// Parallel batch processor for multiple content pieces
pub struct ParallelBatchProcessor {
    applicator: ParallelStrategyApplicator,
}

impl ParallelBatchProcessor {
    /// Create a new parallel batch processor
    pub fn new(strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>>) -> Self {
        Self {
            applicator: ParallelStrategyApplicator::new(strategies),
        }
    }
    
    /// Process multiple content pieces in parallel
    pub fn process_batch(&self, contents: &[String]) -> Vec<Result<String>> {
        contents
            .par_iter()
            .map(|content| self.applicator.apply_parallel(content))
            .collect()
    }
    
    /// Process multiple content pieces in parallel with confidence scoring
    pub fn process_batch_with_confidence(&self, contents: &[String]) -> Vec<Result<String>> {
        contents
            .par_iter()
            .map(|content| self.applicator.apply_with_confidence(content))
            .collect()
    }
}

/// Performance metrics for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelMetrics {
    pub total_strategies: usize,
    pub successful_applications: usize,
    pub failed_applications: usize,
    pub average_processing_time_ms: f64,
    pub parallel_efficiency: f64,
}

impl ParallelMetrics {
    /// Calculate parallel efficiency (0.0 to 1.0)
    pub fn calculate_efficiency(&self) -> f64 {
        if self.total_strategies == 0 {
            return 0.0;
        }
        
        let success_rate = self.successful_applications as f64 / self.total_strategies as f64;
        let parallel_speedup = self.total_strategies as f64; // Theoretical maximum speedup
        let actual_speedup = if self.average_processing_time_ms > 0.0 {
            parallel_speedup / self.average_processing_time_ms
        } else {
            0.0
        };
        
        success_rate * (actual_speedup / parallel_speedup).min(1.0)
    }
}

/// Parallel strategy executor with metrics collection
pub struct ParallelExecutor {
    applicator: ParallelStrategyApplicator,
    metrics: ParallelMetrics,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>>) -> Self {
        let total_strategies = strategies.len();
        Self {
            applicator: ParallelStrategyApplicator::new(strategies),
            metrics: ParallelMetrics {
                total_strategies,
                successful_applications: 0,
                failed_applications: 0,
                average_processing_time_ms: 0.0,
                parallel_efficiency: 0.0,
            },
        }
    }
    
    /// Execute strategies with metrics collection
    pub fn execute_with_metrics(&mut self, content: &str) -> Result<String> {
        let start = std::time::Instant::now();
        
        let result = self.applicator.apply_parallel(content);
        
        let duration = start.elapsed();
        self.metrics.average_processing_time_ms = duration.as_millis() as f64;
        
        match &result {
            Ok(_) => self.metrics.successful_applications += 1,
            Err(_) => self.metrics.failed_applications += 1,
        }
        
        self.metrics.parallel_efficiency = self.metrics.calculate_efficiency();
        
        result
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> &ParallelMetrics {
        &self.metrics
    }
    
    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ParallelMetrics {
            total_strategies: self.metrics.total_strategies,
            successful_applications: 0,
            failed_applications: 0,
            average_processing_time_ms: 0.0,
            parallel_efficiency: 0.0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::RepairStrategy;
    use std::sync::Arc;

    // Mock strategy for testing
    struct MockStrategy {
        name: String,
        priority: u8,
        should_succeed: bool,
    }

    impl RepairStrategy for MockStrategy {
        fn apply(&self, content: &str) -> Result<String> {
            if self.should_succeed {
                Ok(format!("{}: {}", self.name, content))
            } else {
                Err(crate::error::RepairError::generic("Mock error"))
            }
        }
        
        fn priority(&self) -> u8 {
            self.priority
        }
    }

    #[test]
    fn test_parallel_strategy_application() {
        let strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>> = vec![
            Arc::new(MockStrategy {
                name: "Strategy1".to_string(),
                priority: 5,
                should_succeed: true,
            }),
            Arc::new(MockStrategy {
                name: "Strategy2".to_string(),
                priority: 3,
                should_succeed: true,
            }),
        ];
        
        let applicator = ParallelStrategyApplicator::new(strategies);
        let result = applicator.apply_parallel("test content").unwrap();
        
        // Should return the first successful result
        assert!(result.contains("Strategy1") || result.contains("Strategy2"));
    }
    
    #[test]
    fn test_parallel_batch_processing() {
        let strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>> = vec![
            Arc::new(MockStrategy {
                name: "Strategy1".to_string(),
                priority: 5,
                should_succeed: true,
            }),
        ];
        
        let processor = ParallelBatchProcessor::new(strategies);
        let contents = vec![
            "content1".to_string(),
            "content2".to_string(),
        ];
        
        let results = processor.process_batch(&contents);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
    
    #[test]
    fn test_parallel_executor_metrics() {
        let strategies: Vec<Arc<dyn RepairStrategy + Send + Sync>> = vec![
            Arc::new(MockStrategy {
                name: "Strategy1".to_string(),
                priority: 5,
                should_succeed: true,
            }),
        ];
        
        let mut executor = ParallelExecutor::new(strategies);
        let _result = executor.execute_with_metrics("test content");
        
        let metrics = executor.get_metrics();
        assert_eq!(metrics.total_strategies, 1);
        assert_eq!(metrics.successful_applications, 1);
        assert_eq!(metrics.failed_applications, 0);
    }
    
    #[test]
    fn test_parallel_metrics_efficiency() {
        let metrics = ParallelMetrics {
            total_strategies: 4,
            successful_applications: 3,
            failed_applications: 1,
            average_processing_time_ms: 10.0,
            parallel_efficiency: 0.0,
        };
        
        let efficiency = metrics.calculate_efficiency();
        assert!(efficiency > 0.0);
        assert!(efficiency <= 1.0);
    }
}
