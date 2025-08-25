# Testing Infrastructure

This document describes the comprehensive testing infrastructure for the disk-speed-test project.

## Overview

The testing infrastructure provides:

- **Automated CI/CD pipelines** for cross-platform testing
- **Test data management** with controlled test environments
- **Code coverage reporting** with quality gates
- **Test utilities** for creating reproducible test scenarios

## Test Structure

```
tests/
├── benchmark_*.rs              # Integration tests for benchmark operations
├── cli_integration_tests.rs    # CLI interface tests
├── test_infrastructure_tests.rs # Tests for test utilities themselves
src/
├── test_utils/                 # Test utility modules (test builds only)
│   ├── mod.rs                 # Main test utilities
│   ├── test_data.rs           # Test data generation
│   ├── test_environment.rs    # Test environment management
│   └── cleanup.rs             # Resource cleanup utilities
benches/                        # (removed) no performance regression benchmarks
```

## Running Tests

### Quick Test Commands

```bash
# Run all tests
make test

# Run specific test types
make test-unit          # Unit tests only
make test-integration   # Integration tests only
make test-all          # Comprehensive test suite

# Run with coverage
make coverage

# Run benchmarks
make benchmark
```

### Comprehensive Test Suite

The comprehensive test suite (`./scripts/test-runner.sh`) includes:

1. **Prerequisites Check** - Verifies environment setup
2. **Code Formatting** - Ensures consistent code style
3. **Clippy Linting** - Catches common mistakes and improvements
4. **Unit Tests** - Tests individual components
5. **Integration Tests** - Tests component interactions
6. **Code Coverage** - Measures test coverage with quality gates
7. **Security Audit** - Checks for known vulnerabilities

### CI/CD Pipeline

The project uses GitHub Actions for automated testing:

- **`.github/workflows/ci.yml`** - Main CI pipeline
- **`.github/workflows/coverage.yml`** - Code coverage analysis

#### CI Pipeline Features

- **Cross-platform testing** on Ubuntu, Windows, and macOS
- **Multiple Rust versions** (stable and beta)
- **Cross-compilation** verification
- **Code coverage** reporting with Codecov integration
- **Quality gates** with configurable thresholds

## Test Utilities

### TestDataManager

Manages temporary test files and directories:

```rust
use disk_speed_test::test_utils::TestDataManager;

let mut manager = TestDataManager::new()?;

// Create test files
let file_path = manager.create_test_file("test.dat", 1024)?;
let random_file = manager.create_random_test_file("random.dat", 2048)?;

// Automatic cleanup on drop
```

### TestEnvironment

Provides controlled test environments:

```rust
use disk_speed_test::test_utils::{TestEnvironment, TestEnvironmentBuilder};
use std::time::Duration;

let env = TestEnvironmentBuilder::new()
    .min_free_space(100 * 1024 * 1024)  // 100MB
    .max_test_duration(Duration::from_secs(30))
    .use_small_files(true)
    .build()?;

// Create benchmark config suitable for testing
let config = env.create_test_benchmark_config(None);
```

### TestDataGenerator

Generates test data with specific patterns:

```rust
use disk_speed_test::test_utils::test_data::{TestDataGenerator, TestDataPattern};

// Generate reproducible random data
let mut generator = TestDataGenerator::new(TestDataPattern::RandomSeeded(12345));
generator.generate_file(&file_path, 1024)?;

// Generate pattern data
let mut generator = TestDataGenerator::new(TestDataPattern::Sequential);
generator.generate_file(&file_path, 1024)?;
```

### CleanupGuard

RAII cleanup for test resources:

```rust
use disk_speed_test::test_utils::cleanup::CleanupGuard;

{
    let _guard = CleanupGuard::for_file(&test_file);
    // File will be cleaned up when guard is dropped
}
```

## Performance Testing

### Performance Testing

Performance regression benchmarks have been removed from this project. Focus is on correctness, stability, and reporting accuracy rather than enforcing runtime performance thresholds.

## Code Coverage

### Generating Coverage Reports

```bash
# HTML report (opens in browser)
make coverage

# LCOV format for CI
make coverage-ci

# Using cargo-llvm-cov directly
cargo llvm-cov --all-features --workspace --html --open
```

### Coverage Requirements

- **Minimum Coverage**: 80%
- **Quality Gate**: CI fails if coverage drops below threshold
- **Differential Coverage**: PR coverage comparison available

## Test Configuration

### Environment Variables

- `RUST_TEST_THREADS=1` - Run tests sequentially (default for I/O tests)
- `RUST_BACKTRACE=1` - Enable backtraces for debugging
- `DISK_SPEED_TEST_LOG=debug` - Enable debug logging
- `DISK_SPEED_TEST_TEMP_DIR` - Custom temporary directory

### Test Profiles

#### Default Profile
- Sequential execution for I/O tests
- 60-second timeout for slow tests
- 2 retries for flaky tests

#### CI Profile
- Stricter 30-second timeout
- 1 retry only
- More verbose output

#### Coverage Profile
- No retries to avoid double counting
- Includes all tests

## Platform-Specific Testing

### Windows Testing
- Direct I/O with `FILE_FLAG_NO_BUFFERING`
- Device enumeration via Windows APIs
- NTFS/ReFS file system testing

### macOS Testing
- Direct I/O with `F_NOCACHE`
- Device enumeration via system APIs
- APFS/HFS+ file system testing

### Linux Testing
- Direct I/O with `O_DIRECT`
- Device enumeration via `/proc/mounts`
- ext4/xfs/btrfs file system testing

## Troubleshooting

### Common Issues

1. **Insufficient Disk Space**
   ```bash
   # Check available space
   df -h /tmp
   
   # Use smaller test files
   DISK_SPEED_TEST_USE_SMALL_FILES=1 cargo test
   ```

2. **Permission Denied**
   ```bash
   # Skip privileged tests
   DISK_SPEED_TEST_SKIP_PRIVILEGED=1 cargo test
   ```

3. **Test Timeouts**
   ```bash
   # Increase timeout
   DISK_SPEED_TEST_MAX_DURATION=120 cargo test
   ```

4. **Flaky Tests**
   ```bash
   # Run with retries
   cargo nextest run --retries 3
   ```

### Debug Mode

Enable debug logging for detailed test execution:

```bash
RUST_LOG=debug cargo test
```

### Test Isolation

Tests are designed to be isolated:
- Each test uses its own temporary directory
- Sequential execution prevents I/O conflicts
- Automatic cleanup prevents resource leaks

## Contributing

When adding new tests:

1. **Use test utilities** for consistent test environments
2. **Add cleanup** for any resources created
3. **Include performance tests** for new functionality
4. **Update documentation** for new test patterns
5. **Verify CI passes** on all platforms

### Test Naming Conventions

- `test_*` - Unit tests
- `integration_*` - Integration tests
- `benchmark_*` - Performance benchmarks
- `regression_*` - Regression tests

### Test Organization

- Group related tests in modules
- Use descriptive test names
- Include documentation for complex test scenarios
- Add platform-specific tests when needed