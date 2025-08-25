//! Test environment setup and configuration utilities

use super::TestDataManager;
use crate::core::config::BenchmarkConfig;
use crate::core::stats::TestResult;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestEnvironmentConfig {
    /// Minimum free space required for tests (in bytes)
    pub min_free_space: u64,
    /// Maximum test duration to prevent runaway tests
    pub max_test_duration: Duration,
    /// Whether to use small test files for faster testing
    pub use_small_files: bool,
    /// Whether to skip tests that require elevated permissions
    pub skip_privileged_tests: bool,
    /// Custom temporary directory path
    pub custom_temp_dir: Option<PathBuf>,
}

impl Default for TestEnvironmentConfig {
    fn default() -> Self {
        Self {
            min_free_space: 100 * 1024 * 1024, // 100MB
            max_test_duration: Duration::from_secs(30),
            use_small_files: true,
            skip_privileged_tests: false,
            custom_temp_dir: None,
        }
    }
}

/// Controlled test environment for benchmark testing
pub struct TestEnvironment {
    config: TestEnvironmentConfig,
    data_manager: TestDataManager,
    start_time: Instant,
    test_results: Arc<Mutex<Vec<(String, TestResult)>>>,
}

impl TestEnvironment {
    /// Create a new test environment
    pub fn new(config: TestEnvironmentConfig) -> Result<Self> {
        let data_manager = if let Some(ref custom_dir) = config.custom_temp_dir {
            // Create custom temp directory if it doesn't exist
            std::fs::create_dir_all(custom_dir)?;
            TestDataManager::new()?
        } else {
            TestDataManager::new()?
        };

        Ok(Self {
            config,
            data_manager,
            start_time: Instant::now(),
            test_results: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create a test environment with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(TestEnvironmentConfig::default())
    }

    /// Get the test data manager
    pub fn data_manager(&mut self) -> &mut TestDataManager {
        &mut self.data_manager
    }

    /// Check if environment meets minimum requirements
    pub fn check_requirements(&self) -> Result<()> {
        // Check available space
        let available_space = self.get_available_space()?;
        if available_space < self.config.min_free_space {
            anyhow::bail!(
                "Insufficient space: {} bytes available, {} bytes required",
                available_space,
                self.config.min_free_space
            );
        }

        // Check if we've exceeded maximum test duration
        if self.start_time.elapsed() > self.config.max_test_duration {
            anyhow::bail!("Test environment exceeded maximum duration");
        }

        Ok(())
    }

    /// Get available space in test directory
    pub fn get_available_space(&self) -> Result<u64> {
        self.data_manager.available_space()
    }

    /// Create a benchmark configuration suitable for testing
    pub fn create_test_benchmark_config(&self, target_path: Option<PathBuf>) -> BenchmarkConfig {
        let target = target_path.unwrap_or_else(|| self.data_manager.temp_dir_path().to_path_buf());

        BenchmarkConfig {
            target_path: target,
            sequential_block_size: if self.config.use_small_files {
                64 * 1024
            } else {
                4 * 1024 * 1024
            },
            random_block_size: if self.config.use_small_files {
                1024
            } else {
                4 * 1024
            },
            test_duration_seconds: if self.config.use_small_files { 1 } else { 5 },
            disable_os_cache: true,
            file_size_mb: if self.config.use_small_files { 1 } else { 100 },
        }
    }

    /// Record test result
    pub fn record_test_result(&self, test_name: String, result: TestResult) {
        if let Ok(mut results) = self.test_results.lock() {
            results.push((test_name, result));
        }
    }

    /// Get all recorded test results
    pub fn get_test_results(&self) -> Vec<(String, TestResult)> {
        self.test_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Check if test should be skipped based on environment
    pub fn should_skip_test(&self, test_name: &str) -> bool {
        if self.config.skip_privileged_tests {
            // Skip tests that might require elevated permissions
            matches!(
                test_name,
                "direct_io_test" | "device_enumeration_test" | "system_drive_test"
            )
        } else {
            false
        }
    }

    /// Create a timeout guard for test operations
    pub fn create_timeout_guard(&self, operation_name: &str) -> TimeoutGuard {
        TimeoutGuard::new(operation_name.to_string(), self.config.max_test_duration)
    }

    /// Get elapsed time since environment creation
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the environment configuration
    pub fn config(&self) -> &TestEnvironmentConfig {
        &self.config
    }

    /// Cleanup and prepare for next test
    pub fn reset_for_next_test(&mut self) -> Result<()> {
        // Clean up any existing test files
        for file_path in self.data_manager.test_files().to_vec() {
            self.data_manager.cleanup_file(&file_path)?;
        }

        // Check requirements again
        self.check_requirements()?;

        Ok(())
    }
}

/// Timeout guard to prevent runaway test operations
pub struct TimeoutGuard {
    operation_name: String,
    start_time: Instant,
    timeout: Duration,
}

impl TimeoutGuard {
    pub fn new(operation_name: String, timeout: Duration) -> Self {
        Self {
            operation_name,
            start_time: Instant::now(),
            timeout,
        }
    }

    /// Check if operation has timed out
    pub fn check_timeout(&self) -> Result<()> {
        if self.start_time.elapsed() > self.timeout {
            anyhow::bail!(
                "Operation '{}' timed out after {:?}",
                self.operation_name,
                self.timeout
            );
        }
        Ok(())
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> Duration {
        self.timeout.saturating_sub(self.start_time.elapsed())
    }
}

/// Test environment builder for fluent configuration
pub struct TestEnvironmentBuilder {
    config: TestEnvironmentConfig,
}

impl Default for TestEnvironmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvironmentBuilder {
    pub fn new() -> Self {
        Self {
            config: TestEnvironmentConfig::default(),
        }
    }

    pub fn min_free_space(mut self, bytes: u64) -> Self {
        self.config.min_free_space = bytes;
        self
    }

    pub fn max_test_duration(mut self, duration: Duration) -> Self {
        self.config.max_test_duration = duration;
        self
    }

    pub fn use_small_files(mut self, use_small: bool) -> Self {
        self.config.use_small_files = use_small;
        self
    }

    pub fn skip_privileged_tests(mut self, skip: bool) -> Self {
        self.config.skip_privileged_tests = skip;
        self
    }

    pub fn custom_temp_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.custom_temp_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<TestEnvironment> {
        TestEnvironment::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_environment_creation() {
        let env = TestEnvironment::with_defaults().unwrap();
        assert!(env.data_manager.temp_dir_path().exists());
    }

    #[test]
    fn test_environment_builder() {
        let env = TestEnvironmentBuilder::new()
            .min_free_space(50 * 1024 * 1024)
            .max_test_duration(Duration::from_secs(60))
            .use_small_files(true)
            .build()
            .unwrap();

        assert_eq!(env.config.min_free_space, 50 * 1024 * 1024);
        assert_eq!(env.config.max_test_duration, Duration::from_secs(60));
        assert!(env.config.use_small_files);
    }

    #[test]
    fn test_benchmark_config_creation() {
        let env = TestEnvironment::with_defaults().unwrap();
        let config = env.create_test_benchmark_config(None);

        assert_eq!(config.sequential_block_size, 64 * 1024); // Small file mode
        assert_eq!(config.random_block_size, 1024);
        assert_eq!(config.test_duration_seconds, 1);
        assert_eq!(config.file_size_mb, 1);
    }

    #[test]
    fn test_timeout_guard() {
        let guard = TimeoutGuard::new("test_op".to_string(), Duration::from_millis(100));

        // Should not timeout immediately
        assert!(guard.check_timeout().is_ok());

        // Wait and check timeout
        std::thread::sleep(Duration::from_millis(150));
        assert!(guard.check_timeout().is_err());
    }

    #[test]
    fn test_result_recording() {
        let env = TestEnvironment::with_defaults().unwrap();

        let test_result = TestResult {
            min_speed_mbps: 10.0,
            max_speed_mbps: 100.0,
            avg_speed_mbps: 50.0,
            test_duration: Duration::from_secs(5),
            sample_count: 100,
        };

        env.record_test_result("test_benchmark".to_string(), test_result.clone());

        let results = env.get_test_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test_benchmark");
        assert_eq!(results[0].1.avg_speed_mbps, 50.0);
    }
}
