//! Command line argument parsing

use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

/// Output format options
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    /// Display results in a formatted table
    #[default]
    Table,
    /// Output results as JSON
    Json,
    /// Output results as CSV
    Csv,
}

/// CLI commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available storage devices
    ListDevices,
    
    /// Run benchmark tests
    Benchmark {
        /// Target path for test files
        target_path: PathBuf,
        
        /// Sequential block size in bytes (default: 4MB)
        #[arg(long, value_name = "SIZE")]
        sequential_block_size: Option<String>,
        
        /// Random block size in bytes (default: 4KB)
        #[arg(long, value_name = "SIZE")]
        random_block_size: Option<String>,
        
        /// Test duration in seconds (default: 10)
        #[arg(long, short = 'd', value_name = "SECONDS")]
        duration: Option<u64>,
        
        /// Test file size (default: 1GB)
        #[arg(long, value_name = "SIZE")]
        file_size: Option<String>,
        
        /// Enable OS caching (default: disabled for accurate results)
        #[arg(long)]
        enable_cache: bool,
        
        /// Output format
        #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Table)]
        output_format: OutputFormat,
    },
}

/// Cross-platform disk speed testing utility
#[derive(Debug, Parser)]
#[command(name = "disk-speed-test")]
#[command(about = "A cross-platform disk speed testing utility")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Disk Speed Test Team")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

/// Parse a size string (e.g., "4MB", "1GB", "512KB") into bytes
pub fn parse_size(size_str: &str) -> Result<usize, String> {
    let size_str = size_str.trim().to_uppercase();
    
    if size_str.is_empty() {
        return Err("Empty size string".to_string());
    }
    
    // Extract number and unit
    let (number_part, unit_part) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
        (&size_str[..pos], &size_str[pos..])
    } else {
        // No unit specified, assume bytes
        return size_str.parse::<usize>()
            .map_err(|_| format!("Invalid size format: {}", size_str));
    };
    
    let number: f64 = number_part.parse()
        .map_err(|_| format!("Invalid number in size: {}", number_part))?;
    
    // Check for negative numbers
    if number < 0.0 {
        return Err("Size cannot be negative".to_string());
    }
    
    let multiplier = match unit_part {
        "B" => 1,
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        "K" => 1024,
        "M" => 1024 * 1024,
        "G" => 1024 * 1024 * 1024,
        _ => return Err(format!("Unknown size unit: {}", unit_part)),
    };
    
    let result = number * multiplier as f64;
    
    // Check for overflow
    if result > usize::MAX as f64 {
        return Err("Size too large".to_string());
    }
    
    Ok(result as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("4MB").unwrap(), 4 * 1024 * 1024);
        assert_eq!(parse_size("512KB").unwrap(), 512 * 1024);
        assert_eq!(parse_size("2G").unwrap(), 2 * 1024 * 1024 * 1024);
        
        // Test case insensitive
        assert_eq!(parse_size("1mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1Mb").unwrap(), 1024 * 1024);
        
        // Test with spaces
        assert_eq!(parse_size(" 1MB ").unwrap(), 1024 * 1024);
        
        // Test invalid formats
        assert!(parse_size("invalid").is_err());
        assert!(parse_size("1XB").is_err());
        assert!(parse_size("").is_err());
    }
}