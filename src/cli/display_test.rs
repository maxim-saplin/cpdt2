//! Tests for CLI display and output formatting

use super::*;
use disk_speed_test::core::{TestResult, BenchmarkResults, ProgressCallback};
use std::time::Duration;

/// Create a sample test result for testing
fn create_sample_test_result() -> TestResult {
    TestResult::new(
        50.0,  // min_speed_mbps
        150.0, // max_speed_mbps
        100.0, // avg_speed_mbps
        Duration::from_secs(10),
        100,   // sample_count
    )
}

/// Create sample benchmark results for testing
fn create_sample_benchmark_results() -> BenchmarkResults {
    BenchmarkResults {
        sequential_write: TestResult::new(80.0, 120.0, 100.0, Duration::from_secs(10), 100),
        sequential_read: TestResult::new(90.0, 130.0, 110.0, Duration::from_secs(10), 100),
        random_write: TestResult::new(15.0, 25.0, 20.0, Duration::from_secs(10), 100),
        random_read: TestResult::new(18.0, 28.0, 23.0, Duration::from_secs(10), 100),
        memory_copy: TestResult::new(800.0, 1200.0, 1000.0, Duration::from_secs(10), 100),
    }
}

#[test]
fn test_cli_progress_callback_creation() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    assert!(!callback.verbose);
    
    let verbose_callback = CliProgressCallback::new_verbose(OutputFormat::Json);
    assert!(verbose_callback.verbose);
}

#[test]
fn test_format_speed() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    // Test various speed ranges
    assert_eq!(callback.format_speed(0.5), "0.500 MB/s");
    assert_eq!(callback.format_speed(1.5), "1.50 MB/s");
    assert_eq!(callback.format_speed(100.0), "100.00 MB/s");
    assert_eq!(callback.format_speed(1500.0), "1.5 GB/s");
    assert_eq!(callback.format_speed(2048.0), "2.0 GB/s");
}

#[test]
fn test_format_duration() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    assert_eq!(callback.format_duration(Duration::from_secs(5)), "5.0s");
    assert_eq!(callback.format_duration(Duration::from_secs(65)), "1m 5s");
    assert_eq!(callback.format_duration(Duration::from_secs(125)), "2m 5s");
    assert_eq!(callback.format_duration(Duration::from_millis(1500)), "1.5s");
}

#[test]
fn test_colorize_functionality() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    // Test colorization (exact output depends on terminal support)
    let colored = callback.colorize("test", "1;32");
    
    // Should either be colored or plain text
    assert!(colored.contains("test"));
    
    // Test with colors disabled
    let mut no_color_callback = CliProgressCallback::new(OutputFormat::Table);
    no_color_callback.use_colors = false;
    let plain = no_color_callback.colorize("test", "1;32");
    assert_eq!(plain, "test");
}

#[test]
fn test_progress_callback_interface() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    let test_result = create_sample_test_result();
    
    // These should not panic
    callback.on_test_start("Sequential Write");
    callback.on_progress("Sequential Write", 75.5);
    callback.on_test_complete("Sequential Write", &test_result);
}

#[test]
fn test_create_progress_bar() {
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    // Test different widths
    let bar_0 = callback.create_progress_bar(0);
    let bar_10 = callback.create_progress_bar(10);
    let bar_20 = callback.create_progress_bar(20);
    let bar_30 = callback.create_progress_bar(30); // Should be capped at 20
    
    // All should contain brackets
    assert!(bar_0.contains("["));
    assert!(bar_0.contains("]"));
    assert!(bar_10.contains("["));
    assert!(bar_10.contains("]"));
    assert!(bar_20.contains("["));
    assert!(bar_20.contains("]"));
    assert!(bar_30.contains("["));
    assert!(bar_30.contains("]"));
}

#[test]
fn test_extreme_values() {
    let _extreme_result = TestResult::new(
        0.001,  // Very small min
        10000.0, // Very large max
        5000.0,  // Large average
        Duration::from_secs(3600), // 1 hour
        50000,   // Many samples
    );
    
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    // Should handle extreme values without panicking
    assert!(callback.format_speed(0.001).contains("0.001"));
    assert!(callback.format_speed(10000.0).contains("10.0 GB/s"));
    assert_eq!(callback.format_duration(Duration::from_secs(3600)), "60m 0s");
}

/// Integration test for complete workflow
#[test]
fn test_complete_display_workflow() {
    let results = create_sample_benchmark_results();
    let callback = CliProgressCallback::new(OutputFormat::Table);
    
    // Simulate complete test workflow
    callback.on_test_start("Sequential Write");
    callback.on_progress("Sequential Write", 50.0);
    callback.on_progress("Sequential Write", 75.0);
    callback.on_progress("Sequential Write", 100.0);
    callback.on_test_complete("Sequential Write", &results.sequential_write);
    
    callback.on_test_start("Sequential Read");
    callback.on_progress("Sequential Read", 60.0);
    callback.on_test_complete("Sequential Read", &results.sequential_read);
}