//! Test implementations for different benchmark types

use crate::core::{BenchmarkConfig, BenchmarkError, ProgressCallback, TestResult};
use std::fs::{File, OpenOptions};
use std::path::Path;

/// Determine effective test duration, honoring fast-test override via env var
fn effective_test_duration(config: &BenchmarkConfig) -> std::time::Duration {
    if let Ok(ms_str) = std::env::var("DISK_SPEED_TEST_FAST_TEST_MS") {
        if let Ok(ms) = ms_str.parse::<u64>() {
            if ms > 0 {
                return std::time::Duration::from_millis(ms);
            }
        }
    }
    std::time::Duration::from_secs(config.test_duration_seconds)
}

/// Create a file for I/O operations, choosing between direct I/O and buffered I/O based on config
fn create_io_file(
    config: &BenchmarkConfig,
    path: &Path, 
    size: u64
) -> Result<File, BenchmarkError> {
    if config.disable_direct_io {
        // Use standard buffered I/O
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(BenchmarkError::IoError)?;
        }
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(BenchmarkError::IoError)?;
            
        // Pre-allocate the file size for consistency with direct I/O path
        file.set_len(size).map_err(BenchmarkError::IoError)?;
        Ok(file)
    } else {
        // Use direct I/O
        crate::platform::create_direct_io_file(path, size)
            .map_err(|e| {
                // If direct I/O fails, provide a helpful error message
                match e {
                    crate::platform::PlatformError::DirectIoNotSupported => {
                        BenchmarkError::PlatformError(e)
                    }
                    _ => BenchmarkError::PlatformError(e)
                }
            })
    }
}

/// Open a file for I/O operations, choosing between direct I/O and buffered I/O based on config
fn open_io_file(
    config: &BenchmarkConfig,
    path: &Path,
    write: bool
) -> Result<File, BenchmarkError> {
    if config.disable_direct_io {
        // Use standard buffered I/O
        let mut options = OpenOptions::new();
        if write {
            options.write(true);
        } else {
            options.read(true);
        }
        options.open(path).map_err(BenchmarkError::IoError)
    } else {
        // Use direct I/O
        crate::platform::open_direct_io_file(path, write)
            .map_err(|e| {
                // If direct I/O fails, provide a helpful error message
                match e {
                    crate::platform::PlatformError::DirectIoNotSupported => {
                        BenchmarkError::PlatformError(e)
                    }
                    _ => BenchmarkError::PlatformError(e)
                }
            })
    }
}

/// Get the appropriate block size, aligned for direct I/O if needed
fn get_block_size(config: &BenchmarkConfig, requested_size: usize) -> usize {
    if config.disable_direct_io {
        // For buffered I/O, use the requested size as-is
        requested_size
    } else {
        // For direct I/O, ensure size is aligned
        crate::platform::align_block_size_for_direct_io(requested_size)
    }
}

/// Sequential write test implementation
pub fn run_sequential_write_test(
    config: &BenchmarkConfig,
    test_file_path: &Path,
    progress_callback: Option<&dyn ProgressCallback>,
) -> Result<TestResult, BenchmarkError> {
    use crate::core::RealTimeStatsTracker;
    use std::io::Write;
    use std::time::Instant;

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Sequential Write");
    }

    // We'll handle progress reporting directly since we have a reference

    // Create the test file (direct I/O or buffered I/O based on config)
    let mut file = create_io_file(config, test_file_path, config.file_size_bytes())?;

    // Create buffer with the configured block size (aligned for direct I/O if needed)
    let block_size = get_block_size(config, config.sequential_block_size);
    let mut buffer = vec![0u8; block_size];

    // Fill buffer with test pattern (alternating bytes to avoid compression)
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = effective_test_duration(config);
    let mut emitted_progress = false;

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
                emitted_progress = true;
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
        crate::platform::sync_file_system(test_file_path)?;
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Fallback: ensure at least one progress emission for very short tests
    if !emitted_progress {
        if let Some(callback) = progress_callback {
            callback.on_progress("Sequential Write", result.avg_speed_mbps);
        }
    }

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
    use std::io::Read;
    use std::time::Instant;

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Sequential Read");
    }

    // Open the test file (direct I/O or buffered I/O based on config)
    let mut file = open_io_file(config, test_file_path, false)?;

    // Create buffer with the configured block size (aligned for direct I/O if needed)
    let block_size = get_block_size(config, config.sequential_block_size);
    let mut buffer = vec![0u8; block_size];

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = effective_test_duration(config);
    let mut emitted_progress = false;

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
                        emitted_progress = true;
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

    // Fallback: ensure at least one progress emission for very short tests
    if !emitted_progress {
        if let Some(callback) = progress_callback {
            callback.on_progress("Sequential Read", result.avg_speed_mbps);
        }
    }

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
    use rand::Rng;
    use std::io::{Seek, SeekFrom, Write};
    use std::time::Instant;

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Random Write");
    }

    // Open the test file (direct I/O or buffered I/O based on config)
    let mut file = open_io_file(config, test_file_path, true)?;

    // Create buffer with the configured random block size (aligned for direct I/O if needed)
    let block_size = get_block_size(config, config.random_block_size);
    let mut buffer = vec![0u8; block_size];

    // Fill buffer with test pattern (alternating bytes to avoid compression)
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = effective_test_duration(config);
    let mut emitted_progress = false;

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
                emitted_progress = true;
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
        crate::platform::sync_file_system(test_file_path)?;
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Fallback: ensure at least one progress emission for very short tests
    if !emitted_progress {
        if let Some(callback) = progress_callback {
            callback.on_progress("Random Write", result.avg_speed_mbps);
        }
    }

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
    use rand::Rng;
    use std::io::{Read, Seek, SeekFrom};
    use std::time::Instant;

    // Report test start
    if let Some(callback) = progress_callback {
        callback.on_test_start("Random Read");
    }

    // Open the test file (direct I/O or buffered I/O based on config)
    let mut file = open_io_file(config, test_file_path, false)?;

    // Create buffer with the configured random block size (aligned for direct I/O if needed)
    let block_size = get_block_size(config, config.random_block_size);
    let mut buffer = vec![0u8; block_size];

    // Initialize statistics tracking
    let mut stats_tracker = RealTimeStatsTracker::new();
    let test_start = Instant::now();
    let test_duration = effective_test_duration(config);
    let mut emitted_progress = false;

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
                        emitted_progress = true;
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

    // Fallback: ensure at least one progress emission for very short tests
    if !emitted_progress {
        if let Some(callback) = progress_callback {
            callback.on_progress("Random Read", result.avg_speed_mbps);
        }
    }

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
    use std::time::Instant;

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
    let test_duration = effective_test_duration(config);
    let mut emitted_progress = false;

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

            // Record per-block speed and report progress periodically
            let elapsed = copy_start.elapsed();
            if let Some(current_speed) = stats_tracker.record_block(bytes_to_copy, elapsed) {
                if let Some(callback) = progress_callback {
                    callback.on_progress("Memory Copy", current_speed);
                    emitted_progress = true;
                }
            }
        }

        // If we've completed a full copy, continue with another iteration
        // This ensures we keep the test running for the full duration
    }

    // Finalize statistics
    let result = stats_tracker.finalize();

    // Fallback: ensure at least one progress emission for very short tests
    if !emitted_progress {
        if let Some(callback) = progress_callback {
            callback.on_progress("Memory Copy", result.avg_speed_mbps);
        }
    }

    // Report test completion
    if let Some(callback) = progress_callback {
        callback.on_test_complete("Memory Copy", &result);
    }

    Ok(result)
}
