//! Comprehensive unit tests for CLI argument parsing

#[cfg(test)]
mod tests {
    use super::super::args::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_parse_size_comprehensive() {
        // Test all supported units
        assert_eq!(parse_size("1B").unwrap(), 1);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);

        // Test short units
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);

        // Test fractional values
        assert_eq!(parse_size("0.5KB").unwrap(), 512);
        assert_eq!(
            parse_size("1.5MB").unwrap(),
            (1.5 * 1024.0 * 1024.0) as usize
        );
        assert_eq!(
            parse_size("2.25GB").unwrap(),
            (2.25 * 1024.0 * 1024.0 * 1024.0) as usize
        );

        // Test case insensitivity
        assert_eq!(parse_size("1kb").unwrap(), 1024);
        assert_eq!(parse_size("1Mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1gB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1k").unwrap(), 1024);
        assert_eq!(parse_size("1m").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1g").unwrap(), 1024 * 1024 * 1024);

        // Test with whitespace
        assert_eq!(parse_size(" 1MB ").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("\t2GB\n").unwrap(), 2 * 1024 * 1024 * 1024);

        // Test large values
        assert_eq!(parse_size("1024GB").unwrap(), 1024 * 1024 * 1024 * 1024);

        // Test zero
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("0B").unwrap(), 0);
        assert_eq!(parse_size("0KB").unwrap(), 0);
    }

    #[test]
    fn test_parse_size_error_cases() {
        let invalid_inputs = vec![
            "", "   ", "abc", "1XB", "1TB", // Not supported
            "MB1", "1.2.3MB", "-1MB", "-5", "1 MB", // Space in middle
            "MB", "KB", "1e10MB", // Scientific notation
            "∞MB",    // Unicode
            "1MB1KB", // Multiple units
        ];

        for input in invalid_inputs {
            let result = parse_size(input);
            assert!(result.is_err(), "Should have failed for input: '{}'", input);
        }
    }
    #[test]
    fn test_output_format_enum() {
        // Test default
        assert!(matches!(OutputFormat::default(), OutputFormat::Table));

        // Test all variants
        let formats = vec![OutputFormat::Table, OutputFormat::Json, OutputFormat::Csv];

        for format in formats {
            let debug_str = format!("{:?}", format);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_cli_parsing_comprehensive() {
        // Test list-devices command
        let args = vec!["disk-speed-test", "list-devices"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::ListDevices));

        // Test benchmark with all options
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
    fn test_cli_short_flags() {
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
    fn test_cli_error_cases() {
        let error_cases = vec![
            vec!["disk-speed-test"], // No command
            vec!["disk-speed-test", "invalid-command"],
            vec!["disk-speed-test", "benchmark"], // Missing target path
            vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--duration",
                "invalid",
            ],
            vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--output-format",
                "invalid",
            ],
            vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--sequential-block-size",
            ], // Missing value
        ];

        for args in error_cases {
            let result = Cli::try_parse_from(args.clone());
            assert!(result.is_err(), "Should have failed for args: {:?}", args);
        }
    }

    #[test]
    fn test_cli_help_and_version() {
        // Test help flag
        let help_result = Cli::try_parse_from(vec!["disk-speed-test", "--help"]);
        assert!(help_result.is_err()); // Help exits with error code

        // Test version flag
        let version_result = Cli::try_parse_from(vec!["disk-speed-test", "--version"]);
        assert!(version_result.is_err()); // Version exits with error code
    }

    #[test]
    fn test_parse_size_boundary_values() {
        // Test maximum values that shouldn't overflow
        assert!(parse_size("1").is_ok());
        assert!(parse_size("1023").is_ok());
        assert!(parse_size("1024").is_ok());

        // Test reasonable large values
        assert!(parse_size("1000GB").is_ok());
        assert!(parse_size("999.99GB").is_ok());

        // Test very small fractional values
        assert_eq!(parse_size("0.001KB").unwrap(), 1); // Rounds to 1 byte
        assert_eq!(
            parse_size("0.1MB").unwrap(),
            (0.1 * 1024.0 * 1024.0) as usize
        );
    }

    #[test]
    fn test_parse_size_precision() {
        // Test that fractional calculations are reasonably accurate
        let half_mb = parse_size("0.5MB").unwrap();
        assert_eq!(half_mb, 512 * 1024);

        let quarter_gb = parse_size("0.25GB").unwrap();
        assert_eq!(quarter_gb, 256 * 1024 * 1024);

        let three_quarters_kb = parse_size("0.75KB").unwrap();
        assert_eq!(three_quarters_kb, 768);
    }

    #[test]
    fn test_cli_with_various_paths() {
        let paths = vec![
            "/",
            "/tmp",
            "/home/user/test",
            ".",
            "./test",
            "../test",
            "test",
            "relative/path",
            "/path with spaces",
            "/path/with/unicode/测试",
        ];

        for path in paths {
            let args = vec!["disk-speed-test", "benchmark", path];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse path: {}", path);

            match result.unwrap().command {
                Commands::Benchmark { target_path, .. } => {
                    assert_eq!(target_path, PathBuf::from(path));
                }
                _ => panic!("Expected Benchmark command"),
            }
        }
    }

    #[test]
    fn test_cli_with_extreme_values() {
        // Test with very large duration
        let args = vec![
            "disk-speed-test",
            "benchmark",
            "/tmp",
            "--duration",
            "86400",
        ]; // 24 hours
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok());

        // Test with very small duration
        let args = vec!["disk-speed-test", "benchmark", "/tmp", "--duration", "1"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok());

        // Test with zero duration (should be parsed but might be invalid in validation)
        let args = vec!["disk-speed-test", "benchmark", "/tmp", "--duration", "0"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_format_case_sensitivity() {
        // Test only lowercase formats since clap is case-sensitive by default
        let format_cases = vec!["table", "json", "csv"];

        for format_str in format_cases {
            let args = vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--output-format",
                format_str,
            ];
            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed to parse format: {}", format_str);
        }

        // Test that uppercase formats fail (case-sensitive)
        let invalid_formats = vec!["TABLE", "JSON", "CSV", "Table", "Json", "Csv"];
        for format_str in invalid_formats {
            let args = vec![
                "disk-speed-test",
                "benchmark",
                "/tmp",
                "--output-format",
                format_str,
            ];
            let result = Cli::try_parse_from(args);
            assert!(
                result.is_err(),
                "Should have failed for format: {}",
                format_str
            );
        }
    }

    #[test]
    fn test_cli_option_combinations() {
        // Test all possible combinations of boolean flags
        let flag_combinations = vec![vec![], vec!["--enable-cache"]];

        for flags in flag_combinations {
            let mut args = vec!["disk-speed-test", "benchmark", "/tmp"];
            args.extend(flags.iter());

            let result = Cli::try_parse_from(args);
            assert!(result.is_ok(), "Failed with flags: {:?}", flags);
        }
    }

    #[test]
    fn test_parse_size_with_different_locales() {
        // Test with different decimal separators (though Rust parsing uses '.')
        let valid_cases = vec![
            ("1.5MB", (1.5 * 1024.0 * 1024.0) as usize),
            ("2.0GB", (2.0 * 1024.0 * 1024.0 * 1024.0) as usize),
            ("0.5KB", 512),
        ];

        for (input, expected) in valid_cases {
            let result = parse_size(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }

        // Test invalid decimal formats
        let invalid_cases = vec![
            "1,5MB",   // Comma separator
            "1.5.0MB", // Multiple decimals
            "1..5MB",  // Double decimal
        ];

        for input in invalid_cases {
            let result = parse_size(input);
            assert!(result.is_err(), "Should have failed for: {}", input);
        }
    }
}
