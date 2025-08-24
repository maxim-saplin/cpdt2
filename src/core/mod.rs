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
pub use tests::{run_sequential_write_test, run_sequential_read_test, run_random_write_test, run_random_read_test, run_memory_copy_test};

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
    progress_callback: Option<Box<dyn ProgressCallback>>
) -> Result<BenchmarkResults, BenchmarkError> {
    use std::fs;
    use std::time::SystemTime;
    use crate::platform;
    
    // Validate configuration
    config.validate()?;
    
    // Check available space before starting tests
    let _required_space = config.file_size_bytes();
    if let Ok(_metadata) = fs::metadata(&config.target_path) {
        // For directories, we can't directly check available space via metadata
        // We'll rely on the platform-specific device enumeration or handle errors during file creation
    }
    
    // Generate unique test file name to avoid conflicts
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let test_file_name = format!("disk_speed_test_{}.tmp", timestamp);
    let test_file_path = config.target_path.join(&test_file_name);
    
    // Convert boxed callback to reference for easier handling
    let callback_ref = progress_callback.as_ref().map(|cb| cb.as_ref());
    
    // Initialize results with default values
    let mut results = BenchmarkResults {
        sequential_write: TestResult::default(),
        sequential_read: TestResult::default(),
        random_write: TestResult::default(),
        random_read: TestResult::default(),
        memory_copy: TestResult::default(),
    };
    
    // Track which tests have been completed for cleanup purposes
    let mut test_file_created = false;
    
    // Execute tests in sequence with proper error handling and cleanup
    let benchmark_result = (|| -> Result<BenchmarkResults, BenchmarkError> {
        // Test 1: Sequential Write
        // This test creates the test file, so we track its creation
        match tests::run_sequential_write_test(&config, &test_file_path, callback_ref) {
            Ok(result) => {
                results.sequential_write = result;
                test_file_created = true;
            }
            Err(e) => {
                // If sequential write fails, we can't continue with read tests
                return Err(e);
            }
        }
        
        // Test 2: Sequential Read
        // Requires the test file created by sequential write
        match tests::run_sequential_read_test(&config, &test_file_path, callback_ref) {
            Ok(result) => {
                results.sequential_read = result;
            }
            Err(e) => {
                // Log error but continue with other tests that don't require the file
                eprintln!("Warning: Sequential read test failed: {}", e);
                // Set default result to indicate test failure
                results.sequential_read = TestResult::default();
            }
        }
        
        // Test 3: Random Write
        // Uses the existing test file
        match tests::run_random_write_test(&config, &test_file_path, callback_ref) {
            Ok(result) => {
                results.random_write = result;
            }
            Err(e) => {
                // Log error but continue with other tests
                eprintln!("Warning: Random write test failed: {}", e);
                results.random_write = TestResult::default();
            }
        }
        
        // Test 4: Random Read
        // Uses the existing test file
        match tests::run_random_read_test(&config, &test_file_path, callback_ref) {
            Ok(result) => {
                results.random_read = result;
            }
            Err(e) => {
                // Log error but continue with memory test
                eprintln!("Warning: Random read test failed: {}", e);
                results.random_read = TestResult::default();
            }
        }
        
        // Test 5: Memory Copy
        // Independent of disk file, so should always work
        match tests::run_memory_copy_test(&config, callback_ref) {
            Ok(result) => {
                results.memory_copy = result;
            }
            Err(e) => {
                // Log error but don't fail the entire benchmark
                eprintln!("Warning: Memory copy test failed: {}", e);
                results.memory_copy = TestResult::default();
            }
        }
        
        Ok(results)
    })();
    
    // Cleanup: Always attempt to remove the test file if it was created
    if test_file_created {
        if let Err(cleanup_error) = fs::remove_file(&test_file_path) {
            // Log cleanup error but don't fail the benchmark
            eprintln!("Warning: Failed to cleanup test file {}: {}", 
                     test_file_path.display(), cleanup_error);
        }
    }
    
    // Additional cleanup: Sync filesystem if cache bypassing was enabled
    if config.disable_os_cache {
        if let Err(sync_error) = platform::sync_file_system(&config.target_path) {
            // Log sync error but don't fail the benchmark
            eprintln!("Warning: Failed to sync filesystem: {}", sync_error);
        }
    }
    
    benchmark_result
}