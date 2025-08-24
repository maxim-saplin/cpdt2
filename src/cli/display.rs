//! Display and output formatting for CLI

use disk_speed_test::core::{ProgressCallback, TestResult, BenchmarkResults};
use std::io::{self, Write};

/// CLI progress callback implementation
/// 
/// This implementation provides real-time progress display for the command line interface,
/// showing test names and current speeds with proper formatting according to requirements
/// 10.1, 10.2, 10.3, 10.4, 10.5, and 10.6.
#[derive(Debug)]
pub struct CliProgressCallback {
    /// Whether to use colored output (if terminal supports it)
    use_colors: bool,
    /// Whether to show verbose progress information
    verbose: bool,
}

impl CliProgressCallback {
    /// Create a new CLI progress callback with default settings
    pub fn new() -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            verbose: false,
        }
    }
    
    /// Create a new CLI progress callback with custom settings
    /// 
    /// # Arguments
    /// 
    /// * `use_colors` - Whether to use colored output
    /// * `verbose` - Whether to show verbose progress information
    pub fn with_options(use_colors: bool, verbose: bool) -> Self {
        Self {
            use_colors,
            verbose,
        }
    }
    
    /// Enable or disable colored output
    pub fn set_colors(&mut self, use_colors: bool) {
        self.use_colors = use_colors;
    }
    
    /// Enable or disable verbose output
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
    
    /// Format speed with appropriate precision and units
    fn format_speed(&self, speed_mbps: f64) -> String {
        if speed_mbps >= 1000.0 {
            format!("{:.1} GB/s", speed_mbps / 1000.0)
        } else if speed_mbps >= 1.0 {
            format!("{:.2} MB/s", speed_mbps)
        } else {
            format!("{:.3} MB/s", speed_mbps)
        }
    }
    
    /// Apply color formatting if colors are enabled
    fn colorize(&self, text: &str, color_code: &str) -> String {
        if self.use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    }
    
    /// Clear the current line (for progress updates)
    fn clear_line(&self) {
        if self.use_colors {
            print!("\r\x1b[K");
        } else {
            print!("\r");
        }
    }
}

impl Default for CliProgressCallback {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressCallback for CliProgressCallback {
    fn on_test_start(&self, test_name: &str) {
        // Requirements 10.1, 10.2, 10.3, 10.4, 10.5: Display test name when starting
        let colored_name = self.colorize(test_name, "1;36"); // Bright cyan
        println!("Starting {}...", colored_name);
        
        if self.verbose {
            println!("  Initializing test parameters and creating test files...");
        }
        
        // Ensure output is flushed
        io::stdout().flush().unwrap_or(());
    }
    
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
        // Requirements 10.1, 10.2, 10.3, 10.4, 10.5: Show current test name and speed
        self.clear_line();
        
        let colored_name = self.colorize(test_name, "1;33"); // Bright yellow
        let formatted_speed = self.format_speed(current_speed_mbps);
        let colored_speed = self.colorize(&formatted_speed, "1;32"); // Bright green
        
        print!("{}: {}", colored_name, colored_speed);
        
        if self.verbose {
            print!(" (real-time)");
        }
        
        // Ensure output is flushed for real-time display
        io::stdout().flush().unwrap_or(());
    }
    
    fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        // Requirement 10.6: Show Min, Max, and average (bold) speeds when test completes
        self.clear_line();
        
        let colored_name = self.colorize(test_name, "1;32"); // Bright green
        let min_speed = self.format_speed(result.min_speed_mbps);
        let max_speed = self.format_speed(result.max_speed_mbps);
        let avg_speed = self.format_speed(result.avg_speed_mbps);
        
        // Make average speed bold as per requirement
        let bold_avg = self.colorize(&avg_speed, "1"); // Bold
        
        println!("{} complete:", colored_name);
        println!("  Min: {}, Max: {}, Avg: {}", 
                 self.colorize(&min_speed, "37"), // Light gray
                 self.colorize(&max_speed, "37"), // Light gray  
                 bold_avg);
        
        if self.verbose {
            println!("  Duration: {:.1}s, Samples: {}", 
                     result.test_duration.as_secs_f64(),
                     result.sample_count);
        }
        
        // Add a blank line for readability
        println!();
        
        // Ensure output is flushed
        io::stdout().flush().unwrap_or(());
    }
}

/// Format benchmark results as a table
#[allow(dead_code)]
pub fn format_results_table(_results: &BenchmarkResults) -> String {
    // TODO: Implement table formatting in task 16
    format!("Results table formatting not yet implemented")
}

/// Format benchmark results as JSON
#[allow(dead_code)]
pub fn format_results_json(results: &BenchmarkResults) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(results)
}

/// Format benchmark results as CSV
#[allow(dead_code)]
pub fn format_results_csv(_results: &BenchmarkResults) -> String {
    // TODO: Implement CSV formatting in task 16
    format!("CSV formatting not yet implemented")
}