//! Command-line interface for the disk speed test utility

use anyhow::Result;

pub mod args;
pub mod display;
pub mod device_list;

use args::{Cli, Commands};
use device_list::list_devices_command;

/// Main CLI entry point
pub fn run_cli() -> Result<()> {
    let cli = Cli::parse_args()?;
    
    match cli.command {
        Commands::ListDevices => {
            list_devices_command()?;
        }
        Commands::Benchmark { 
            target_path,
            sequential_block_size: _,
            random_block_size: _,
            duration: _,
            file_size: _,
            enable_cache: _,
            output_format: _,
        } => {
            // TODO: Implement benchmark command in task 15
            println!("Benchmark command not yet implemented");
            println!("Target path: {}", target_path.display());
        }
    }
    
    Ok(())
}