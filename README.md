# Disk Speed Test

A cross-platform disk speed testing utility written in Rust.

## Features

- Cross-platform support (Windows, macOS, Linux, Android, iOS)
- Sequential and random read/write tests
- Memory copy performance testing
- Real-time progress reporting
- Comprehensive statistics (min/max/avg speeds)
- Direct I/O to bypass OS caching
- Command-line interface with device listing

## Building

### Prerequisites

- Rust 1.70 or later
- Platform-specific toolchains for cross-compilation (optional)

### Build for current platform

```bash
cargo build --release
```

### Cross-compilation

```bash
# Windows (from Linux/macOS)
cargo build --release --target x86_64-pc-windows-gnu

# macOS (from Linux)
cargo build --release --target x86_64-apple-darwin

# Linux (from macOS/Windows)
cargo build --release --target x86_64-unknown-linux-gnu
```

## Usage

### List available devices

```bash
disk-speed-test list-devices
```

### Run benchmark

```bash
# Basic benchmark on current directory
disk-speed-test benchmark .

# Custom configuration
disk-speed-test benchmark /path/to/test \
  --sequential-block-size 8MB \
  --random-block-size 8KB \
  --duration 30 \
  --file-size 2GB
```

### Options

- `--sequential-block-size`: Block size for sequential tests (default: 4MB)
- `--random-block-size`: Block size for random tests (default: 4KB)
- `--duration`: Test duration in seconds (default: 10)
- `--file-size`: Test file size (default: 1GB)
- `--enable-cache`: Enable OS caching (default: disabled)
- `--output-format`: Output format (table, json, csv)

## Library Usage

The core functionality is available as a library:

```rust
use disk_speed_test::{BenchmarkConfig, run_benchmark};

let config = BenchmarkConfig::default();
let results = run_benchmark(config, None)?;

println!("Sequential Write: {:.1} MB/s", results.sequential_write.avg_speed_mbps);
```

## License

MIT License - see LICENSE file for details.