//! Disk Speed Test CLI
//! 
//! Command-line interface for the disk speed test utility.

use std::process;
use disk_speed_test::BenchmarkError;

mod cli;

use cli::CliApp;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        
        // Print additional context for common errors
        match e {
            BenchmarkError::PermissionDenied(path) => {
                eprintln!("Try running with elevated permissions or choose a different target path.");
                eprintln!("Target path: {}", path.display());
            }
            BenchmarkError::InsufficientSpace { required, available } => {
                eprintln!("Required: {} MB, Available: {} MB", 
                    required / 1024 / 1024, 
                    available / 1024 / 1024);
                eprintln!("Try reducing the test file size with --file-size option.");
            }
            BenchmarkError::ConfigurationError(_) => {
                eprintln!("Use --help for usage information.");
            }
            _ => {}
        }
        
        process::exit(1);
    }
}

fn run() -> Result<(), BenchmarkError> {
    let app = CliApp::new();
    app.run()
}