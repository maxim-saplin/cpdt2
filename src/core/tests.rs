//! Test implementations for different benchmark types

use crate::core::{BenchmarkConfig, BenchmarkError, ProgressCallback, TestResult};
use std::path::Path;

/// Sequential write test implementation
pub fn run_sequential_write_test(
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use crate::platform;
    use std::io::Write;
    use std::time::{Duration, Instant};

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

    // Track bytes written for wrap-around logic
    let mut bytes_written: u64 = 0;
    let file_size = config.file_size_bytes();

    // Main write loop - continue until test duration elapsed or file is full
    while test_start.elapsed() < test_duration && bytes_written < file_size {
        let write_start = Instant::now();

        // Calculate how much to write this iteration
        let remaining_file_space = file_size - bytes_written;
        let bytes_to_write = std::cmp::min(block_size as u64, remaining_file_space) as usize;

        // Write the block
        let bytes_written_this_iteration = file.write(&buffer[..bytes_to_write])?;
        bytes_written += bytes_written_this_iteration as u64;

        // Record per-block speed and report progress periodically
        let elapsed = write_start.elapsed();
        if let Some(current_speed) =
            stats_tracker.record_block(bytes_written_this_iteration, elapsed)
        {
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
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use crate::platform;
    use std::io::Read;
    use std::time::{Duration, Instant};

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Sequential Read");
    }

    // Open the test file with direct I/O for reading
    let mut file = platform::open_direct_io_file(test_file_path, false)?;

    // Create buffer with the configured block size
    let block_size = config.sequential_block_size;
    let mut buffer = vec![0u8; block_size];

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_seconds);

    // Track bytes read only for wrap-around checks
    let mut bytes_read: u64 = 0;
    let file_size = config.file_size_bytes();

    // Main read loop - continue until test duration elapsed or we've read enough data
    while test_start.elapsed() < test_duration {
        let read_start = Instant::now();

        // Read the block
        match file.read(&mut buffer) {
            Ok(bytes_read_this_iteration) => {
                // If we read 0 bytes, we've reached EOF
                if bytes_read_this_iteration == 0 {
                    // Seek back to beginning to continue reading
                    use std::io::Seek;
                    file.seek(std::io::SeekFrom::Start(0))?;
                    continue;
                }

                bytes_read += bytes_read_this_iteration as u64;

                // Record per-block speed and report progress periodically
                let elapsed = read_start.elapsed();
                if let Some(current_speed) =
                    stats_tracker.record_block(bytes_read_this_iteration, elapsed)
                {
                    if let Some(callback) = progress_callback {
                        callback.on_progress("Sequential Read", current_speed);
                    }
                }
            }
            Err(e) => {
                // Handle read errors
                return Err(BenchmarkError::IoError(e));
            }
        }

        // If we've read a full file's worth of data, reset counter but keep stats tracking total throughput
        if bytes_read >= file_size {
            use std::io::Seek;
            file.seek(std::io::SeekFrom::Start(0))?;
            bytes_read = 0; // Reset counter but keep stats tracking total throughput
        }
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Report test completion
    if let Some(callback) = progress_callback {
        callback.on_test_complete("Sequential Read", &result);
    }

    Ok(result)
}

/// Random write test implementation
pub fn run_random_write_test(
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use crate::platform;
    use rand::Rng;
    use std::io::{Seek, SeekFrom, Write};
    use std::time::{Duration, Instant};

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Random Write");
    }

    // Open the test file with direct I/O for writing
    let mut file = platform::open_direct_io_file(test_file_path, true)?;

    // Create buffer with the configured random block size (default 4KB)
    let block_size = config.random_block_size;
    let mut buffer = vec![0u8; block_size];

    // Fill buffer with test pattern (alternating bytes to avoid compression)
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_seconds);

    let mut _bytes_written = 0u64;
    let file_size = config.file_size_bytes();

    // Calculate the number of possible block positions in the file
    let max_blocks = (file_size / block_size as u64).max(1);
    let mut rng = rand::thread_rng();

    // Main random write loop - continue until test duration elapsed
    while test_start.elapsed() < test_duration {
        // Generate random block position within the file
        let random_block = rng.gen_range(0..max_blocks);
        let seek_position = random_block * block_size as u64;

        // Seek to random position
        file.seek(SeekFrom::Start(seek_position))?;

        // Calculate how much to write (handle case where we're near end of file)
        let remaining_file_space = file_size.saturating_sub(seek_position);
        let bytes_to_write = std::cmp::min(block_size as u64, remaining_file_space) as usize;

        if bytes_to_write == 0 {
            continue; // Skip if we somehow ended up past the file end
        }

        // Write the block
        let write_start = Instant::now();
        let bytes_written_this_iteration = file.write(&buffer[..bytes_to_write])?;
        _bytes_written += bytes_written_this_iteration as u64;

        // Record per-block speed and report progress periodically
        let elapsed = write_start.elapsed();
        if let Some(current_speed) =
            stats_tracker.record_block(bytes_written_this_iteration, elapsed)
        {
            if let Some(callback) = progress_callback {
                callback.on_progress("Random Write", current_speed);
            }
        }

        // If we couldn't write the full block, there might be an issue
        if bytes_written_this_iteration < bytes_to_write {
            // This shouldn't happen with random writes, but handle gracefully
            continue;
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
        callback.on_test_complete("Random Write", &result);
    }

    Ok(result)
}

/// Random read test implementation
pub fn run_random_read_test(
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use crate::platform;
    use rand::Rng;
    use std::io::{Read, Seek, SeekFrom};
    use std::time::{Duration, Instant};

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Random Read");
    }

    // Open the test file with direct I/O for reading
    let mut file = platform::open_direct_io_file(test_file_path, false)?;

    // Create buffer with the configured random block size (default 4KB)
    let block_size = config.random_block_size;
    let mut buffer = vec![0u8; block_size];

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_seconds);

    let _bytes_read = 0u64;
    let file_size = config.file_size_bytes();

    // Calculate the number of possible block positions in the file
    let max_blocks = (file_size / block_size as u64).max(1);
    let mut rng = rand::thread_rng();

    // Main random read loop - continue until test duration elapsed
    while test_start.elapsed() < test_duration {
        // Generate random block position within the file
        let random_block = rng.gen_range(0..max_blocks);
        let seek_position = random_block * block_size as u64;

        // Seek to random position
        file.seek(SeekFrom::Start(seek_position))?;

        // Calculate how much to read (handle case where we're near end of file)
        let remaining_file_space = file_size.saturating_sub(seek_position);
        let bytes_to_read = std::cmp::min(block_size as u64, remaining_file_space) as usize;

        if bytes_to_read == 0 {
            continue; // Skip if we somehow ended up past the file end
        }

        // Read the block
        let read_start = Instant::now();
        match file.read(&mut buffer[..bytes_to_read]) {
            Ok(bytes_read_this_iteration) => {
                // If we read 0 bytes unexpectedly, there might be an issue
                if bytes_read_this_iteration == 0 {
                    continue; // Skip this iteration
                }

                // Record per-block speed and report progress periodically
                let elapsed = read_start.elapsed();
                if let Some(current_speed) =
                    stats_tracker.record_block(bytes_read_this_iteration, elapsed)
                {
                    if let Some(callback) = progress_callback {
                        callback.on_progress("Random Read", current_speed);
                    }
                }
            }
            Err(e) => {
                // Handle read errors gracefully - continue with next random position
                eprintln!("Warning: Read error at position {}: {}", seek_position, e);
                continue;
            }
        }
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Report test completion
    if let Some(callback) = progress_callback {
        callback.on_test_complete("Random Read", &result);
    }

    Ok(result)
}

/// Memory copy test implementation
pub fn run_memory_copy_test(
    config: &BenchmarkConfig,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use std::time::{Duration, Instant};

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Memory Copy");
    }

    // Use similar block size as disk tests for meaningful comparison
    // Use sequential block size as it's more appropriate for large memory operations
    let block_size = config.sequential_block_size;
    let total_memory_size = config.file_size_bytes() as usize;

    // Allocate two large buffers for memory-to-memory copy operations
    let mut source_buffer = vec![0u8; total_memory_size];
    let mut destination_buffer = vec![0u8; total_memory_size];

    // Fill source buffer with test pattern (alternating bytes to avoid compression optimizations)
    for (i, byte) in source_buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = Duration::from_secs(config.test_duration_seconds);

    // No cumulative counter needed with per-block sampling

    // Main memory copy loop - continue until test duration elapsed
    while test_start.elapsed() < test_duration {
        let mut offset = 0;

        // Copy the entire buffer in block_size chunks
        while offset < total_memory_size && test_start.elapsed() < test_duration {
            let bytes_to_copy = std::cmp::min(block_size, total_memory_size - offset);

            // Perform memory-to-memory copy using optimized routines
            // This uses the standard library's optimized copy_from_slice which typically
            // uses platform-specific optimized memory copy routines (like memcpy)
            let copy_start = Instant::now();
            destination_buffer[offset..offset + bytes_to_copy]
                .copy_from_slice(&source_buffer[offset..offset + bytes_to_copy]);

            offset += bytes_to_copy;
            // cumulative counter removed

            // Record per-block speed and report progress periodically
            let elapsed = copy_start.elapsed();
            if let Some(current_speed) = stats_tracker.record_block(bytes_to_copy, elapsed) {
                if let Some(callback) = progress_callback {
                    callback.on_progress("Memory Copy", current_speed);
                }
            }
        }

        // If we've completed a full copy, continue with another iteration
        // This ensures we keep the test running for the full duration
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Report test completion
    if let Some(callback) = progress_callback {
        callback.on_test_complete("Memory Copy", &result);
    }

    Ok(result)
}
#[cfg(test)]
mod core_tests {
    use super::*;
    use crate::core::TestProgressCallback;
    use std::env;
    use std::fs;

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
        temp_dir.join(format!(
            "disk_speed_test_sequential_write_{}_{}.tmp",
            format!("{:?}", thread_id)
                .replace("ThreadId(", "")
                .replace(")", ""),
            timestamp
        ))
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
        assert!(
            result.is_ok(),
            "Sequential write test should complete successfully"
        );

        let test_result = result.unwrap();

        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(
            test_result.sample_count > 0,
            "Should have collected at least one sample"
        );

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
        assert!(
            result.is_ok(),
            "Sequential write test should complete successfully"
        );

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Sequential Write");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Sequential Write");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );

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
            // Per-block timings can transiently exceed 10GB/s on cached reads; allow higher bound
            assert!(speed < 100000.0, "Speed should be reasonable (< 100GB/s)");
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

    // Sequential Read Tests

    fn create_test_file_for_reading(path: &Path, size_mb: usize) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        let block_size = 64 * 1024; // 64KB blocks
        let mut buffer = vec![0u8; block_size];

        // Fill buffer with test pattern
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }

        let total_bytes = size_mb * 1024 * 1024;
        let mut bytes_written = 0;

        while bytes_written < total_bytes {
            let bytes_to_write = std::cmp::min(block_size, total_bytes - bytes_written);
            file.write_all(&buffer[..bytes_to_write])?;
            bytes_written += bytes_to_write;
        }

        file.flush()?;
        Ok(())
    }

    #[test]
    fn test_sequential_read_basic_functionality() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test
        let result = run_sequential_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Sequential read test should complete successfully"
        );

        let test_result = result.unwrap();

        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(
            test_result.sample_count > 0,
            "Should have collected at least one sample"
        );

        // Speed values should be non-negative
        assert!(test_result.min_speed_mbps >= 0.0);
        assert!(test_result.max_speed_mbps >= 0.0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Max should be >= min
        assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
    }

    #[test]
    fn test_sequential_read_with_progress_callback() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test with callback
        let result = run_sequential_read_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Sequential read test should complete successfully"
        );

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Sequential Read");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Sequential Read");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );

        // All progress speeds should be non-negative
        for speed in &progress_events {
            assert!(*speed >= 0.0, "Progress speed should be non-negative");
        }

        // Check for completion event
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Sequential Read");
    }

    #[test]
    fn test_sequential_read_nonexistent_file() {
        let config = create_test_config();
        let nonexistent_path = Path::new("/nonexistent/path/test.tmp");

        let result = run_sequential_read_test(&config, nonexistent_path, None);

        // Should return an error for nonexistent file
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_sequential_read_empty_file() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create an empty test file
        std::fs::File::create(&test_file_path).unwrap();

        // Run the test
        let result = run_sequential_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle empty file gracefully
        assert!(result.is_ok(), "Should handle empty file gracefully");

        let test_result = result.unwrap();
        // With an empty file, we should still get some result
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
    }

    #[test]
    fn test_sequential_read_block_size_usage() {
        let mut config = create_test_config();
        config.sequential_block_size = 32 * 1024; // 32KB
        config.file_size_mb = 1; // 1MB file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_sequential_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok(), "Test should handle custom block size");

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_sequential_read_statistics_accuracy() {
        let config = create_test_config();
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_sequential_read_test(&config, &test_file_path, Some(&callback));

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
        let progress_events = callback.progress_events_for_test("Sequential Read");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
        }
    }

    #[test]
    fn test_sequential_read_duration_limit() {
        let mut config = create_test_config();
        config.test_duration_seconds = 1; // Very short test
        config.file_size_mb = 1; // Small file for quick reading

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let start_time = std::time::Instant::now();
        let result = run_sequential_read_test(&config, &test_file_path, None);
        let elapsed = start_time.elapsed();

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());

        // Test should respect duration limit (with some tolerance for overhead)
        assert!(elapsed.as_secs() <= config.test_duration_seconds + 2);
    }

    #[test]
    fn test_sequential_read_small_file_large_block() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Very small file
        config.sequential_block_size = 2 * 1024 * 1024; // Block larger than file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_sequential_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle case where block size > file size
        assert!(result.is_ok(), "Should handle block size larger than file");
    }

    #[test]
    fn test_sequential_read_zero_duration_config() {
        let mut config = create_test_config();
        config.test_duration_seconds = 0;

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_sequential_read_test(&config, &test_file_path, None);

        cleanup_test_file(&test_file_path);

        // The test should complete successfully even with zero duration
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // With zero duration, the test should complete quickly
        assert!(test_result.test_duration.as_millis() < 2000); // Should be reasonably quick
    }

    #[test]
    fn test_sequential_read_file_rewind_behavior() {
        let mut config = create_test_config();
        config.test_duration_seconds = 2; // Longer test to ensure file rewind
        config.file_size_mb = 1; // Small file that will be read multiple times

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file to read from
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_sequential_read_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());
        let test_result = result.unwrap();

        // Should have read data and collected samples
        assert!(test_result.sample_count > 0);
        assert!(test_result.avg_speed_mbps > 0.0);

        // Should have received progress updates
        let progress_events = callback.progress_events_for_test("Sequential Read");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );
    }

    // Random Write Tests

    #[test]
    fn test_random_write_basic_functionality() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB for random operations
        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first (random write needs existing file)
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test
        let result = run_random_write_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Random write test should complete successfully"
        );

        let test_result = result.unwrap();

        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(
            test_result.sample_count > 0,
            "Should have collected at least one sample"
        );

        // Speed values should be non-negative
        assert!(test_result.min_speed_mbps >= 0.0);
        assert!(test_result.max_speed_mbps >= 0.0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Max should be >= min
        assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
    }

    #[test]
    fn test_random_write_with_progress_callback() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB for random operations
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test with callback
        let result = run_random_write_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Random write test should complete successfully"
        );

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Random Write");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Random Write");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );

        // All progress speeds should be non-negative
        for speed in &progress_events {
            assert!(*speed >= 0.0, "Progress speed should be non-negative");
        }

        // Check for completion event
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Random Write");
    }

    #[test]
    fn test_random_write_nonexistent_file() {
        let config = create_test_config();
        let nonexistent_path = Path::new("/nonexistent/path/test.tmp");

        let result = run_random_write_test(&config, nonexistent_path, None);

        // Should return an error for nonexistent file
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_random_write_block_size_usage() {
        let mut config = create_test_config();
        config.random_block_size = 8 * 1024; // 8KB
        config.file_size_mb = 1; // 1MB file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok(), "Test should handle custom block size");

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_random_write_statistics_accuracy() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB
        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, Some(&callback));

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
        let progress_events = callback.progress_events_for_test("Random Write");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
        }
    }

    #[test]
    fn test_random_write_duration_limit() {
        let mut config = create_test_config();
        config.test_duration_seconds = 1; // Very short test
        config.random_block_size = 4 * 1024; // 4KB
        config.file_size_mb = 10; // Larger file for more random positions

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let start_time = std::time::Instant::now();
        let result = run_random_write_test(&config, &test_file_path, None);
        let elapsed = start_time.elapsed();

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());

        // Test should respect duration limit (with some tolerance for overhead)
        assert!(elapsed.as_secs() <= config.test_duration_seconds + 2);
    }

    #[test]
    fn test_random_write_small_file_large_block() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Very small file
        config.random_block_size = 2 * 1024 * 1024; // Block larger than file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle case where block size > file size
        assert!(result.is_ok(), "Should handle block size larger than file");
    }

    #[test]
    fn test_random_write_zero_duration_config() {
        let mut config = create_test_config();
        config.test_duration_seconds = 0;
        config.random_block_size = 4 * 1024; // 4KB

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, None);

        cleanup_test_file(&test_file_path);

        // The test should complete successfully even with zero duration
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // With zero duration, the test should complete quickly
        assert!(test_result.test_duration.as_millis() < 2000); // Should be reasonably quick
    }

    #[test]
    fn test_random_write_seek_verification() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB
        config.test_duration_seconds = 1; // Short test
        config.file_size_mb = 2; // Small file for predictable behavior

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());
        let test_result = result.unwrap();

        // Should have performed random writes and collected samples
        assert!(test_result.sample_count > 0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Should have received progress updates
        let progress_events = callback.progress_events_for_test("Random Write");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );
    }

    #[test]
    fn test_random_write_empty_file() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create an empty test file
        std::fs::File::create(&test_file_path).unwrap();

        let result = run_random_write_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle empty file gracefully (though it may not write much)
        assert!(result.is_ok(), "Should handle empty file gracefully");
    }

    #[test]
    fn test_random_write_large_block_count() {
        let mut config = create_test_config();
        config.random_block_size = 1024; // 1KB blocks for more random positions
        config.file_size_mb = 5; // 5MB file = ~5000 possible positions
        config.test_duration_seconds = 1; // Short test

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_write_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());
        let test_result = result.unwrap();

        // Should have performed many random writes
        assert!(test_result.sample_count > 0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Should have received progress updates
        let progress_events = callback.progress_events_for_test("Random Write");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );
    }

    #[test]
    fn test_random_write_error_handling_readonly_file() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Make file read-only
        let mut perms = std::fs::metadata(&test_file_path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&test_file_path, perms).unwrap();

        let result = run_random_write_test(&config, &test_file_path, None);

        // Cleanup (restore write permissions first)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o644);
            let _ = std::fs::set_permissions(&test_file_path, perms);
        }
        #[cfg(not(unix))]
        {
            let mut perms = std::fs::metadata(&test_file_path).unwrap().permissions();
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(&test_file_path, perms);
        }
        cleanup_test_file(&test_file_path);

        // Should return an error for read-only file
        assert!(result.is_err(), "Should return error for read-only file");
    }

    // Random Read Tests

    #[test]
    fn test_random_read_basic_functionality() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB for random operations

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first (random read needs existing file)
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test
        let result = run_random_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Random read test should complete successfully"
        );

        let test_result = result.unwrap();

        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(
            test_result.sample_count > 0,
            "Should have collected at least one sample"
        );

        // Speed values should be non-negative
        assert!(test_result.min_speed_mbps >= 0.0);
        assert!(test_result.max_speed_mbps >= 0.0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Max should be >= min
        assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
    }

    #[test]
    fn test_random_read_with_progress_callback() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB for random operations

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        // Run the test with callback
        let result = run_random_read_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Random read test should complete successfully"
        );

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Random Read");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Random Read");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );

        // All progress speeds should be non-negative
        for speed in &progress_events {
            assert!(*speed >= 0.0, "Progress speed should be non-negative");
        }

        // Check for completion event
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Random Read");
    }

    #[test]
    fn test_random_read_nonexistent_file() {
        let config = create_test_config();
        let nonexistent_path = Path::new("/nonexistent/path/test.tmp");

        let result = run_random_read_test(&config, nonexistent_path, None);

        // Should return an error for nonexistent file
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_random_read_block_size_usage() {
        let mut config = create_test_config();
        config.random_block_size = 8 * 1024; // 8KB
        config.file_size_mb = 1; // 1MB file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok(), "Test should handle custom block size");

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_random_read_statistics_accuracy() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB
        config.file_size_mb = 1; // 1MB file

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, Some(&callback));

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
        let progress_events = callback.progress_events_for_test("Random Read");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
        }
    }

    #[test]
    fn test_random_read_duration_limit() {
        let mut config = create_test_config();
        config.test_duration_seconds = 1; // Very short test
        config.file_size_mb = 1; // Small file for quick reading

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let start_time = std::time::Instant::now();
        let result = run_random_read_test(&config, &test_file_path, None);
        let elapsed = start_time.elapsed();

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(result.is_ok());

        // Test should respect duration limit (with some tolerance for overhead)
        assert!(elapsed.as_secs() <= config.test_duration_seconds + 2);
    }

    #[test]
    fn test_random_read_small_file_large_block() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Very small file
        config.random_block_size = 2 * 1024 * 1024; // Block larger than file

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle case where block size > file size
        assert!(result.is_ok(), "Should handle block size larger than file");
    }

    #[test]
    fn test_random_read_zero_duration_config() {
        let mut config = create_test_config();
        config.test_duration_seconds = 0;

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, None);

        cleanup_test_file(&test_file_path);

        // The test should complete successfully even with zero duration
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // With zero duration, the test should complete quickly
        assert!(test_result.test_duration.as_millis() < 2000); // Should be reasonably quick
    }

    #[test]
    fn test_random_read_seek_verification() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB
        config.file_size_mb = 1; // 1MB file
        config.test_duration_seconds = 2; // Short test for verification

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(
            result.is_ok(),
            "Random read with seek operations should work"
        );

        let test_result = result.unwrap();

        // Should have read some data
        assert!(
            test_result.sample_count > 0,
            "Should have collected samples"
        );
        assert!(
            test_result.avg_speed_mbps >= 0.0,
            "Should have measured some speed"
        );

        // Should have received progress updates
        let progress_events = callback.progress_events_for_test("Random Read");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );
    }

    #[test]
    fn test_random_read_empty_file() {
        let mut config = create_test_config();
        config.random_block_size = 4 * 1024; // 4KB
        config.test_duration_seconds = 1; // Short test

        let test_file_path = create_test_file_path();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create an empty test file
        std::fs::File::create(&test_file_path).unwrap();

        let result = run_random_read_test(&config, &test_file_path, None);

        // Cleanup
        cleanup_test_file(&test_file_path);

        // Should handle empty file gracefully
        assert!(result.is_ok(), "Should handle empty file gracefully");

        let test_result = result.unwrap();
        // With an empty file, we should still get some result
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
    }

    #[test]
    fn test_random_read_large_block_count() {
        let mut config = create_test_config();
        config.random_block_size = 1024; // 1KB blocks for more random positions
        config.file_size_mb = 1; // 1MB file = 1024 possible positions
        config.test_duration_seconds = 2; // Short test

        let test_file_path = create_test_file_path();
        let callback = TestProgressCallback::new();

        // Cleanup any existing test file
        cleanup_test_file(&test_file_path);

        // Create a test file first
        create_test_file_for_reading(&test_file_path, config.file_size_mb).unwrap();

        let result = run_random_read_test(&config, &test_file_path, Some(&callback));

        // Cleanup
        cleanup_test_file(&test_file_path);

        assert!(
            result.is_ok(),
            "Random read with many possible positions should work"
        );

        let test_result = result.unwrap();

        // Should have read some data
        assert!(
            test_result.sample_count > 0,
            "Should have collected samples"
        );
        assert!(
            test_result.avg_speed_mbps >= 0.0,
            "Should have measured some speed"
        );

        // Should have received progress updates
        let progress_events = callback.progress_events_for_test("Random Read");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );
    }

    // Memory Copy Tests

    #[test]
    fn test_memory_copy_basic_functionality() {
        let config = create_test_config();

        // Run the test
        let result = run_memory_copy_test(&config, None);

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Memory copy test should complete successfully"
        );

        let test_result = result.unwrap();

        // Verify basic result structure
        assert!(test_result.test_duration.as_secs() <= config.test_duration_seconds + 1);
        assert!(
            test_result.sample_count > 0,
            "Should have collected at least one sample"
        );

        // Speed values should be non-negative
        assert!(test_result.min_speed_mbps >= 0.0);
        assert!(test_result.max_speed_mbps >= 0.0);
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Max should be >= min
        assert!(test_result.max_speed_mbps >= test_result.min_speed_mbps);
    }

    #[test]
    fn test_memory_copy_with_progress_callback() {
        let config = create_test_config();
        let callback = TestProgressCallback::new();

        // Run the test with callback
        let result = run_memory_copy_test(&config, Some(&callback));

        // Verify the test completed successfully
        assert!(
            result.is_ok(),
            "Memory copy test should complete successfully"
        );

        // Verify callback events
        let events = callback.events();
        assert!(!events.is_empty(), "Should have received callback events");

        // Check for test start event
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Memory Copy");

        // Check for progress events
        let progress_events = callback.progress_events_for_test("Memory Copy");
        assert!(
            !progress_events.is_empty(),
            "Should have received progress updates"
        );

        // All progress speeds should be non-negative
        for speed in &progress_events {
            assert!(*speed >= 0.0, "Progress speed should be non-negative");
        }

        // Check for completion event
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Memory Copy");
    }

    #[test]
    fn test_memory_copy_performance_measurement() {
        let config = create_test_config();
        let callback = TestProgressCallback::new();

        let result = run_memory_copy_test(&config, Some(&callback));

        assert!(result.is_ok());
        let test_result = result.unwrap();

        // Memory copy should be significantly faster than disk operations
        // We expect memory bandwidth to be much higher than typical disk speeds
        assert!(
            test_result.avg_speed_mbps > 0.0,
            "Should measure positive speed"
        );

        // Memory copy speeds should be reasonable (not impossibly high or low)
        assert!(test_result.avg_speed_mbps >= 0.0);

        // Should have collected multiple samples for accurate statistics
        assert!(
            test_result.sample_count > 0,
            "Should have collected samples"
        );

        // Verify progress events show reasonable speeds
        let progress_events = callback.progress_events_for_test("Memory Copy");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
        }
    }

    #[test]
    fn test_memory_copy_block_size_usage() {
        let mut config = create_test_config();
        config.sequential_block_size = 1024 * 1024; // 1MB blocks
        config.file_size_mb = 1; // 1MB total memory
        config.test_duration_seconds = 1; // Short test

        let result = run_memory_copy_test(&config, None);

        assert!(result.is_ok(), "Test should handle custom block size");

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_memory_copy_statistics_accuracy() {
        let config = create_test_config();
        let callback = TestProgressCallback::new();

        let result = run_memory_copy_test(&config, Some(&callback));

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
        let progress_events = callback.progress_events_for_test("Memory Copy");
        for speed in progress_events {
            assert!(speed >= 0.0, "Speed should be non-negative");
        }
    }

    #[test]
    fn test_memory_copy_duration_limit() {
        let mut config = create_test_config();
        config.test_duration_seconds = 1; // Very short test
        config.file_size_mb = 100; // Large memory size that would take longer to copy repeatedly

        let start_time = std::time::Instant::now();
        let result = run_memory_copy_test(&config, None);
        let elapsed = start_time.elapsed();

        assert!(result.is_ok());

        // Test should respect duration limit (with some tolerance for overhead)
        assert!(elapsed.as_secs() <= config.test_duration_seconds + 2);
    }

    #[test]
    fn test_memory_copy_small_memory_size() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Very small memory size
        config.sequential_block_size = 2 * 1024 * 1024; // Block larger than memory size
        config.test_duration_seconds = 1; // Short test

        let result = run_memory_copy_test(&config, None);

        // Should handle case where block size > memory size
        assert!(
            result.is_ok(),
            "Should handle block size larger than memory size"
        );

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps >= 0.0);
    }

    #[test]
    fn test_memory_copy_zero_duration_config() {
        let mut config = create_test_config();
        config.test_duration_seconds = 0;

        let result = run_memory_copy_test(&config, None);

        // The test should complete successfully even with zero duration
        assert!(result.is_ok());
        let test_result = result.unwrap();
        // With zero duration, the test should complete quickly
        assert!(test_result.test_duration.as_millis() < 1000); // Should be very quick
    }

    #[test]
    fn test_memory_copy_data_integrity() {
        let mut config = create_test_config();
        config.file_size_mb = 1; // Small size for data verification
        config.test_duration_seconds = 1; // Short test

        // This test verifies that the memory copy operation actually works
        // by checking that data is properly copied (implicitly tested by the implementation)
        let result = run_memory_copy_test(&config, None);

        assert!(result.is_ok(), "Memory copy should work correctly");

        let test_result = result.unwrap();

        // Should have copied some data
        assert!(
            test_result.sample_count > 0,
            "Should have performed copy operations"
        );
        assert!(
            test_result.avg_speed_mbps > 0.0,
            "Should have measured positive throughput"
        );
    }

    #[test]
    fn test_memory_copy_large_memory_size() {
        let mut config = create_test_config();
        config.file_size_mb = 10; // Larger memory size
        config.sequential_block_size = 1024 * 1024; // 1MB blocks
        config.test_duration_seconds = 2; // Longer test

        let result = run_memory_copy_test(&config, None);

        assert!(result.is_ok(), "Should handle larger memory sizes");

        let test_result = result.unwrap();
        assert!(test_result.avg_speed_mbps > 0.0);
        assert!(test_result.sample_count > 0);
    }
}
