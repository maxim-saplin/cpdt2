//! Disk Speed Test Library
//!
//! A cross-platform library for benchmarking disk performance with support for
//! sequential and random read/write operations, memory copy tests, and real-time
//! progress reporting.

pub mod core;
pub mod platform;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

// Re-export core types for library consumers
pub use core::{
    run_benchmark, BenchmarkConfig, BenchmarkError, BenchmarkResults, NoOpProgressCallback,
    ProgressCallback, ProgressEvent, ProgressReporter, RealTimeStatsTracker, StatisticsCollector,
    TestProgressCallback, TestResult,
};

pub use platform::{DeviceType, PlatformError, PlatformOps, StorageDevice};

/// Main library interface for running disk speed benchmarks
///
/// # Example
///
/// ```rust
/// use disk_speed_test::{BenchmarkConfig, run_benchmark};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = BenchmarkConfig {
///     target_path: PathBuf::from("."),
///     sequential_block_size: 4 * 1024 * 1024, // 4MB
///     random_block_size: 4 * 1024,            // 4KB
///     test_duration_seconds: 10,
///     disable_os_cache: true,
///     file_size_mb: 1024, // 1GB
/// };
///
/// let results = run_benchmark(config, None)?;
/// println!("Sequential Write: {:.2} MB/s", results.sequential_write.avg_speed_mbps);
/// # Ok(())
/// # }
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
