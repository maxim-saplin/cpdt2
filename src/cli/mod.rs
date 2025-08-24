//! Command-line interface for the disk speed test utility

use crate::BenchmarkError;

pub mod args;
pub mod display;
pub mod device_list;

pub use args::CliArgs;
pub use display::ProgressDisplay;

/// Main CLI application structure
pub struct CliApp {
    args: CliArgs,
}

impl CliApp {
    /// Create a new CLI application
    pub fn new() -> Self {
        Self {
            args: CliArgs::parse(),
        }
    }
    
    /// Run the CLI application
    pub fn run(self) -> Result<(), BenchmarkError> {
        // Stub implementation - will be implemented in task 15
        todo!("CLI application run method will be implemented in task 15")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_app_creation() {
        // This test will be updated when CLI parsing is implemented
        assert!(true);
    }
}