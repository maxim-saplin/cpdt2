//! Tests for CLI argument parsing

#[cfg(test)]
mod tests {
    use super::super::args::{parse_size, Cli, Commands, OutputFormat};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_list_devices_command() {
        let args = vec!["disk-speed-test", "list-devices"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ListDevices => {
                // Success - this is what we expect
            }
            _ => panic!("Expected ListDevices command"),
        }
    }

    #[test]
    fn test_benchmark_command_basic() {
        let args = vec!["disk-speed-test", "benchmark", "/tmp"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark { target_path, .. } => {
                assert_eq!(target_path, PathBuf::from("/tmp"));
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_benchmark_command_with_options() {
        let args = vec![
            "disk-speed-test",
            "benchmark",
            "/tmp",
            "--sequential-block-size",
            "8MB",
            "--random-block-size",
            "8KB",
            "--duration",
            "30",
            "--file-size",
            "2GB",
            "--enable-cache",
            "--output-format",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark {
                target_path,
                sequential_block_size,
                random_block_size,
                duration,
                file_size,
                enable_cache,
                disable_direct_io: _,
                output_format,
            } => {
                assert_eq!(target_path, PathBuf::from("/tmp"));
                assert_eq!(sequential_block_size, Some("8MB".to_string()));
                assert_eq!(random_block_size, Some("8KB".to_string()));
                assert_eq!(duration, Some(30));
                assert_eq!(file_size, Some("2GB".to_string()));
                assert!(enable_cache);
                assert!(matches!(output_format, OutputFormat::Json));
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_benchmark_command_defaults() {
        let args = vec!["disk-speed-test", "benchmark", "/tmp"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark {
                sequential_block_size,
                random_block_size,
                duration,
                file_size,
                enable_cache,
                output_format,
                ..
            } => {
                assert_eq!(sequential_block_size, None);
                assert_eq!(random_block_size, None);
                assert_eq!(duration, None);
                assert_eq!(file_size, None);
                assert!(!enable_cache); // Default is false (cache disabled)
                assert!(matches!(output_format, OutputFormat::Table));
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_help_generation() {
        // Test that help can be generated without panicking
        let result = Cli::try_parse_from(vec!["disk-speed-test", "--help"]);
        assert!(result.is_err()); // Help exits with error code

        // The error should contain help text
        let error = result.unwrap_err();
        let help_text = error.to_string();
        assert!(help_text.contains("disk speed testing utility"));
        assert!(help_text.contains("list-devices"));
        assert!(help_text.contains("benchmark"));
    }

    #[test]
    fn test_version_flag() {
        let result = Cli::try_parse_from(vec!["disk-speed-test", "--version"]);
        assert!(result.is_err()); // Version exits with error code

        let error = result.unwrap_err();
        let version_text = error.to_string();
        assert!(version_text.contains("0.1.0")); // Should contain version from Cargo.toml
    }

    #[test]
    fn test_invalid_command() {
        let result = Cli::try_parse_from(vec!["disk-speed-test", "invalid-command"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_target_path() {
        let result = Cli::try_parse_from(vec!["disk-speed-test", "benchmark"]);
        assert!(result.is_err()); // Should require target path
    }

    #[test]
    fn test_invalid_output_format() {
        let result = Cli::try_parse_from(vec![
            "disk-speed-test",
            "benchmark",
            "/tmp",
            "--output-format",
            "invalid",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_short_flags() {
        let args = vec![
            "disk-speed-test",
            "benchmark",
            "/tmp",
            "-d",
            "15",
            "-o",
            "csv",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark {
                duration,
                output_format,
                ..
            } => {
                assert_eq!(duration, Some(15));
                assert!(matches!(output_format, OutputFormat::Csv));
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_parse_size_function() {
        // Test basic sizes
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);

        // Test different cases
        assert_eq!(parse_size("1kb").unwrap(), 1024);
        assert_eq!(parse_size("1Mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1gB").unwrap(), 1024 * 1024 * 1024);

        // Test short units
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);

        // Test with spaces
        assert_eq!(parse_size(" 1MB ").unwrap(), 1024 * 1024);

        // Test fractional values
        assert_eq!(parse_size("0.5MB").unwrap(), 512 * 1024);
        assert_eq!(
            parse_size("1.5GB").unwrap(),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as usize
        );

        // Test invalid formats
        assert!(parse_size("invalid").is_err());
        assert!(parse_size("1XB").is_err());
        assert!(parse_size("").is_err());
        assert!(parse_size("MB").is_err());
    }

    #[test]
    fn test_output_format_values() {
        // Test that all output format values are accepted
        let formats = vec!["table", "json", "csv"];

        for format in formats {
            let args = vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--output-format",
                format,
            ];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse output format: {}", format);
        }
    }

    #[test]
    fn test_configuration_validation() {
        // Test various size configurations to ensure they parse correctly
        let test_cases = vec![
            ("--sequential-block-size", "8MB"),
            ("--random-block-size", "8KB"),
            ("--file-size", "2GB"),
            ("--file-size", "500MB"),
            ("--sequential-block-size", "1024"),
            ("--random-block-size", "4096"),
        ];

        for (flag, value) in test_cases {
            let args = vec!["disk-speed-test", "benchmark", "/tmp", flag, value];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse {} {}", flag, value);
        }
    }

    #[test]
    fn test_duration_validation() {
        // Test duration parameter validation
        let valid_durations = vec!["1", "10", "30", "60", "300"];

        for duration in valid_durations {
            let args = vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--duration",
                duration,
            ];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse duration: {}", duration);
        }
    }

    #[test]
    fn test_all_options_together() {
        // Test that all options can be used together
        let args = vec![
            "disk-speed-test",
            "benchmark",
            "/tmp/test",
            "--sequential-block-size",
            "8MB",
            "--random-block-size",
            "8KB",
            "--duration",
            "30",
            "--file-size",
            "2GB",
            "--enable-cache",
            "--output-format",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark {
                target_path,
                sequential_block_size,
                random_block_size,
                duration,
                file_size,
                enable_cache,
                disable_direct_io: _,
                output_format,
            } => {
                assert_eq!(target_path, PathBuf::from("/tmp/test"));
                assert_eq!(sequential_block_size, Some("8MB".to_string()));
                assert_eq!(random_block_size, Some("8KB".to_string()));
                assert_eq!(duration, Some(30));
                assert_eq!(file_size, Some("2GB".to_string()));
                assert!(enable_cache);
                assert!(matches!(output_format, OutputFormat::Json));
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_relative_and_absolute_paths() {
        // Test that both relative and absolute paths are accepted
        let paths = vec![
            "/tmp",
            "/home/user/test",
            ".",
            "./test",
            "../test",
            "test",
            "/",
        ];

        for path in paths {
            let args = vec!["disk-speed-test", "benchmark", path];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse path: {}", path);
        }
    }

    #[test]
    fn test_case_insensitive_size_parsing() {
        // Test that size parsing is case insensitive
        let size_variants = vec![
            ("1mb", 1024 * 1024),
            ("1MB", 1024 * 1024),
            ("1Mb", 1024 * 1024),
            ("1mB", 1024 * 1024),
            ("2gb", 2 * 1024 * 1024 * 1024),
            ("2GB", 2 * 1024 * 1024 * 1024),
            ("512kb", 512 * 1024),
            ("512KB", 512 * 1024),
        ];

        for (input, expected) in size_variants {
            let result = parse_size(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_edge_case_sizes() {
        // Test edge cases for size parsing
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("1B").unwrap(), 1);
        assert_eq!(parse_size("1023B").unwrap(), 1023);

        // Test very large sizes
        assert_eq!(parse_size("1024GB").unwrap(), 1024 * 1024 * 1024 * 1024);

        // Test fractional sizes
        assert_eq!(parse_size("0.5KB").unwrap(), 512);
        assert_eq!(
            parse_size("2.5MB").unwrap(),
            (2.5 * 1024.0 * 1024.0) as usize
        );
    }

    #[test]
    fn test_invalid_size_formats() {
        // Test various invalid size formats
        let invalid_sizes = vec![
            "", "abc", "1XB", "1TB", // Not supported
            "MB1", "1.2.3MB", "-1MB",
        ];

        for invalid_size in invalid_sizes {
            let result = parse_size(invalid_size);
            assert!(result.is_err(), "Should have failed for: {}", invalid_size);
        }
    }

    #[test]
    fn test_size_with_spaces() {
        // Test that spaces are handled correctly (trimmed at start/end, but not in middle)
        assert_eq!(parse_size(" 1MB ").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("  2GB  ").unwrap(), 2 * 1024 * 1024 * 1024);

        // Space in middle should still work since we trim the whole string first
        // but the number parsing will handle it appropriately
        let result = parse_size("1 MB");
        // This should fail because "1 " is not a valid number
        assert!(result.is_err());
    }
}
