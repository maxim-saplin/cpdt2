### Architecture

This document explains how the solution is arranged and the main structures used.

## High-level layout

- `src/lib.rs`: Library crate export surface (re-exports core types and platform types).
- `src/main.rs`: CLI entrypoint with process setup and error-to-exit-code mapping.
- `src/core/`: Core, platform-agnostic benchmarking logic.
  - `config.rs`: Configuration structures and validation.
  - `stats.rs`: Statistics collection and result types.
  - `progress.rs`: Progress reporting abstractions and helpers.
  - `mod.rs`: Public API (`run_benchmark`, errors, results) and orchestration.
- `src/platform/`: Platform Abstraction Layer (PAL) and types.
  - `mod.rs`: `PlatformOps` trait, `StorageDevice`, `DeviceType`, and dispatch helpers.
  - Platform-specific modules (`windows.rs`, `macos.rs`, `linux.rs`, `android.rs`, `ios.rs`).
- `src/cli/`: CLI adapter around the library.
  - `args.rs`: `clap`-based argument parsing.
  - `display.rs`: TTY-friendly and structured (JSON/CSV) output.
  - `device_list.rs`: Device enumeration command.
  - `mod.rs`: CLI command routing and `run_benchmark` wiring.
- `src/test_utils/`: Optional helpers for tests (behind feature).

## Execution flow

1) CLI parses arguments into a `BenchmarkConfig` and options: `cli::mod::run_benchmark_command`.
2) CLI constructs a `CliProgressCallback` and calls `core::run_benchmark`.
3) `core::run_benchmark` validates config, prepares a unique test file path, and runs tests:
   - Sequential Write → Sequential Read → Random Write → Random Read → Memory Copy.
   - Each test reports progress via a `ProgressCallback` (if provided).
4) After tests, the temporary file is cleaned up and results returned as `BenchmarkResults`.
5) CLI renders results via `cli::display::display_results` (Table/JSON/CSV).

## Core modules and structures

- `BenchmarkConfig` (`core::config`)
  - Fields: `target_path: PathBuf`, `sequential_block_size: usize`, `random_block_size: usize`,
    `test_duration_seconds: u64`, `disable_os_cache: bool`, `file_size_mb: usize`.
  - Methods: `new`, `validate`, `file_size_bytes`.

- `TestResult` (`core::stats`)
  - Fields: `min_speed_mbps` (P5), `max_speed_mbps` (P95), `avg_speed_mbps`, `test_duration: Duration`,
    `sample_count: usize`.
  - Produced by `StatisticsCollector::finalize()`; uses nearest-rank percentiles for robustness.

- `StatisticsCollector` and `RealTimeStatsTracker` (`core::stats`)
  - Sample speeds approximately every 100ms, compute min/P5, max/P95, average, and expose instantaneous/current averages.
  - Helpers: `calculate_speed_mbps(bytes, duration)`; `record_block`, `update_progress`, `finalize`.

- `ProgressCallback` (`core` trait)
  - Methods: `on_test_start`, `on_progress`, `on_test_complete`.
  - Implementations:
    - `CliProgressCallback` (in `cli::display`) for user-facing progress.
    - `NoOpProgressCallback` (in `core::progress`).
    - `TestProgressCallback` (in `core::progress`) for capturing events in tests.
  - `ProgressReporter` (in `core::progress`) wraps callbacks with throttling (default 100ms) and thread-safety.

- `BenchmarkResults` (`core`)
  - Aggregates five `TestResult` values: `sequential_write`, `sequential_read`, `random_write`, `random_read`, `memory_copy`.

- `BenchmarkError` (`core`)
  - Variants: `PlatformError`, `IoError`, `ConfigurationError`, `InsufficientSpace`, `PermissionDenied`, `TestInterrupted`.
  - Used across CLI and core to provide actionable error messages.

## Platform Abstraction Layer (PAL)

- `PlatformOps` trait (`platform`)
  - `list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError>`
  - `get_app_data_dir() -> Result<PathBuf, PlatformError>`
  - `create_direct_io_file(path, size) -> Result<File, PlatformError>`
  - `open_direct_io_file(path, write) -> Result<File, PlatformError>`
  - `sync_file_system(path) -> Result<(), PlatformError>`
- `StorageDevice` and `DeviceType` describe discovered devices.
- Thin convenience functions (`platform::list_storage_devices`, etc.) dispatch to the active platform module via `cfg`.
- `PlatformError` encapsulates IO and platform-specific failure modes.

## CLI layer

- `args.rs`: `clap`-based `Cli` with subcommands:
  - `list-devices` → `device_list::list_devices_command()`.
  - `benchmark <target_path>` with options: `--sequential-block-size`, `--random-block-size`,
    `--duration`, `--file-size`, `--enable-cache`, `--output-format {table|json|csv}`.
  - `parse_size` utility supports suffixes: B/KB/MB/GB and K/M/G.
- `display.rs`: Formats progress and results.
  - Table output with color, bold average, durations, sample counts.
  - JSON/CSV structured output for machine consumption.
- `main.rs`: Maps `BenchmarkError` kinds to exit codes and provides a panic/SIGINT handler.

## Testing strategy

- Unit tests across core modules validate configuration, statistics, progress throttling, and error handling.
- Integration tests under `src/core/tests.rs` and the `tests/` directory validate end-to-end flows and CLI behavior.
- `test_utils` feature-gated helpers assist with temporary directories and fixtures.

## Extensibility

- The core library is UI-agnostic and re-exported in `lib.rs`, enabling reuse by future GUI apps.
- New tests or metrics can be added by extending `core::tests` implementations and augmenting `BenchmarkResults`.
- Additional output formats can be implemented by adding formatters in `cli::display` without touching the core.
