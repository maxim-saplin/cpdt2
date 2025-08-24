//! Core benchmarking functionality

use std::path::PathBuf;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod config;
pub mod tests;
pub mod stats;
pub mod progress;

#[cfg(test)]
mod progress_integration_test;

pub use config::BenchmarkConfig;
pub use stats::{TestResult, StatisticsCollector, RealTimeStatsTracker};
pub use progress::{ProgressReporter, NoOpProgressCallback, TestProgressCallback, ProgressEvent};

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
/// 
/// This trait allows consumers to receive real-time updates during benchmark execution,
/// including when tests start, progress updates with current speed, and completion notifications.
/// 
/// # Thread Safety
/// 
/// Implementations must be `Send + Sync` as they may be called from different threads
/// during benchmark execution.
/// 
/// # Example
/// 
/// ```rust
/// use disk_speed_test::core::{ProgressCallback, TestResult};
/// 
/// struct MyProgressCallback;
/// 
/// impl ProgressCallback for MyProgressCallback {
///     fn on_test_start(&self, test_name: &str) {
///         println!("Starting test: {}", test_name);
///     }
///     
///     fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
///         println!("{}: {:.2} MB/s", test_name, current_speed_mbps);
///     }
///     
///     fn on_test_complete(&self, test_name: &str, result: &TestResult) {
///         println!("{} completed: avg {:.2} MB/s", test_name, result.avg_speed_mbps);
///     }
/// }
/// ```
pub trait ProgressCallback: Send + Sync {
    /// Called when a test starts
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the test that is starting (e.g., "Sequential Write")
    fn on_test_start(&self, test_name: &str);
    
    /// Called periodically during test execution with current speed
    /// 
    /// This method is called approximately every 100ms during test execution
    /// to provide real-time speed updates.
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the currently running test
    /// * `current_speed_mbps` - Current instantaneous speed in MB/s
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64);
    
    /// Called when a test completes
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the test that completed
    /// * `result` - Final test results including min, max, and average speeds
    fn on_test_complete(&self, test_name: &str, result: &TestResult);
}

/// Errors that can occur during benchmark execution
#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Platform error: {0}")]
    PlatformError(#[from] crate::platform::PlatformError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Insufficient space: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },
    
    #[error("Permission denied for path: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Test interrupted: {0}")]
    TestInterrupted(String),
}

/// Main function to run all benchmark tests
/// 
/// This function executes all five benchmark tests in sequence:
/// 1. Sequential Write
/// 2. Sequential Read  
/// 3. Random Write
/// 4. Random Read
/// 5. Memory Copy
/// 
/// # Arguments
/// 
/// * `config` - Configuration parameters for the benchmark
/// * `progress_callback` - Optional callback for progress updates
/// 
/// # Returns
/// 
/// Returns `BenchmarkResults` containing performance statistics for all tests
/// 
/// # Errors
/// 
/// Returns `BenchmarkError` if any test fails or configuration is invalid
pub fn run_benchmark(
    config: BenchmarkConfig,
    _progress_callback: Option<Box<dyn ProgressCallback>>
) -> Result<BenchmarkResults, BenchmarkError> {
    // Validate configuration
    config.validate()?;
    
    // TODO: Implement actual benchmark execution
    // This is a placeholder that will be implemented in later tasks
    
    Ok(BenchmarkResults {
        sequential_write: TestResult::default(),
        sequential_read: TestResult::default(),
        random_write: TestResult::default(),
        random_read: TestResult::default(),
        memory_copy: TestResult::default(),
    })
}