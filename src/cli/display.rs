//! Display and output formatting for CLI

use crate::cli::args::OutputFormat;
use anyhow::Result;
use disk_speed_test::{BenchmarkError, BenchmarkResults, ProgressCallback, TestResult};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// CLI progress callback implementation
///
/// This implementation provides real-time progress display for the command line interface,
/// showing test names and current speeds with proper formatting according to requirements
/// 10.1, 10.2, 10.3, 10.4, 10.5, and 10.6.
#[derive(Debug)]
pub struct CliProgressCallback {
    /// Whether to use colored output (if terminal supports it)
    pub use_colors: bool,
    /// Whether to show verbose progress information
    pub verbose: bool,
    /// Output format for results display
    output_format: OutputFormat,
    /// Track if we're currently showing progress (for proper line clearing)
    showing_progress: Arc<Mutex<bool>>,
}

impl CliProgressCallback {
    /// Create a new CLI progress callback with specified output format
    pub fn new(output_format: OutputFormat) -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            verbose: false,
            output_format,
            showing_progress: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new CLI progress callback with verbose output enabled
    #[allow(dead_code)]
    pub fn new_verbose(output_format: OutputFormat) -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            verbose: true,
            output_format,
            showing_progress: Arc::new(Mutex::new(false)),
        }
    }

    /// Format speed with appropriate precision and units
    pub fn format_speed(&self, speed_mbps: f64) -> String {
        if speed_mbps >= 1000.0 {
            format!("{:.1} GB/s", speed_mbps / 1000.0)
        } else if speed_mbps >= 1.0 {
            format!("{:.2} MB/s", speed_mbps)
        } else {
            format!("{:.3} MB/s", speed_mbps)
        }
    }

    /// Apply color formatting if colors are enabled
    pub fn colorize(&self, text: &str, color_code: &str) -> String {
        if self.use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    }

    /// Clear the current line (for progress updates)
    fn clear_line(&self) {
        if let Ok(mut showing) = self.showing_progress.lock() {
            if *showing {
                if self.use_colors {
                    print!("\r\x1b[K");
                } else {
                    print!("\r{}\r", " ".repeat(80)); // Clear with spaces, then return to start
                }
                *showing = false;
            }
        }
    }

    /// Format duration in a human-readable way
    pub fn format_duration(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        if total_secs >= 60 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{}m {}s", mins, secs)
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        }
    }

    /// Create a progress bar string
    #[allow(dead_code)]
    pub fn create_progress_bar(&self, width: usize) -> String {
        if !self.use_colors {
            return format!("[{}]", "=".repeat(width.min(20)));
        }

        let filled = "█".repeat(width.min(20));
        let empty = "░".repeat(20_usize.saturating_sub(width));
        format!("[{}{}]", filled, empty)
    }
}

impl Default for CliProgressCallback {
    fn default() -> Self {
        Self::new(OutputFormat::Table)
    }
}

impl ProgressCallback for CliProgressCallback {
    fn on_test_start(&self, test_name: &str) {
        // Requirements 10.1, 10.2, 10.3, 10.4, 10.5: Display test name when starting
        let colored_name = self.colorize(test_name, "1;36"); // Bright cyan

        // Only show detailed start message for table format
        match self.output_format {
            OutputFormat::Table => {
                println!(
                    "\n{} Starting {}...",
                    self.colorize("▶", "1;32"), // Green arrow
                    colored_name
                );

                if self.verbose {
                    println!(
                        "  {} Initializing test parameters and creating test files...",
                        self.colorize("ℹ", "1;34")
                    ); // Blue info
                }
            }
            OutputFormat::Json | OutputFormat::Csv => {
                // For structured output formats, show minimal progress
                if self.verbose {
                    eprintln!("Starting {}...", test_name);
                }
            }
        }

        // Ensure output is flushed
        io::stdout().flush().unwrap_or(());
    }

    fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
        // Requirements 10.1, 10.2, 10.3, 10.4, 10.5: Show current test name and speed

        // Only show progress for table format to avoid interfering with structured output
        match self.output_format {
            OutputFormat::Table => {
                self.clear_line();

                let colored_name = self.colorize(test_name, "1;33"); // Bright yellow
                let formatted_speed = self.format_speed(current_speed_mbps);
                let colored_speed = self.colorize(&formatted_speed, "1;32"); // Bright green

                // Create a simple progress indicator
                let progress_indicator = if self.use_colors {
                    self.colorize("●", "1;32") // Green dot
                } else {
                    "*".to_string()
                };

                print!(
                    "  {} {}: {}",
                    progress_indicator, colored_name, colored_speed
                );

                if self.verbose {
                    print!(" {}", self.colorize("(real-time)", "37")); // Light gray
                }

                if let Ok(mut showing) = self.showing_progress.lock() {
                    *showing = true;
                }
            }
            OutputFormat::Json | OutputFormat::Csv => {
                // For structured formats, only show progress if verbose and to stderr
                if self.verbose {
                    eprintln!("{}: {:.2} MB/s", test_name, current_speed_mbps);
                }
            }
        }

        // Ensure output is flushed for real-time display
        io::stdout().flush().unwrap_or(());
    }

    fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        // Requirement 10.6: Show P5 (low-percentile), Max, and average (bold) speeds when test completes

        match self.output_format {
            OutputFormat::Table => {
                self.clear_line();

                let colored_name = self.colorize(test_name, "1;32"); // Bright green
                let min_speed = self.format_speed(result.min_speed_mbps);
                let max_speed = self.format_speed(result.max_speed_mbps);
                let avg_speed = self.format_speed(result.avg_speed_mbps);

                // Make average speed bold as per requirement
                let bold_avg = self.colorize(&avg_speed, "1"); // Bold

                let checkmark = if self.use_colors {
                    self.colorize("✓", "1;32") // Green checkmark
                } else {
                    "✓".to_string()
                };

                println!("  {} {} complete", checkmark, colored_name);
                println!(
                    "    Min (P5): {} | Max (P95): {} | Avg: {}",
                    self.colorize(&min_speed, "37"), // Light gray
                    self.colorize(&max_speed, "37"), // Light gray
                    bold_avg
                );

                if self.verbose {
                    let duration_str = self.format_duration(result.test_duration);
                    println!(
                        "    {} Duration: {}, Samples: {}",
                        self.colorize("ℹ", "1;34"), // Blue info
                        self.colorize(&duration_str, "37"),
                        self.colorize(&result.sample_count.to_string(), "37")
                    );
                }
            }
            OutputFormat::Json | OutputFormat::Csv => {
                // For structured formats, only show completion if verbose and to stderr
                if self.verbose {
                    eprintln!(
                        "{} complete: avg {:.2} MB/s",
                        test_name, result.avg_speed_mbps
                    );
                }
            }
        }

        // Ensure output is flushed
        io::stdout().flush().unwrap_or(());
    }
}

/// Display benchmark results in the specified format
pub fn display_results(results: &BenchmarkResults, output_format: &OutputFormat) -> Result<()> {
    match output_format {
        OutputFormat::Table => {
            display_results_table(results);
        }
        OutputFormat::Json => {
            let json_output = format_results_json(results)?;
            println!("{}", json_output);
        }
        OutputFormat::Csv => {
            let csv_output = format_results_csv(results);
            println!("{}", csv_output);
        }
    }
    Ok(())
}

/// Display error messages with helpful diagnostic information
/// Requirement 11.4: Report errors with helpful diagnostic information
pub fn display_error(error: &BenchmarkError) {
    let use_colors = atty::is(atty::Stream::Stderr);

    let colorize = |text: &str, color_code: &str| -> String {
        if use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    };

    let error_prefix = colorize("Error:", "1;31"); // Bold red
    let warning_prefix = colorize("Warning:", "1;33"); // Bold yellow
    let info_prefix = colorize("Info:", "1;34"); // Bold blue

    match error {
        BenchmarkError::PlatformError(e) => {
            eprintln!("{} Platform-specific operation failed: {}", error_prefix, e);
            eprintln!(
                "{} This may be due to insufficient permissions or unsupported hardware.",
                info_prefix
            );
            eprintln!("  Try running with administrator/root privileges or on a different device.");
        }
        BenchmarkError::IoError(e) => {
            eprintln!("{} I/O operation failed: {}", error_prefix, e);
            match e.kind() {
                io::ErrorKind::PermissionDenied => {
                    eprintln!(
                        "{} Permission denied accessing the target path.",
                        info_prefix
                    );
                    eprintln!("  Try running with administrator/root privileges or choose a different path.");
                }
                io::ErrorKind::NotFound => {
                    eprintln!("{} Target path not found.", info_prefix);
                    eprintln!("  Verify the path exists and is accessible.");
                }
                io::ErrorKind::AlreadyExists => {
                    eprintln!("{} Test file already exists.", warning_prefix);
                    eprintln!("  This may indicate a previous test didn't clean up properly.");
                }
                io::ErrorKind::InvalidInput => {
                    eprintln!("{} Invalid input parameters.", info_prefix);
                    eprintln!("  Check your block sizes and file size parameters.");
                }
                _ => {
                    eprintln!(
                        "{} Check disk space, permissions, and path accessibility.",
                        info_prefix
                    );
                }
            }
        }
        BenchmarkError::ConfigurationError(msg) => {
            eprintln!("{} Configuration error: {}", error_prefix, msg);
            eprintln!(
                "{} Check your command line arguments and try again.",
                info_prefix
            );
            eprintln!("  Use --help to see available options and their formats.");
        }
        BenchmarkError::InsufficientSpace {
            required,
            available,
        } => {
            eprintln!("{} Insufficient disk space for test file.", error_prefix);
            eprintln!(
                "  Required: {:.2} GB",
                *required as f64 / (1024.0 * 1024.0 * 1024.0)
            );
            eprintln!(
                "  Available: {:.2} GB",
                *available as f64 / (1024.0 * 1024.0 * 1024.0)
            );
            eprintln!(
                "{} Try using a smaller file size with --file-size option.",
                info_prefix
            );
            eprintln!("  Example: --file-size 100MB");
        }
        BenchmarkError::PermissionDenied(path) => {
            eprintln!(
                "{} Permission denied accessing: {}",
                error_prefix,
                path.display()
            );
            eprintln!("{} Try one of the following:", info_prefix);
            eprintln!("  • Run with administrator/root privileges");
            eprintln!("  • Choose a different target path you have write access to");
            eprintln!("  • Use the list-devices command to find suitable locations");
        }
        BenchmarkError::TestInterrupted(msg) => {
            eprintln!("{} Test was interrupted: {}", warning_prefix, msg);
            eprintln!(
                "{} Partial results may be incomplete or inaccurate.",
                info_prefix
            );
            eprintln!("  Try running the test again with stable system conditions.");
        }
    }
}

/// Display benchmark results as a formatted table
/// Requirements 11.1, 11.2, 11.3: Clear table showing all test results with P5, Max, and bold average speeds
fn display_results_table(results: &BenchmarkResults) {
    let use_colors = atty::is(atty::Stream::Stdout);

    let colorize = |text: &str, color_code: &str| -> String {
        if use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    };

    // Header with styling
    println!(
        "\n{}",
        colorize(
            "╔══════════════════════════════════════════════════════════════════╗",
            "1;36"
        )
    );
    println!(
        "{}",
        colorize(
            "║                        BENCHMARK RESULTS                        ║",
            "1;36"
        )
    );
    println!(
        "{}",
        colorize(
            "╚══════════════════════════════════════════════════════════════════╝",
            "1;36"
        )
    );
    println!();

    // Table header with better formatting (match row widths)
    let h_test = colorize(&format!("{:<30}", "Test"), "1;37");
    let h_min = colorize(&format!("{:>12}", "Min (MB/s)"), "37");
    let h_max = colorize(&format!("{:>12}", "Max (MB/s)"), "37");
    let h_avg = colorize(&format!("{:>12}", "Avg (MB/s)"), "1;33");
    println!("{} {} {} {}", h_test, h_min, h_max, h_avg);

    // Separator length matches visible table width: 30 + 12 + 12 + 12 + 3 spaces = 69
    let separator_width = 69;
    let separator = if use_colors {
        colorize(&"─".repeat(separator_width), "37")
    } else {
        "-".repeat(separator_width)
    };
    println!("{}", separator);

    // Format each test result with enhanced display
    display_test_result_enhanced("Sequential Write", &results.sequential_write, use_colors);
    display_test_result_enhanced("Sequential Read", &results.sequential_read, use_colors);
    display_test_result_enhanced("Random Write", &results.random_write, use_colors);
    display_test_result_enhanced("Random Read", &results.random_read, use_colors);
    display_test_result_enhanced("Memory Copy", &results.memory_copy, use_colors);

    println!();

    // Add summary information
    let avg_sequential =
        (results.sequential_write.avg_speed_mbps + results.sequential_read.avg_speed_mbps) / 2.0;
    let avg_random =
        (results.random_write.avg_speed_mbps + results.random_read.avg_speed_mbps) / 2.0;

    println!("{}", colorize("Summary:", "1;36"));
    println!("  Sequential Average: {:.2} MB/s", avg_sequential);
    println!("  Random Average: {:.2} MB/s", avg_random);
    println!(
        "  Memory Bandwidth: {:.2} MB/s",
        results.memory_copy.avg_speed_mbps
    );

    // Performance indicators
    if avg_sequential > 500.0 {
        println!(
            "  {} Excellent sequential performance (SSD-class)",
            colorize("✓", "1;32")
        );
    } else if avg_sequential > 100.0 {
        println!("  {} Good sequential performance", colorize("✓", "1;33"));
    } else {
        println!(
            "  {} Consider upgrading storage for better performance",
            colorize("ℹ", "1;34")
        );
    }

    if avg_random > 50.0 {
        println!(
            "  {} Excellent random access performance",
            colorize("✓", "1;32")
        );
    } else if avg_random > 10.0 {
        println!(
            "  {} Moderate random access performance",
            colorize("✓", "1;33")
        );
    }

    println!();
}

/// Display a single test result row with enhanced formatting
pub fn display_test_result_enhanced(test_name: &str, result: &TestResult, use_colors: bool) {
    let colorize = |text: &str, color_code: &str| -> String {
        if use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    };

    let _format_duration = |duration: Duration| -> String {
        let total_secs = duration.as_secs();
        if total_secs >= 60 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{}m{}s", mins, secs)
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        }
    };

    // Pad the test name BEFORE applying color so visible width matches the column
    let padded_test_name = format!("{:<30}", test_name);
    let colored_test_name = if result.avg_speed_mbps > 100.0 {
        colorize(&padded_test_name, "1;32") // Green for good performance
    } else if result.avg_speed_mbps > 10.0 {
        colorize(&padded_test_name, "1;33") // Yellow for moderate performance
    } else if result.avg_speed_mbps > 0.0 {
        colorize(&padded_test_name, "1;31") // Red for poor performance
    } else {
        colorize(&padded_test_name, "1;90") // Dark gray for failed tests
    };

    // Make average speed bold as per requirement 11.2
    let bold_avg = colorize(&format!("{:>12.2}", result.avg_speed_mbps), "1");

    println!(
        "{} {:>12.2} {:>12.2} {}",
        colored_test_name,
        result.min_speed_mbps,
        result.max_speed_mbps,
        bold_avg
    );
}

/// Display a single test result row (legacy function for compatibility)
#[allow(dead_code)]
pub fn display_test_result(test_name: &str, result: &TestResult) {
    display_test_result_enhanced(test_name, result, false);
}

/// Format benchmark results as JSON with additional metadata
pub fn format_results_json(results: &BenchmarkResults) -> Result<String, serde_json::Error> {
    use serde_json::json;
    use std::time::SystemTime;

    // Create enhanced JSON with metadata
    let enhanced_results = json!({
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "version": env!("CARGO_PKG_VERSION"),
        "results": {
            "sequential_write": {
                "min_speed_mbps": results.sequential_write.min_speed_mbps,
                "max_speed_mbps": results.sequential_write.max_speed_mbps,
                "avg_speed_mbps": results.sequential_write.avg_speed_mbps,
                "duration_seconds": results.sequential_write.test_duration.as_secs_f64(),
                "sample_count": results.sequential_write.sample_count
            },
            "sequential_read": {
                "min_speed_mbps": results.sequential_read.min_speed_mbps,
                "max_speed_mbps": results.sequential_read.max_speed_mbps,
                "avg_speed_mbps": results.sequential_read.avg_speed_mbps,
                "duration_seconds": results.sequential_read.test_duration.as_secs_f64(),
                "sample_count": results.sequential_read.sample_count
            },
            "random_write": {
                "min_speed_mbps": results.random_write.min_speed_mbps,
                "max_speed_mbps": results.random_write.max_speed_mbps,
                "avg_speed_mbps": results.random_write.avg_speed_mbps,
                "duration_seconds": results.random_write.test_duration.as_secs_f64(),
                "sample_count": results.random_write.sample_count
            },
            "random_read": {
                "min_speed_mbps": results.random_read.min_speed_mbps,
                "max_speed_mbps": results.random_read.max_speed_mbps,
                "avg_speed_mbps": results.random_read.avg_speed_mbps,
                "duration_seconds": results.random_read.test_duration.as_secs_f64(),
                "sample_count": results.random_read.sample_count
            },
            "memory_copy": {
                "min_speed_mbps": results.memory_copy.min_speed_mbps,
                "max_speed_mbps": results.memory_copy.max_speed_mbps,
                "avg_speed_mbps": results.memory_copy.avg_speed_mbps,
                "duration_seconds": results.memory_copy.test_duration.as_secs_f64(),
                "sample_count": results.memory_copy.sample_count
            }
        },
        "summary": {
            "sequential_avg": (results.sequential_write.avg_speed_mbps + results.sequential_read.avg_speed_mbps) / 2.0,
            "random_avg": (results.random_write.avg_speed_mbps + results.random_read.avg_speed_mbps) / 2.0,
            "memory_bandwidth": results.memory_copy.avg_speed_mbps
        }
    });

    serde_json::to_string_pretty(&enhanced_results)
}

/// Format benchmark results as CSV with enhanced data
pub fn format_results_csv(results: &BenchmarkResults) -> String {
    let mut csv = String::new();

    // Enhanced CSV header with more information (P5 replaces Min)
    csv.push_str("Test,P5 (MB/s),P95 (MB/s),Avg (MB/s),Duration (s),Samples\n");

    // Helper function to format a test result as CSV row
    let format_test_csv = |name: &str, result: &TestResult| -> String {
        format!(
            "{},{:.2},{:.2},{:.2},{:.2},{}\n",
            name,
            result.min_speed_mbps,
            result.max_speed_mbps,
            result.avg_speed_mbps,
            result.test_duration.as_secs_f64(),
            result.sample_count
        )
    };

    // Add each test result
    csv.push_str(&format_test_csv(
        "Sequential Write",
        &results.sequential_write,
    ));
    csv.push_str(&format_test_csv(
        "Sequential Read",
        &results.sequential_read,
    ));
    csv.push_str(&format_test_csv("Random Write", &results.random_write));
    csv.push_str(&format_test_csv("Random Read", &results.random_read));
    csv.push_str(&format_test_csv("Memory Copy", &results.memory_copy));

    // Add summary row
    let sequential_avg =
        (results.sequential_write.avg_speed_mbps + results.sequential_read.avg_speed_mbps) / 2.0;
    let random_avg =
        (results.random_write.avg_speed_mbps + results.random_read.avg_speed_mbps) / 2.0;

    csv.push_str("\n# Summary\n");
    csv.push_str(&format!("Sequential Average,,,{:.2},,\n", sequential_avg));
    csv.push_str(&format!("Random Average,,,{:.2},,\n", random_avg));
    csv.push_str(&format!(
        "Memory Bandwidth,,,{:.2},,\n",
        results.memory_copy.avg_speed_mbps
    ));

    csv
}

/// Display helpful usage tips and examples
#[allow(dead_code)]
pub fn display_usage_tips() {
    let use_colors = atty::is(atty::Stream::Stdout);

    let colorize = |text: &str, color_code: &str| -> String {
        if use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    };

    println!("{}", colorize("Usage Tips:", "1;36"));
    println!();
    println!("{}:", colorize("List available devices", "1;33"));
    println!("  disk-speed-test list-devices");
    println!();
    println!("{}:", colorize("Basic benchmark", "1;33"));
    println!("  disk-speed-test benchmark /path/to/test");
    println!();
    println!("{}:", colorize("Custom settings", "1;33"));
    println!("  disk-speed-test benchmark /path/to/test \\");
    println!("    --duration 30 \\");
    println!("    --file-size 2GB \\");
    println!("    --sequential-block-size 8MB \\");
    println!("    --random-block-size 8KB");
    println!();
    println!("{}:", colorize("Output formats", "1;33"));
    println!("  disk-speed-test benchmark /path/to/test --output-format json");
    println!("  disk-speed-test benchmark /path/to/test --output-format csv");
    println!();
    println!("{}:", colorize("Performance Tips", "1;32"));
    println!("  • Close other applications during testing for accurate results");
    println!("  • Use --enable-cache to test with OS caching enabled");
    println!("  • Larger file sizes provide more stable results");
    println!("  • Test on different devices to compare performance");
}
