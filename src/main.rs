//! Disk Speed Test CLI
//! 
//! Command-line interface for the disk speed test utility
//! 
//! This is the main entry point for the disk speed test command-line application.
//! It provides a cross-platform interface for benchmarking disk performance with
//! comprehensive error handling and proper exit codes.

use std::process;
use std::env;

mod cli;

use cli::run_cli;
use disk_speed_test::BenchmarkError;

/// Main entry point for the disk speed test CLI
/// 
/// This function handles command line execution with proper error handling
/// and exit codes as specified in requirements 1.1, 1.2, 2.1, 2.2, 2.3.
/// 
/// Exit codes:
/// - 0: Success
/// - 1: General error (configuration, I/O, etc.)
/// - 2: Platform-specific error
/// - 3: Permission denied
/// - 4: Insufficient disk space
/// - 5: Test interrupted
fn main() {
    // Set up panic handler for better error reporting
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Fatal error: {}", panic_info);
        eprintln!("This is likely a bug. Please report this issue.");
        process::exit(128);
    }));
    
    // Handle SIGINT (Ctrl+C) gracefully
    #[cfg(unix)]
    {
        extern "C" fn handle_sigint(_: i32) {
            eprintln!("\nBenchmark interrupted by user");
            process::exit(130); // Standard exit code for SIGINT
        }
        
        unsafe {
            libc::signal(libc::SIGINT, handle_sigint as usize);
        }
    }
    
    // Check for help or version flags early
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-V" => {
                println!("disk-speed-test {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--help" | "-h" => {
                // Let clap handle help display
            }
            _ => {}
        }
    }
    
    // Run the CLI with comprehensive error handling
    match run_cli() {
        Ok(()) => {
            process::exit(0);
        }
        Err(e) => {
            // Display error with helpful diagnostic information
            display_cli_error(&e);
            
            // Exit with appropriate code based on error type
            let exit_code = match e.downcast_ref::<BenchmarkError>() {
                Some(BenchmarkError::PlatformError(_)) => 2,
                Some(BenchmarkError::PermissionDenied(_)) => 3,
                Some(BenchmarkError::InsufficientSpace { .. }) => 4,
                Some(BenchmarkError::TestInterrupted(_)) => 5,
                Some(BenchmarkError::ConfigurationError(_)) => 1,
                Some(BenchmarkError::IoError(_)) => 1,
                None => 1, // General error
            };
            
            process::exit(exit_code);
        }
    }
}

/// Display CLI-specific error messages with helpful information
/// 
/// This function provides user-friendly error messages and suggestions
/// for resolving common issues, as required by requirements 11.4 and 11.5.
fn display_cli_error(error: &anyhow::Error) {
    use disk_speed_test::BenchmarkError;
    
    // Check if this is a BenchmarkError that we can provide specific help for
    if let Some(benchmark_error) = error.downcast_ref::<BenchmarkError>() {
        cli::display::display_error(benchmark_error);
        return;
    }
    
    // Handle other types of errors
    let use_colors = atty::is(atty::Stream::Stderr);
    let colorize = |text: &str, color_code: &str| -> String {
        if use_colors {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    };
    
    let error_prefix = colorize("Error:", "1;31"); // Bold red
    let info_prefix = colorize("Info:", "1;34"); // Bold blue
    
    eprintln!("{} {}", error_prefix, error);
    
    // Provide general help based on error message content
    let error_msg = error.to_string().to_lowercase();
    
    if error_msg.contains("permission") || error_msg.contains("access") {
        eprintln!("{} Try running with administrator/root privileges or choose a different path.", info_prefix);
        eprintln!("  Use 'disk-speed-test list-devices' to find suitable locations.");
    } else if error_msg.contains("not found") || error_msg.contains("no such file") {
        eprintln!("{} Verify the target path exists and is accessible.", info_prefix);
        eprintln!("  Use 'disk-speed-test list-devices' to see available devices.");
    } else if error_msg.contains("space") || error_msg.contains("full") {
        eprintln!("{} Try using a smaller file size with --file-size option.", info_prefix);
        eprintln!("  Example: --file-size 100MB");
    } else if error_msg.contains("argument") || error_msg.contains("option") {
        eprintln!("{} Check your command line arguments.", info_prefix);
        eprintln!("  Use 'disk-speed-test --help' for usage information.");
    } else {
        eprintln!("{} Use 'disk-speed-test --help' for usage information.", info_prefix);
        eprintln!("  Use 'disk-speed-test list-devices' to see available devices.");
    }
}