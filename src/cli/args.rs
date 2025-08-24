//! Command line argument parsing

use std::path::PathBuf;
use anyhow::Result;

/// Output format options
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// CLI commands
#[derive(Debug)]
#[allow(dead_code)]
pub enum Commands {
    /// List available storage devices
    ListDevices,
    
    /// Run benchmark tests
    Benchmark {
        /// Target path for test files
        target_path: PathBuf,
        
        /// Sequential block size in bytes
        sequential_block_size: Option<usize>,
        
        /// Random block size in bytes
        random_block_size: Option<usize>,
        
        /// Test duration in seconds
        duration: Option<u64>,
        
        /// Test file size in MB
        file_size: Option<usize>,
        
        /// Enable OS caching (default: disabled)
        enable_cache: bool,
        
        /// Output format
        output_format: OutputFormat,
    },
}

/// Main CLI structure
#[derive(Debug)]
pub struct Cli {
    pub command: Commands,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Result<Self> {
        // TODO: Implement actual argument parsing in task 15
        // This is a placeholder that will be replaced with clap-based parsing
        
        let args: Vec<String> = std::env::args().collect();
        
        if args.len() < 2 {
            return Ok(Cli {
                command: Commands::ListDevices,
            });
        }
        
        match args[1].as_str() {
            "list-devices" => Ok(Cli {
                command: Commands::ListDevices,
            }),
            "benchmark" => {
                let target_path = if args.len() > 2 {
                    PathBuf::from(&args[2])
                } else {
                    PathBuf::from(".")
                };
                
                Ok(Cli {
                    command: Commands::Benchmark {
                        target_path,
                        sequential_block_size: None,
                        random_block_size: None,
                        duration: None,
                        file_size: None,
                        enable_cache: false,
                        output_format: OutputFormat::Table,
                    },
                })
            }
            _ => Ok(Cli {
                command: Commands::ListDevices,
            }),
        }
    }
}