//! Core benchmarking functionality

use std::path::PathBuf;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod config;
pub mod tests;
pub mod stats;

pub use config::BenchmarkConfig;
pub use stats::{TestResult, StatisticsCollector, RealTimeStatsTracker};

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