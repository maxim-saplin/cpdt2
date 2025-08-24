//! Command-line argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Disk Speed Test CLI
#[derive(Parser, Debug)]
#[command(name = "disk-speed-test")]
#[command(about = "Cross-platform disk speed testing utility")]
#[command(version)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List available storage devices
    ListDevices,
    
    /// Run benchmark tests
    Benchmark {
        /// Target path for test files
        target_path: PathBuf,
        
        /// Sequential block size (e.g., 4MB, 8MB)
        #[arg(long, value_name = "SIZE")]
        sequential_block_size: Option<String>,
        
        /// Random block size (e.g., 4KB, 8KB)
        #[arg(long, value_name = "SIZE")]
        random_block_size: Option<String>,
        
        /// Test duration per benchmark in seconds
        #[arg(long, default_value = "10")]
        duration: u64,
        
        /// Test file size (e.g., 1GB, 512MB)
        #[arg(long, value_name = "SIZE")]
        file_size: Option<String>,
        
        /// Enable OS caching (default: disabled for accurate results)
        #[arg(long)]
        enable_cache: bool,
        
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        output_format: OutputFormat,
    },
}

/// Output format options
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse() -> Self {
        Parser::parse()
    }
}

/// Parse size string (e.g., "4MB", "1GB") to bytes
pub fn parse_size(size_str: &str) -> Result<usize, String> {
    let size_str = size_str.to_uppercase();
    
    if let Some(num_str) = size_str.strip_suffix("GB") {
        let num: f64 = num_str.parse().map_err(|_| "Invalid number format")?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as usize)
    } else if let Some(num_str) = size_str.strip_suffix("MB") {
        let num: f64 = num_str.parse().map_err(|_| "Invalid number format")?;
        Ok((num * 1024.0 * 1024.0) as usize)
    } else if let Some(num_str) = size_str.strip_suffix("KB") {
        let num: f64 = num_str.parse().map_err(|_| "Invalid number format")?;
        Ok((num * 1024.0) as usize)
    } else if let Some(num_str) = size_str.strip_suffix("B") {
        let num: usize = num_str.parse().map_err(|_| "Invalid number format")?;
        Ok(num)
    } else {
        // Try parsing as plain number (assume bytes)
        let num: usize = size_str.parse().map_err(|_| "Invalid size format. Use format like '4MB', '1GB', etc.")?;
        Ok(num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("4MB").unwrap(), 4 * 1024 * 1024);
        assert_eq!(parse_size("4KB").unwrap(), 4 * 1024);
        assert_eq!(parse_size("1024B").unwrap(), 1024);
        assert_eq!(parse_size("1024").unwrap(), 1024);
        
        assert!(parse_size("invalid").is_err());
        assert!(parse_size("1.5XB").is_err());
    }
    
    #[test]
    fn test_parse_size_case_insensitive() {
        assert_eq!(parse_size("1gb").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("4mb").unwrap(), 4 * 1024 * 1024);
        assert_eq!(parse_size("4kb").unwrap(), 4 * 1024);
    }
}