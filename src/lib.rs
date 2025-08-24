//! Disk Speed Test Library
//! 
//! A cross-platform library for benchmarking disk and memory performance.
//! Supports Windows, macOS, Linux, Android, and iOS platforms.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

pub mod core;
pub mod platform;

// Re-export main types for easier consumption
pub use core::{
    BenchmarkConfig, BenchmarkResults, TestResult, ProgressCallback,
    run_benchmark
};
pub use platform::{PlatformOps, StorageDevice, DeviceType};

/// Main error type for benchmark operations
#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Platform error: {0}")]
    PlatformError(#[from] platform::PlatformError),
    
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

/// Result type for benchmark operations
pub type BenchmarkResult<T> = Result<T, BenchmarkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify that main types are properly exported
        let _config = BenchmarkConfig::default();
        assert!(true); // Basic compilation test
    }
}