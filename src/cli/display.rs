//! Display and output formatting for CLI

use crate::core::{ProgressCallback, TestResult, BenchmarkResults};
use std::io::{self, Write};

/// Progress display implementation for CLI
pub struct ProgressDisplay {
    current_test: String,
}

impl ProgressDisplay {
    /// Create a new progress display
    pub fn new() -> Self {
        Self {
            current_test: String::new(),
        }
    }
}

impl ProgressCallback for ProgressDisplay {
    fn on_test_start(&self, test_name: &str) {
        print!("\r{}: Starting...", test_name);
        io::stdout().flush().unwrap();
    }
    
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
        print!("\r{}: {:.1} MB/s", test_name, current_speed_mbps);
        io::stdout().flush().unwrap();
    }
    
    fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        println!("\r{}: Min: {:.1} MB/s, Max: {:.1} MB/s, Avg: {:.1} MB/s", 
            test_name, 
            result.min_speed_mbps, 
            result.max_speed_mbps, 
            result.avg_speed_mbps
        );
    }
}

impl Default for ProgressDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Format benchmark results as a table
pub fn format_results_table(results: &BenchmarkResults) -> String {
    // Stub implementation - will be implemented in task 16
    todo!("Results table formatting will be implemented in task 16")
}

/// Format benchmark results as JSON
pub fn format_results_json(results: &BenchmarkResults) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(results)
}

/// Format benchmark results as CSV
pub fn format_results_csv(results: &BenchmarkResults) -> String {
    // Stub implementation - will be implemented in task 16
    todo!("Results CSV formatting will be implemented in task 16")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TestResult;
    use std::time::Duration;

    #[test]
    fn test_progress_display_creation() {
        let display = ProgressDisplay::new();
        assert!(display.current_test.is_empty());
    }
    
    #[test]
    fn test_json_formatting() {
        let results = BenchmarkResults {
            sequential_write: TestResult {
                min_speed_mbps: 100.0,
                max_speed_mbps: 200.0,
                avg_speed_mbps: 150.0,
                test_duration: Duration::from_secs(10),
            },
            sequential_read: TestResult::default(),
            random_write: TestResult::default(),
            random_read: TestResult::default(),
            memory_copy: TestResult::default(),
        };
        
        let json = format_results_json(&results).unwrap();
        assert!(json.contains("sequential_write"));
        assert!(json.contains("150.0"));
    }
}