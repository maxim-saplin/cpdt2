//! Display and output formatting for CLI

use disk_speed_test::core::{ProgressCallback, TestResult, BenchmarkResults};

/// CLI progress callback implementation
#[allow(dead_code)]
pub struct CliProgressCallback;

impl ProgressCallback for CliProgressCallback {
    fn on_test_start(&self, test_name: &str) {
        println!("Starting {}...", test_name);
    }
    
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
        print!("\r{}: {:.2} MB/s", test_name, current_speed_mbps);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }
    
    fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        println!("\n{} complete: Min: {:.2} MB/s, Max: {:.2} MB/s, Avg: {:.2} MB/s", 
                 test_name, result.min_speed_mbps, result.max_speed_mbps, result.avg_speed_mbps);
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