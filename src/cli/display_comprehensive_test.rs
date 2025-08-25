//! Comprehensive unit tests for CLI display functionality

#[cfg(test)]
mod tests {
    use super::super::args::OutputFormat;
    use super::super::display::*;
    use disk_speed_test::platform::PlatformError;
    use disk_speed_test::{BenchmarkError, BenchmarkResults, ProgressCallback, TestResult};
    use std::io;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Create comprehensive test results with varied data
    fn create_comprehensive_test_results() -> BenchmarkResults {
        BenchmarkResults {
            sequential_write: TestResult::new(
                45.2,  // P5
                156.8, // P95
                98.7,  // avg
                Duration::from_secs(12),
                120, // samples
            ),
            sequential_read: TestResult::new(
                52.1,  // P5
                178.9, // P95
                115.3, // avg
                Duration::from_secs(11),
                110, // samples
            ),
            random_write: TestResult::new(
                8.5,  // P5
                32.7, // P95
                18.9, // avg
                Duration::from_secs(10),
                100, // samples
            ),
            random_read: TestResult::new(
                12.3, // P5
                45.6, // P95
                25.4, // avg
                Duration::from_secs(10),
                100, // samples
            ),
            memory_copy: TestResult::new(
                1250.0, // P5
                2890.5, // P95
                2156.7, // avg
                Duration::from_secs(8),
                80, // samples
            ),
        }
    }

    /// Create test results with edge case values
    fn create_edge_case_test_results() -> BenchmarkResults {
        BenchmarkResults {
            sequential_write: TestResult::new(0.001, 0.002, 0.0015, Duration::from_millis(100), 1),
            sequential_read: TestResult::new(
                15000.0,
                25000.0,
                20000.0,
                Duration::from_secs(3600),
                50000,
            ),
            random_write: TestResult::new(0.0, 0.0, 0.0, Duration::from_secs(0), 0),
            random_read: TestResult::new(
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NAN,
                Duration::from_secs(1),
                1,
            ),
            memory_copy: TestResult::new(999.99, 1000.01, 1000.0, Duration::from_millis(999), 999),
        }
    }

    #[test]
    fn test_cli_progress_callback_all_output_formats() {
        let formats = vec![OutputFormat::Table, OutputFormat::Json, OutputFormat::Csv];

        for format in formats {
            let callback = CliProgressCallback::new(format.clone());
            let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

            // These should not panic regardless of format
            callback.on_test_start("Test");
            callback.on_progress("Test", 125.5);
            callback.on_test_complete("Test", &test_result);
        }
    }

    #[test]
    fn test_cli_progress_callback_verbose_mode() {
        let callback = CliProgressCallback::new_verbose(OutputFormat::Table);
        assert!(callback.verbose);

        let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

        // Verbose mode should not cause panics
        callback.on_test_start("Verbose Test");
        callback.on_progress("Verbose Test", 125.5);
        callback.on_test_complete("Verbose Test", &test_result);
    }

    #[test]
    fn test_format_speed_edge_cases() {
        let callback = CliProgressCallback::new(OutputFormat::Table);

        // Test very small speeds
        assert!(callback.format_speed(0.0).contains("0.000"));
        assert!(callback.format_speed(0.001).contains("0.001"));
        assert!(callback.format_speed(0.999).contains("0.999"));

        // Test boundary around 1 MB/s
        assert!(callback.format_speed(0.999).contains("MB/s"));
        assert!(callback.format_speed(1.0).contains("1.00 MB/s"));
        assert!(callback.format_speed(1.001).contains("1.00 MB/s"));

        // Test boundary around 1000 MB/s (1 GB/s)
        assert!(callback.format_speed(999.9).contains("999.90 MB/s"));
        assert!(callback.format_speed(1000.0).contains("1.0 GB/s"));
        assert!(callback.format_speed(1000.1).contains("1.0 GB/s"));

        // Test very large speeds
        assert!(callback.format_speed(10000.0).contains("10.0 GB/s"));
        assert!(callback.format_speed(99999.9).contains("100.0 GB/s"));

        // Test special float values
        assert!(callback.format_speed(f64::INFINITY).contains("inf"));
        assert!(callback.format_speed(f64::NEG_INFINITY).contains("inf"));
        assert!(callback.format_speed(f64::NAN).contains("NaN"));
    }

    #[test]
    fn test_format_duration_comprehensive() {
        let callback = CliProgressCallback::new(OutputFormat::Table);

        // Test sub-second durations
        assert_eq!(callback.format_duration(Duration::from_millis(0)), "0.0s");
        assert_eq!(callback.format_duration(Duration::from_millis(500)), "0.5s");
        assert_eq!(callback.format_duration(Duration::from_millis(999)), "1.0s");

        // Test second durations
        assert_eq!(callback.format_duration(Duration::from_secs(1)), "1.0s");
        assert_eq!(callback.format_duration(Duration::from_secs(59)), "59.0s");

        // Test minute durations
        assert_eq!(callback.format_duration(Duration::from_secs(60)), "1m 0s");
        assert_eq!(callback.format_duration(Duration::from_secs(61)), "1m 1s");
        assert_eq!(callback.format_duration(Duration::from_secs(119)), "1m 59s");
        assert_eq!(callback.format_duration(Duration::from_secs(120)), "2m 0s");

        // Test longer durations
        assert_eq!(
            callback.format_duration(Duration::from_secs(3600)),
            "60m 0s"
        );
        assert_eq!(
            callback.format_duration(Duration::from_secs(3661)),
            "61m 1s"
        );

        // Test fractional seconds with minutes
        assert_eq!(
            callback.format_duration(Duration::from_millis(60500)),
            "1m 0s"
        );
        assert_eq!(
            callback.format_duration(Duration::from_millis(61500)),
            "1m 1s"
        );
    }

    #[test]
    fn test_colorize_functionality_comprehensive() {
        let mut callback = CliProgressCallback::new(OutputFormat::Table);

        // Test with colors enabled
        callback.use_colors = true;
        let colored = callback.colorize("test", "1;32");
        assert!(colored.starts_with("\x1b[1;32m"));
        assert!(colored.ends_with("\x1b[0m"));
        assert!(colored.contains("test"));

        // Test with colors disabled
        callback.use_colors = false;
        let plain = callback.colorize("test", "1;32");
        assert_eq!(plain, "test");

        // Test with empty text
        callback.use_colors = true;
        let empty_colored = callback.colorize("", "1;32");
        assert!(empty_colored.starts_with("\x1b[1;32m"));
        assert!(empty_colored.ends_with("\x1b[0m"));

        // Test with special characters
        let special = callback.colorize("!@#$%^&*()", "1;31");
        assert!(special.contains("!@#$%^&*()"));

        // Test with unicode
        let unicode = callback.colorize("测试🚀", "1;34");
        assert!(unicode.contains("测试🚀"));
    }

    #[test]
    fn test_create_progress_bar_comprehensive() {
        let callback = CliProgressCallback::new(OutputFormat::Table);

        // Test various widths
        for width in 0..=25 {
            let bar = callback.create_progress_bar(width);
            assert!(bar.starts_with('['));
            assert!(bar.ends_with(']'));

            // Bar should be reasonable length (the exact length depends on unicode vs ascii)
            assert!(bar.len() >= 2); // At least brackets
            assert!(bar.len() <= 100); // Reasonable upper bound (unicode can be longer)
        }

        // Test with colors disabled
        let mut no_color_callback = CliProgressCallback::new(OutputFormat::Table);
        no_color_callback.use_colors = false;

        let bar = no_color_callback.create_progress_bar(10);
        assert!(bar.contains('='));
        assert!(!bar.contains('█')); // Should not contain unicode blocks
    }

    #[test]
    fn test_display_results_table_format() {
        let results = create_comprehensive_test_results();

        // This should not panic
        let result = display_results(&results, &OutputFormat::Table);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_results_json_format() {
        let results = create_comprehensive_test_results();

        let result = display_results(&results, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_results_csv_format() {
        let results = create_comprehensive_test_results();

        let result = display_results(&results, &OutputFormat::Csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_results_json_comprehensive() {
        let results = create_comprehensive_test_results();

        let json_result = format_results_json(&results);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();

        // Verify JSON structure
        assert!(json.contains("timestamp"));
        assert!(json.contains("version"));
        assert!(json.contains("results"));
        assert!(json.contains("summary"));

        // Verify all test types are present
        assert!(json.contains("sequential_write"));
        assert!(json.contains("sequential_read"));
        assert!(json.contains("random_write"));
        assert!(json.contains("random_read"));
        assert!(json.contains("memory_copy"));

        // Verify specific values
        assert!(json.contains("98.7")); // sequential_write avg
        assert!(json.contains("115.3")); // sequential_read avg
        assert!(json.contains("18.9")); // random_write avg
        assert!(json.contains("25.4")); // random_read avg
        assert!(json.contains("2156.7")); // memory_copy avg

        // Verify summary calculations
        let sequential_avg = (98.7 + 115.3) / 2.0;
        let random_avg = (18.9 + 25.4) / 2.0;
        assert!(json.contains(&format!("{:.1}", sequential_avg)));
        assert!(json.contains(&format!("{:.1}", random_avg)));
        assert!(json.contains("2156.7")); // memory bandwidth

        // Verify JSON is valid
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_format_results_csv_comprehensive() {
        let results = create_comprehensive_test_results();

        let csv = format_results_csv(&results);

        // Verify CSV structure
        let lines: Vec<&str> = csv.lines().collect();
        assert!(!lines.is_empty());

        // Check header
        assert!(lines[0].contains("Test,P5 (MB/s),P95 (MB/s),Avg (MB/s),Duration (s),Samples"));

        // Check data rows
        assert!(csv.contains("Sequential Write,45.20,156.80,98.70,12.00,120"));
        assert!(csv.contains("Sequential Read,52.10,178.90,115.30,11.00,110"));
        assert!(csv.contains("Random Write,8.50,32.70,18.90,10.00,100"));
        assert!(csv.contains("Random Read,12.30,45.60,25.40,10.00,100"));
        assert!(csv.contains("Memory Copy,1250.00,2890.50,2156.70,8.00,80"));

        // Check summary section
        assert!(csv.contains("# Summary"));
        assert!(csv.contains("Sequential Average"));
        assert!(csv.contains("Random Average"));
        assert!(csv.contains("Memory Bandwidth"));
    }

    #[test]
    fn test_display_error_comprehensive() {
        let errors = vec![
            BenchmarkError::PlatformError(PlatformError::UnsupportedPlatform(
                "Test OS".to_string(),
            )),
            BenchmarkError::IoError(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Access denied",
            )),
            BenchmarkError::IoError(io::Error::new(io::ErrorKind::NotFound, "File not found")),
            BenchmarkError::IoError(io::Error::new(io::ErrorKind::AlreadyExists, "File exists")),
            BenchmarkError::IoError(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input")),
            BenchmarkError::ConfigurationError("Invalid config".to_string()),
            BenchmarkError::InsufficientSpace {
                required: 1024 * 1024 * 1024,
                available: 512 * 1024 * 1024,
            },
            BenchmarkError::PermissionDenied(PathBuf::from("/restricted/path")),
            BenchmarkError::TestInterrupted("User cancelled".to_string()),
        ];

        for error in errors {
            // These should not panic
            display_error(&error);
        }
    }

    #[test]
    fn test_display_test_result_enhanced() {
        let test_result = TestResult::new(50.0, 150.0, 100.0, Duration::from_secs(10), 100);

        // Test with colors enabled
        display_test_result_enhanced("Test", &test_result, true);

        // Test with colors disabled
        display_test_result_enhanced("Test", &test_result, false);

        // Test with edge case values
        let edge_result = TestResult::new(0.001, 10000.0, 5000.0, Duration::from_secs(3600), 50000);
        display_test_result_enhanced("Edge Case", &edge_result, true);

        // Test with zero values
        let zero_result = TestResult::new(0.0, 0.0, 0.0, Duration::from_secs(0), 0);
        display_test_result_enhanced("Zero Test", &zero_result, false);
    }

    #[test]
    fn test_display_usage_tips() {
        // This should not panic
        display_usage_tips();
    }

    #[test]
    fn test_progress_callback_with_all_test_types() {
        let callback = CliProgressCallback::new(OutputFormat::Table);
        let results = create_comprehensive_test_results();

        let test_names = [
            "Sequential Write",
            "Sequential Read",
            "Random Write",
            "Random Read",
            "Memory Copy",
        ];

        let test_results = [
            &results.sequential_write,
            &results.sequential_read,
            &results.random_write,
            &results.random_read,
            &results.memory_copy,
        ];

        for (name, result) in test_names.iter().zip(test_results.iter()) {
            callback.on_test_start(name);

            // Simulate multiple progress updates
            for i in 1..=5 {
                let progress_speed = result.avg_speed_mbps * (i as f64 / 5.0);
                callback.on_progress(name, progress_speed);
            }

            callback.on_test_complete(name, result);
        }
    }

    #[test]
    fn test_progress_callback_with_edge_case_results() {
        let callback = CliProgressCallback::new(OutputFormat::Table);
        let results = create_edge_case_test_results();

        // Test with very small values
        callback.on_test_start("Tiny Speed Test");
        callback.on_progress("Tiny Speed Test", 0.001);
        callback.on_test_complete("Tiny Speed Test", &results.sequential_write);

        // Test with very large values
        callback.on_test_start("Huge Speed Test");
        callback.on_progress("Huge Speed Test", 20000.0);
        callback.on_test_complete("Huge Speed Test", &results.sequential_read);

        // Test with zero/invalid values
        callback.on_test_start("Invalid Test");
        callback.on_progress("Invalid Test", f64::NAN);
        callback.on_test_complete("Invalid Test", &results.random_read);
    }

    #[test]
    fn test_json_format_with_edge_cases() {
        let results = create_edge_case_test_results();

        let json_result = format_results_json(&results);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();

        // Should handle special float values gracefully
        assert!(json.contains("null") || json.contains("inf") || json.contains("NaN"));

        // Should still be valid JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_csv_format_with_edge_cases() {
        let results = create_edge_case_test_results();

        let csv = format_results_csv(&results);

        // Should not panic and should produce some output
        assert!(!csv.is_empty());
        assert!(csv.contains("Test,P5 (MB/s),P95 (MB/s),Avg (MB/s),Duration (s),Samples"));

        // Should handle special values
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines.len() > 5); // Header + 5 test rows + summary
    }

    #[test]
    fn test_callback_with_unicode_test_names() {
        let callback = CliProgressCallback::new(OutputFormat::Table);
        let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

        let unicode_names = vec![
            "测试写入",
            "тест чтения",
            "🚀 Memory Test",
            "Ñoño Test",
            "العربية",
        ];

        for name in unicode_names {
            callback.on_test_start(name);
            callback.on_progress(name, 125.5);
            callback.on_test_complete(name, &test_result);
        }
    }

    #[test]
    fn test_callback_with_very_long_test_names() {
        let callback = CliProgressCallback::new(OutputFormat::Table);
        let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

        let long_name = "A".repeat(1000);

        // Should not panic with very long names
        callback.on_test_start(&long_name);
        callback.on_progress(&long_name, 125.5);
        callback.on_test_complete(&long_name, &test_result);
    }

    #[test]
    fn test_callback_with_empty_test_names() {
        let callback = CliProgressCallback::new(OutputFormat::Table);
        let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

        // Should handle empty names gracefully
        callback.on_test_start("");
        callback.on_progress("", 125.5);
        callback.on_test_complete("", &test_result);
    }

    #[test]
    fn test_display_results_with_all_zero_results() {
        let zero_results = BenchmarkResults {
            sequential_write: TestResult::default(),
            sequential_read: TestResult::default(),
            random_write: TestResult::default(),
            random_read: TestResult::default(),
            memory_copy: TestResult::default(),
        };

        // Should handle all-zero results without panicking
        let table_result = display_results(&zero_results, &OutputFormat::Table);
        assert!(table_result.is_ok());

        let json_result = display_results(&zero_results, &OutputFormat::Json);
        assert!(json_result.is_ok());

        let csv_result = display_results(&zero_results, &OutputFormat::Csv);
        assert!(csv_result.is_ok());
    }

    #[test]
    fn test_error_display_with_unicode_paths() {
        let unicode_path = PathBuf::from("/测试/路径/文件.tmp");
        let error = BenchmarkError::PermissionDenied(unicode_path);

        // Should handle unicode paths without panicking
        display_error(&error);
    }

    #[test]
    fn test_error_display_with_very_long_messages() {
        let long_message = "A".repeat(10000);
        let error = BenchmarkError::ConfigurationError(long_message);

        // Should handle very long error messages
        display_error(&error);
    }

    #[test]
    fn test_callback_thread_safety_simulation() {
        use std::sync::Arc;

        let callback = Arc::new(CliProgressCallback::new(OutputFormat::Table));
        let test_result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);

        // Simulate concurrent access (though ProgressCallback isn't actually Send+Sync)
        // This tests that the methods don't panic when called in sequence
        for i in 0..100 {
            let test_name = format!("Concurrent Test {}", i);
            callback.on_test_start(&test_name);
            callback.on_progress(&test_name, i as f64);
            callback.on_test_complete(&test_name, &test_result);
        }
    }
}
