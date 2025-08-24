//! Disk Speed Test CLI
//! 
//! Command-line interface for the disk speed test utility

use std::process;

mod cli;

use cli::run_cli;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}