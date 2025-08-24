# Disk Speed Test

A cross-platform disk speed testing utility that measures sequential and random read/write performance, plus memory copy speed for comparison.

## Features

- Cross-platform support (Windows, macOS, Linux, Android, iOS)
- Sequential read/write tests with configurable block sizes
- Random read/write tests with configurable block sizes  
- Memory copy performance testing
- Real-time progress reporting
- Comprehensive statistics (min/max/average speeds)
- Direct I/O to bypass OS caching
- Command-line interface with device listing

## Building

```bash
# Build for current platform
cargo build --release

# Cross-compile for other platforms (requires cross-compilation setup)
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

## Usage

```bash
# List available storage devices
./disk-speed-test list-devices

# Run benchmark on current directory
./disk-speed-test benchmark .

# Run benchmark with custom settings
./disk-speed-test benchmark /path/to/test \
  --sequential-block-size 8388608 \
  --random-block-size 8192 \
  --duration 30 \
  --file-size 2048
```

## Library Usage

```rust
use disk_speed_test::{BenchmarkConfig, run_benchmark};
use std::path::PathBuf;

let config = BenchmarkConfig::new(PathBuf::from("/tmp"));
let results = run_benchmark(config, None)?;

println!("Sequential Write: {:.2} MB/s", results.sequential_write.avg_speed_mbps);
println!("Sequential Read: {:.2} MB/s", results.sequential_read.avg_speed_mbps);
println!("Random Write: {:.2} MB/s", results.random_write.avg_speed_mbps);
println!("Random Read: {:.2} MB/s", results.random_read.avg_speed_mbps);
println!("Memory Copy: {:.2} MB/s", results.memory_copy.avg_speed_mbps);
```

## Development Status

This project is currently under development. Core interfaces and project structure are complete, but individual test implementations and platform-specific code are still being developed.

## License

MIT License