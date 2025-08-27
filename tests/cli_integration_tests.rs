//! CLI integration tests for complete command-line workflows
//!
//! These tests verify that the CLI interface works correctly with all
//! command combinations, error handling, and output formatting.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.join("disk-speed-test")
}

/// Create a temporary directory for testing
fn create_temp_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

#[test]
fn test_cli_help_command() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify help contains expected sections
    assert!(stdout.contains("cross-platform disk speed testing utility"));
    assert!(stdout.contains("list-devices"));
    assert!(stdout.contains("benchmark"));
}

#[test]
fn test_cli_benchmark_help_command() {
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Benchmark help command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify benchmark help contains expected options
    assert!(stdout.contains("--sequential-block-size"));
    assert!(stdout.contains("--random-block-size"));
    assert!(stdout.contains("--duration"));
    assert!(stdout.contains("--file-size"));
    assert!(stdout.contains("--enable-cache"));
    assert!(stdout.contains("--output-format"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Version command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("disk-speed-test"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_list_devices_command() {
    let output = Command::new(get_binary_path())
        .arg("list-devices")
        .output()
        .expect("Failed to execute command");

    // Command should succeed even if device enumeration is not implemented
    assert!(
        output.status.success(),
        "List devices command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Available storage devices")
            || stdout.contains("device enumeration not yet implemented")
    );
}

#[test]
fn test_cli_benchmark_basic() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("1") // Short test
        .arg("--file-size")
        .arg("1MB") // Small file
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Basic benchmark should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify output contains expected test results
    assert!(stdout.contains("Sequential Write"));
    assert!(stdout.contains("Sequential Read"));
    assert!(stdout.contains("Random Write"));
    assert!(stdout.contains("Random Read"));
    assert!(stdout.contains("Memory Copy"));
    assert!(stdout.contains("MB/s"));
}

#[test]
fn test_cli_benchmark_with_custom_parameters() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--sequential-block-size")
        .arg("1MB")
        .arg("--random-block-size")
        .arg("8KB")
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("2MB")
        .arg("--enable-cache")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Custom parameter benchmark should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify configuration is displayed
    assert!(stdout.contains("Benchmark Configuration"));
    assert!(stdout.contains("Sequential block size: 1 MB"));
    assert!(stdout.contains("Random block size: 8 KB"));
    assert!(stdout.contains("Test duration: 1 seconds"));
    assert!(stdout.contains("Test file size: 2 MB"));
    assert!(stdout.contains("OS caching: enabled"));
}

#[test]
fn test_cli_benchmark_json_output() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("1MB")
        .arg("--output-format")
        .arg("json")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "JSON output benchmark should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify JSON structure
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    let json = json_result.unwrap();
    assert!(json["results"]["sequential_write"]["avg_speed_mbps"].is_number());
    assert!(json["results"]["sequential_read"]["avg_speed_mbps"].is_number());
    assert!(json["results"]["random_write"]["avg_speed_mbps"].is_number());
    assert!(json["results"]["random_read"]["avg_speed_mbps"].is_number());
    assert!(json["results"]["memory_copy"]["avg_speed_mbps"].is_number());
    assert!(json["timestamp"].is_number());
    assert!(json["version"].is_string());
}

#[test]
fn test_cli_benchmark_csv_output() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("1MB")
        .arg("--output-format")
        .arg("csv")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "CSV output benchmark should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify CSV structure
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(!lines.is_empty(), "CSV output should not be empty");

    // Should have header (P5 and P95)
    assert!(lines[0].contains("Test,P5 (MB/s),P95 (MB/s),Avg (MB/s)"));

    // Should have data rows for each test
    assert!(lines
        .iter()
        .any(|line| line.starts_with("Sequential Write,")));
    assert!(lines
        .iter()
        .any(|line| line.starts_with("Sequential Read,")));
    assert!(lines.iter().any(|line| line.starts_with("Random Write,")));
    assert!(lines.iter().any(|line| line.starts_with("Random Read,")));
    assert!(lines.iter().any(|line| line.starts_with("Memory Copy,")));
}

#[test]
fn test_cli_error_handling_invalid_path() {
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg("/nonexistent/path/that/does/not/exist")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with invalid path");
    assert_eq!(output.status.code(), Some(1), "Should exit with code 1");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Configuration"));
}

#[test]
fn test_cli_error_handling_invalid_size() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--file-size")
        .arg("invalid_size")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with invalid size");
    assert_eq!(output.status.code(), Some(1), "Should exit with code 1");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Invalid"));
}

#[test]
fn test_cli_error_handling_invalid_duration() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("0") // Invalid duration
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail with zero duration");
    assert_eq!(output.status.code(), Some(1), "Should exit with code 1");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Configuration"));
}

#[test]
fn test_cli_argument_parsing() {
    // Test various argument combinations to ensure parsing works correctly
    // Use non-existent path to fail quickly at validation, not during benchmark execution

    let test_cases = [
        // Basic case
        vec![
            "benchmark",
            "/nonexistent/test/path",
            "--duration",
            "1",
            "--file-size",
            "1MB",
        ],
        // With all optional parameters
        vec![
            "benchmark",
            "/nonexistent/test/path",
            "--sequential-block-size",
            "1MB",
            "--random-block-size",
            "8KB",
            "--duration",
            "1",
            "--file-size",
            "1MB",
            "--enable-cache",
            "--output-format",
            "table",
        ],
        // Short flags where available
        vec![
            "benchmark",
            "/nonexistent/test/path",
            "-d",
            "1",
            "-o",
            "json",
        ],
    ];

    for (i, args) in test_cases.iter().enumerate() {
        let output = Command::new(get_binary_path())
            .args(args)
            .output()
            .expect("Failed to execute command");

        // Should fail due to nonexistent path, not due to argument parsing
        assert!(
            !output.status.success(),
            "Should fail due to nonexistent path"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should be a path error, not an argument parsing error
        assert!(
            stderr.contains("does not exist") || stderr.contains("Configuration"),
            "Test case {} should fail on path validation, not argument parsing: {:?}. Stderr: {}",
            i,
            args,
            stderr
        );
    }
}

#[test]
fn test_cli_size_parsing() {
    // Test size parsing by checking argument validation without running full benchmarks
    // This tests that the CLI accepts various size formats correctly

    // Test invalid size format should fail quickly (argument parsing error)
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(".")
        .arg("--file-size")
        .arg("invalid_size")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Invalid size format should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Invalid"));

    // Test that valid size formats are accepted by checking they don't fail at argument parsing
    // We'll use a non-existent path so it fails quickly at validation, not during benchmark
    let size_formats = vec!["1MB", "512KB", "1024", "2MB", "4KB", "8MB"];

    for size in size_formats {
        let output = Command::new(get_binary_path())
            .arg("benchmark")
            .arg("/nonexistent/test/path") // This will fail quickly at path validation
            .arg("--file-size")
            .arg(size)
            .arg("--duration")
            .arg("1")
            .output()
            .expect("Failed to execute command");

        // Should fail due to nonexistent path, not due to size parsing
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should be a path error, not a size parsing error
        assert!(
            stderr.contains("does not exist") || stderr.contains("Configuration"),
            "Size format '{}' should parse correctly but fail on path. Stderr: {}",
            size,
            stderr
        );
    }
}

#[test]
fn test_cli_configuration_display() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--sequential-block-size")
        .arg("1MB")
        .arg("--random-block-size")
        .arg("8KB")
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("1MB")
        .arg("--enable-cache")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify configuration is properly displayed
    assert!(stdout.contains("Benchmark Configuration:"));
    assert!(stdout.contains("Target path:"));
    assert!(stdout.contains("Sequential block size: 1 MB"));
    assert!(stdout.contains("Random block size: 8 KB"));
    assert!(stdout.contains("Test duration: 1 seconds"));
    assert!(stdout.contains("Test file size: 1 MB"));
    assert!(stdout.contains("OS caching: enabled"));
}

#[test]
fn test_cli_progress_display() {
    let temp_dir = create_temp_test_dir();

    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("1MB")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .env("DISK_SPEED_TEST_FAST_TEST_MS", "50")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify progress and results are displayed
    assert!(stdout.contains("Starting benchmark tests"));
    assert!(stdout.contains("BENCHMARK RESULTS") || stdout.contains("Sequential Write"));

    // Should show completion indicators
    assert!(stdout.contains("complete") || stdout.contains("✓"));
}

#[test]
fn test_cli_exit_codes() {
    let temp_dir = create_temp_test_dir();

    // Test successful execution (exit code 0)
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(temp_dir.path())
        .arg("--duration")
        .arg("1")
        .arg("--file-size")
        .arg("1MB")
        .arg("--disable-direct-io") // Use buffered I/O for compatibility
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(0),
        "Successful benchmark should exit with code 0"
    );

    // Test configuration error (exit code 1)
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg("/nonexistent/path")
        .output()
        .expect("Failed to execute command");

    assert_eq!(
        output.status.code(),
        Some(1),
        "Configuration error should exit with code 1"
    );

    // Test invalid arguments (should be handled by clap)
    let output = Command::new(get_binary_path())
        .arg("invalid-command")
        .output()
        .expect("Failed to execute command");

    assert_ne!(
        output.status.code(),
        Some(0),
        "Invalid command should not succeed"
    );
}

#[test]
fn test_cli_output_format_consistency() {
    // Test that all output formats are accepted by the CLI
    // We already have individual tests for each format that run full benchmarks
    // This test just verifies the format arguments are parsed correctly

    let formats = vec!["table", "json", "csv"];

    for format in formats {
        let output = Command::new(get_binary_path())
            .arg("benchmark")
            .arg("/nonexistent/test/path") // Will fail quickly at path validation
            .arg("--duration")
            .arg("1")
            .arg("--file-size")
            .arg("1MB")
            .arg("--output-format")
            .arg(format)
            .output()
            .expect("Failed to execute command");

        // Should fail due to nonexistent path, not due to format parsing
        assert!(
            !output.status.success(),
            "Should fail due to nonexistent path"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should be a path error, not a format parsing error
        assert!(
            stderr.contains("does not exist") || stderr.contains("Configuration"),
            "Format '{}' should parse correctly but fail on path. Stderr: {}",
            format,
            stderr
        );
    }

    // Test invalid format should fail at argument parsing
    let output = Command::new(get_binary_path())
        .arg("benchmark")
        .arg(".")
        .arg("--output-format")
        .arg("invalid_format")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Invalid format should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value") || stderr.contains("possible values"),
        "Should be a format parsing error. Stderr: {}",
        stderr
    );
}
