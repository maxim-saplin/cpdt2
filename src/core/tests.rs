//! Test implementations for different benchmark types

use std::path::Path;
use crate::core::{BenchmarkConfig, TestResult, ProgressCallback, BenchmarkError};

/// Sequential write test implementation
pub fn run_sequential_write_test(
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use std::io::Write;
    use std::time::{Duration, Instant};
    use crate::core::RealTimeStatsTracker;
    use crate::platform;

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Sequential Write");
    }

    // We'll handle progress reporting directly since we have a reference

    // Create the test file with direct I/O
    let mut file = platform::create_direct_io_file(test_file_path, config.file_size_bytes())?;

    // Create buffer with the configured block size
    let block_size = config.sequential_block_size;
    let mut buffer = vec![0u8; block_size];
    
    // Fill buffer with test pattern (alternating bytes to avoid compression)
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_seconds);
    
    let mut bytes_written = 0u64;
    let file_size = config.file_size_bytes();

    // Main write loop - continue until test duration elapsed or file is full
    while test_start.elapsed() < test_duration && bytes_written < file_size {
        let _write_start = Instant::now();
        
        // Calculate how much to write this iteration
        let remaining_file_space = file_size - bytes_written;
        let bytes_to_write = std::cmp::min(block_size as u64, remaining_file_space) as usize;
        
        // Write the block
        let bytes_written_this_iteration = file.write(&buffer[..bytes_to_write])?;
        bytes_written += bytes_written_this_iteration as u64;
        
        // Update statistics and report progress
        if let Some(current_speed) = stats_tracker.update_progress(bytes_written) {
            if let Some(callback) = progress_callback {
                callback.on_progress("Sequential Write", current_speed);
            }
        }
        
        // If we couldn't write the full block, we're likely at EOF
        if bytes_written_this_iteration < bytes_to_write {
            break;
        }
        
        // If we've filled the file, seek back to beginning to continue writing
        if bytes_written >= file_size {
            use std::io::Seek;
            file.seek(std::io::SeekFrom::Start(0))?;
            bytes_written = 0; // Reset counter but keep stats tracking total throughput
        }
    }

    // Ensure data is written to disk
    file.flush()?;
    if config.disable_os_cache {
        platform::sync_file_system(test_file_path)?;
    }

    // Finalize statistics
    let result = stats_tracker.finalize();
    
    // Report test completion
    if let Some(callback) = progress_callback {
        callback.on_test_complete("Sequential Write", &result);
    }

    Ok(result)
}

/// Sequential read test implementation
pub fn run_sequential_read_test(
    _config: &BenchmarkConfig,
    _test_file_path: &Path,
    _progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    // TODO: Implement sequential read test in task 10
    // This is a placeholder that will be implemented later
    Ok(TestResult::default())
}

/// Random write test implementation
pub fn run_random_write_test(
    _config: &BenchmarkConfig,
    _test_file_path: &Path,
    _progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    // TODO: Implement random write test in task 11
    // This is a placeholder that will be implemented later
    Ok(TestResult::default())
}

/// Random read test implementation
pub fn run_random_read_test(
    _config: &BenchmarkConfig,
    _test_file_path: &Path,
    _progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    // TODO: Implement random read test in task 12
    // This is a placeholder that will be implemented later
    Ok(TestResult::default())
}

/// Memory copy test implementation
pub fn run_memory_copy_test(
    _config: &BenchmarkConfig,
    _progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    // TODO: Implement memory copy test in task 13
    // This is a placeholder that will be implemented later
    Ok(TestResult::default())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TestProgressCallback;
    use std::fs;
    use std::env;

    fn create_test_config() -> BenchmarkConfig {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        config.file_size_mb = 1; // Small file for testing
        config.test_duration_seconds = 1; // Short duration for testing
        config.sequential_block_size = 64 * 1024; // 64KB blocks for testing
        config
    }

    fn create_test_file_path() -> std::path::PathBuf {
        let temp_dir = env::temp_dir();
        let thread_id = std::thread::current().id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.join(format!("disk_speed_test_sequential_write_{}_{}.tmp", 
                             format!("{:?}", thread_id).replace("ThreadId(", "").replace(")", ""),
                             timestamp))
    }

    fn cleanup_test_file(path: &Path) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_sequential_write_basic_functionality() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Run the test
        let result = run_sequential_write_test(&config, &test_file_path, None);
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(result.is_ok(), "Sequential write test should complete successfully");
        
        let test_result = result.unwrap();
        
        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(test_result.sample_count > 0, "Should have collected at least one sample");
        
        // Speed values should be non-negative
        assert!(test_result.min_speed_mbps >= 0.0);
        assert!(test_result.max_speed_mbps >= 0.0);
        assert!(test_result.avg_speed_mbps >= 0.0);
        
        // Max should be >= min
        assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
    }

    #[test]
    fn test_sequential_write_with_progress_callback() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Run the test with callback
        let result = run_sequential_write_test(&config, &test_file_path, Some(&callback));
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(result.is_ok(), "Sequential write test should complete successfully");

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Sequential Write");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Sequential Write");
        assert!(!progress_events.is_empty(), "Should have received progress updates");
        
        // All progress speeds should be non-negative
        for speed in &progress_events {
            assert!(*speed >= 0.0, "Progress speed should be non-negative");
        }

        // Check for completion event
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Sequential Write");
    }

    #[test]
    fn test_sequential_write_file_creation() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Verify file doesn't exist initially
        assert!(!test_file_path.exists());

        // Run the test
        let result = run_sequential_write_test(&config, &test_file_path, None);
        
        // File should exist after test
        assert!(test_file_path.exists(), "Test file should be created");
        
        // File should have some content
        let metadata = fs::metadata(&test_file_path).unwrap();
        assert!(metadata.len() > 0, "Test file should have content");
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok(), "Test should complete successfully");
    }

    #[test]
    fn test_sequential_write_block_size_usage() {
        let mut config = create_test_config();
        config.sequential_block_size = 32 * 1024; // 32KB
        config.file_size_mb = 1; // 1MB file
        
        let test_file_path = create_test_file_path();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        let result = run_sequential_write_test(&config, &test_file_path, None);
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok(), "Test should handle custom block size");
        
        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_sequential_write_error_handling_invalid_path() {
        let config = create_test_config();
        let invalid_path = Path::new("/invalid/nonexistent/path/test.tmp");

        let result = run_sequential_write_test(&config, invalid_path, None);
        
        // Should return an error for invalid path
        assert!(result.is_err(), "Should return error for invalid path");
    }

    #[test]
    fn test_sequential_write_statistics_accuracy() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        let result = run_sequential_write_test(&config, &test_file_path, Some(&callback));
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());
        let test_result = result.unwrap();

        // Verify statistics make sense
        if test_result.sample_count > 1 {
            // If we have multiple samples, min and max might be different
            assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
            
            // Average should be between min and max
            assert!(test_result.avg_speed_mbps >= test_result.min_speed_mbps);
            assert!(test_result.avg_speed_mbps <= test_result.max_speed_mbps);
        }

        // Verify progress events show reasonable speeds
        let progress_events = callback.progress_events_for_test("Sequential Write");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
            assert!(speed < 10000.0, "Speed should be reasonable (< 10GB/s)");
        }
    }

    #[test]
    fn test_sequential_write_duration_limit() {
        let mut config = create_test_config();
        config.test_duration_seconds = 1; // Very short test
        config.file_size_mb = 1000; // Large file that can't be written in 1 second
        
        let test_file_path = create_test_file_path();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        let start_time = std::time::Instant::now();
        let result = run_sequential_write_test(&config, &test_file_path, None);
        let elapsed = start_time.elapsed();
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());
        
        // Test should respect duration limit (with some tolerance for overhead)
        assert!(elapsed.as_secs() <= config.test_duration_seconds + 2);
    }

    #[test]
    fn test_sequential_write_small_file_size() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Very small file
        config.sequential_block_size = 2 * 1024 * 1024; // Block larger than file
        
        let test_file_path = create_test_file_path();
        
        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        let result = run_sequential_write_test(&config, &test_file_path, None);
        
        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle case where block size > file size
        assert!(result.is_ok(), "Should handle block size larger than file");
    }

    #[test]
    fn test_sequential_write_zero_duration_config() {
        let mut config = create_test_config();
        config.test_duration_seconds = 0;
        
        let test_file_path = create_test_file_path();
        
        // This should be caught by config validation, but test the function directly
        // The function should handle this gracefully
        cleanup_test_file(&test_file_path);

        let result = run_sequential_write_test(&config, &test_file_path, None);
        
        cleanup_test_file(&test_file_path);

        // The test should complete successfully even with zero duration
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // With zero duration, the test should complete quickly but may still take some time
        // for file creation and cleanup
        assert!(test_result.test_duration.as_millis() < 2000); // Should be reasonably quick
    }
}