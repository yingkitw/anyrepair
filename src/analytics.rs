//! Analytics module for tracking repair success metrics and performance

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Repair success metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairMetrics {
    /// Total number of repairs attempted
    pub total_repairs: u64,
    /// Number of successful repairs
    pub successful_repairs: u64,
    /// Number of failed repairs
    pub failed_repairs: u64,
    /// Average repair time in milliseconds
    pub avg_repair_time_ms: f64,
    /// Total time spent on repairs in milliseconds
    pub total_repair_time_ms: u64,
    /// Metrics by format
    pub format_metrics: HashMap<String, FormatMetrics>,
}

/// Metrics for a specific format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatMetrics {
    /// Format name
    pub format: String,
    /// Number of repairs for this format
    pub repair_count: u64,
    /// Number of successful repairs for this format
    pub success_count: u64,
    /// Average repair time for this format
    pub avg_time_ms: f64,
    /// Confidence score average
    pub avg_confidence: f64,
}

/// Analytics tracker for repair operations
pub struct AnalyticsTracker {
    metrics: Arc<Mutex<RepairMetrics>>,
}

impl AnalyticsTracker {
    /// Create a new analytics tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(RepairMetrics {
                total_repairs: 0,
                successful_repairs: 0,
                failed_repairs: 0,
                avg_repair_time_ms: 0.0,
                total_repair_time_ms: 0,
                format_metrics: HashMap::new(),
            })),
        }
    }

    /// Record a repair operation
    pub fn record_repair(
        &self,
        format: &str,
        success: bool,
        duration: Duration,
        confidence: f64,
    ) {
        let mut metrics = self.metrics.lock().unwrap();
        let duration_ms = duration.as_millis() as u64;

        metrics.total_repairs += 1;
        if success {
            metrics.successful_repairs += 1;
        } else {
            metrics.failed_repairs += 1;
        }

        metrics.total_repair_time_ms += duration_ms;
        metrics.avg_repair_time_ms =
            metrics.total_repair_time_ms as f64 / metrics.total_repairs as f64;

        // Update format-specific metrics
        let format_metric = metrics
            .format_metrics
            .entry(format.to_string())
            .or_insert(FormatMetrics {
                format: format.to_string(),
                repair_count: 0,
                success_count: 0,
                avg_time_ms: 0.0,
                avg_confidence: 0.0,
            });

        format_metric.repair_count += 1;
        if success {
            format_metric.success_count += 1;
        }

        format_metric.avg_time_ms =
            (format_metric.avg_time_ms * (format_metric.repair_count - 1) as f64 + duration_ms as f64)
                / format_metric.repair_count as f64;

        format_metric.avg_confidence =
            (format_metric.avg_confidence * (format_metric.repair_count - 1) as f64 + confidence)
                / format_metric.repair_count as f64;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> RepairMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Get success rate as percentage
    pub fn get_success_rate(&self) -> f64 {
        let metrics = self.metrics.lock().unwrap();
        if metrics.total_repairs == 0 {
            0.0
        } else {
            (metrics.successful_repairs as f64 / metrics.total_repairs as f64) * 100.0
        }
    }

    /// Get metrics for a specific format
    pub fn get_format_metrics(&self, format: &str) -> Option<FormatMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.format_metrics.get(format).cloned()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = RepairMetrics {
            total_repairs: 0,
            successful_repairs: 0,
            failed_repairs: 0,
            avg_repair_time_ms: 0.0,
            total_repair_time_ms: 0,
            format_metrics: HashMap::new(),
        };
    }
}

impl Default for AnalyticsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitor for tracking operation timings
pub struct PerformanceMonitor {
    start_time: Instant,
    operation_name: String,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(operation_name: &str) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Get operation name
    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_tracker_creation() {
        let tracker = AnalyticsTracker::new();
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.total_repairs, 0);
        assert_eq!(metrics.successful_repairs, 0);
        assert_eq!(metrics.failed_repairs, 0);
    }

    #[test]
    fn test_record_repair() {
        let tracker = AnalyticsTracker::new();
        tracker.record_repair("json", true, Duration::from_millis(10), 0.95);

        let metrics = tracker.get_metrics();
        assert_eq!(metrics.total_repairs, 1);
        assert_eq!(metrics.successful_repairs, 1);
        assert_eq!(metrics.failed_repairs, 0);
        assert!(metrics.avg_repair_time_ms > 0.0);
    }

    #[test]
    fn test_success_rate() {
        let tracker = AnalyticsTracker::new();
        tracker.record_repair("json", true, Duration::from_millis(10), 0.95);
        tracker.record_repair("json", false, Duration::from_millis(5), 0.50);

        let success_rate = tracker.get_success_rate();
        assert_eq!(success_rate, 50.0);
    }

    #[test]
    fn test_format_metrics() {
        let tracker = AnalyticsTracker::new();
        tracker.record_repair("json", true, Duration::from_millis(10), 0.95);
        tracker.record_repair("yaml", true, Duration::from_millis(15), 0.85);

        let json_metrics = tracker.get_format_metrics("json").unwrap();
        assert_eq!(json_metrics.repair_count, 1);
        assert_eq!(json_metrics.success_count, 1);

        let yaml_metrics = tracker.get_format_metrics("yaml").unwrap();
        assert_eq!(yaml_metrics.repair_count, 1);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = monitor.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
