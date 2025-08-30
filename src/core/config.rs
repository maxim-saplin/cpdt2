//! Benchmark configuration structures and validation

use crate::core::BenchmarkError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration parameters for benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Target path for test files
    pub target_path: PathBuf,

    /// Block size for sequential operations (default: 4MB)
    pub sequential_block_size: usize,

    /// Block size for random operations (default: 4KB)
    pub random_block_size: usize,

    /// Duration to run each test in seconds (default: 10)
    pub test_duration_seconds: u64,

    /// Whether to disable OS caching (default: true)
    pub disable_os_cache: bool,

    /// Whether to disable direct I/O operations (default: false)
    /// When true, uses buffered I/O which may be slower but more compatible
    pub disable_direct_io: bool,

    /// Size of test file in MB (default: 1024)
    pub file_size_mb: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            target_path: PathBuf::from("."),
            sequential_block_size: 4 * 1024 * 1024, // 4MB
            random_block_size: 4 * 1024,            // 4KB
            test_duration_seconds: 10,
            disable_os_cache: true,
            disable_direct_io: false, // Enable direct I/O by default for performance
            file_size_mb: 1024,       // 1GB
        }
    }
}

impl BenchmarkConfig {
    /// Create a new configuration with the specified target path
    pub fn new(target_path: PathBuf) -> Self {
        Self {
            target_path,
            ..Default::default()
        }
    }

    /// Validate the configuration parameters
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        // Validate block sizes
        if self.sequential_block_size == 0 {
            return Err(BenchmarkError::ConfigurationError(
                "Sequential block size must be greater than 0".to_string(),
            ));
        }

        if self.random_block_size == 0 {
            return Err(BenchmarkError::ConfigurationError(
                "Random block size must be greater than 0".to_string(),
            ));
        }

        // Validate test duration
        if self.test_duration_seconds == 0 {
            return Err(BenchmarkError::ConfigurationError(
                "Test duration must be greater than 0".to_string(),
            ));
        }

        // Validate file size
        if self.file_size_mb == 0 {
            return Err(BenchmarkError::ConfigurationError(
                "File size must be greater than 0".to_string(),
            ));
        }

        // Validate target path exists
        if !self.target_path.exists() {
            return Err(BenchmarkError::ConfigurationError(format!(
                "Target path does not exist: {}",
                self.target_path.display()
            )));
        }

        Ok(())
    }

    /// Get the test file size in bytes
    pub fn file_size_bytes(&self) -> u64 {
        (self.file_size_mb as u64) * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.sequential_block_size, 4 * 1024 * 1024);
        assert_eq!(config.random_block_size, 4 * 1024);
        assert_eq!(config.test_duration_seconds, 10);
        assert!(config.disable_os_cache);
        assert_eq!(config.file_size_mb, 1024);
    }

    #[test]
    fn test_config_validation() {
        let mut config = BenchmarkConfig::new(env::current_dir().unwrap());

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid block sizes should fail
        config.sequential_block_size = 0;
        assert!(config.validate().is_err());

        config.sequential_block_size = 4 * 1024 * 1024;
        config.random_block_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_file_size_bytes() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.file_size_bytes(), 1024 * 1024 * 1024); // 1GB
    }
}
