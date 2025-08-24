//! Command-line interface for the disk speed test utility

use anyhow::Result;
use disk_speed_test::{BenchmarkConfig, run_benchmark};

pub mod args;
pub mod display;
pub mod device_list;

#[cfg(test)]
mod args_test;

#[cfg(test)]
mod display_test;

use args::{Cli, Commands, OutputFormat, parse_size};
use device_list::list_devices_command;
use display::CliProgressCallback;

/// Main CLI entry point
pub fn run_cli() -> Result<()> {
    let cli = Cli::parse_args();
    
    match cli.command {
        Commands::ListDevices => {
            list_devices_command()?;
        }
        Commands::Benchmark { 
            target_path,
            sequential_block_size,
            random_block_size,
            duration,
            file_size,
            enable_cache,
            output_format,
        } => {
            run_benchmark_command(
                target_path,
                sequential_block_size,
                random_block_size,
                duration,
                file_size,
                enable_cache,
                output_format,
            )?;
        }
    }
    
    Ok(())
}

/// Run the benchmark command with the specified parameters
fn run_benchmark_command(
    target_path: std::path::PathBuf,
    sequential_block_size: Option<String>,
    random_block_size: Option<String>,
    duration: Option<u64>,
    file_size: Option<String>,
    enable_cache: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // Create benchmark configuration
    let mut config = BenchmarkConfig::new(target_path);
    
    // Parse and set optional parameters
    if let Some(size_str) = sequential_block_size {
        config.sequential_block_size = parse_size(&size_str)
            .map_err(|e| anyhow::anyhow!("Invalid sequential block size: {}", e))?;
    }
    
    if let Some(size_str) = random_block_size {
        config.random_block_size = parse_size(&size_str)
            .map_err(|e| anyhow::anyhow!("Invalid random block size: {}", e))?;
    }
    
    if let Some(duration_secs) = duration {
        config.test_duration_seconds = duration_secs;
    }
    
    if let Some(size_str) = file_size {
        let size_bytes = parse_size(&size_str)
            .map_err(|e| anyhow::anyhow!("Invalid file size: {}", e))?;
        config.file_size_mb = size_bytes / (1024 * 1024);
        if config.file_size_mb == 0 {
            config.file_size_mb = 1; // Minimum 1MB
        }
    }
    
    // Set cache behavior (note: disable_os_cache is opposite of enable_cache)
    config.disable_os_cache = !enable_cache;
    
    // Validate configuration with enhanced error reporting
    if let Err(e) = config.validate() {
        return Err(anyhow::anyhow!("Configuration validation failed: {}", e));
    }
    
    // Create progress callback
    let progress_callback = CliProgressCallback::new(output_format.clone());
    
    // Display configuration (only for table format)
    if matches!(output_format, OutputFormat::Table) {
        display_benchmark_config(&config);
        println!("\nStarting benchmark tests...\n");
    }
    
    let results = run_benchmark(config, Some(Box::new(progress_callback)))
        .map_err(|e| anyhow::anyhow!("Benchmark failed: {}", e))?;
    
    // Display results
    display::display_results(&results, &output_format)?;
    
    Ok(())
}

/// Display the benchmark configuration before starting tests
fn display_benchmark_config(config: &BenchmarkConfig) {
    println!("Benchmark Configuration:");
    println!("  Target path: {}", config.target_path.display());
    println!("  Sequential block size: {} MB", config.sequential_block_size / (1024 * 1024));
    println!("  Random block size: {} KB", config.random_block_size / 1024);
    println!("  Test duration: {} seconds", config.test_duration_seconds);
    println!("  Test file size: {} MB", config.file_size_mb);
    println!("  OS caching: {}", if config.disable_os_cache { "disabled" } else { "enabled" });
}