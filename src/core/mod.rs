//! Core benchmarking functionality
//! 
//! This module contains the main benchmarking logic, configuration structures,
//! and test implementations.

use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub mod config;
pub mod stats;
pub mod tests;

pub use config::BenchmarkConfig;
pub use stats::TestResult;

/// Results from a complete benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub sequential_write: TestResult,
    pub sequential_read: TestResult,
    pub random_write: TestResult,
    pub random_read: TestResult,
    pub memory_copy: TestResult,
}

/// Trait for receiving progress updates during benchmark execution
pub trait ProgressCallback: Send + Sync {
    /// Called when a test starts
    fn on_test_start(&self, test_name: &str);
    
    /// Called periodically during test execution with current speed
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64);
    
    /// Called when a test completes
    fn on_test_complete(&self, test_name: &str, result: &TestResult);
}

/// Main function to run all benchmark tests
pub fn run_benchmark(
    config: BenchmarkConfig,
    progress_callback: Option<Box<dyn ProgressCallback>>,
) -> crate::BenchmarkResult<BenchmarkResults> {
    // This is a stub implementation - will be implemented in later tasks
    todo!("run_benchmark implementation will be added in task 14")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_results_serialization() {
        let results = BenchmarkResults {
            sequential_write: TestResult::default(),
            sequential_read: TestResult::default(),
            random_write: TestResult::default(),
            random_read: TestResult::default(),
            memory_copy: TestResult::default(),
        };
        
        let json = serde_json::to_string(&results).unwrap();
        let deserialized: BenchmarkResults = serde_json::from_str(&json).unwrap();
        
        assert_eq!(results.sequential_write.avg_speed_mbps, deserialized.sequential_write.avg_speed_mbps);
    }
}