//! Performance regression benchmarks
//!
//! These benchmarks are designed to detect performance regressions in the disk speed test
//! library by establishing baseline performance metrics and alerting when performance
//! degrades significantly.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use disk_speed_test::{run_benchmark, BenchmarkConfig};
use std::path::PathBuf;
use std::time::Duration;

/// Baseline performance thresholds (in MB/s)
const MIN_MEMORY_COPY_SPEED: f64 = 1000.0; // 1 GB/s minimum for memory copy
const MIN_SEQUENTIAL_WRITE_SPEED: f64 = 10.0; // 10 MB/s minimum for sequential write
const MIN_SEQUENTIAL_READ_SPEED: f64 = 10.0; // 10 MB/s minimum for sequential read

/// Test configuration for regression benchmarks
fn create_regression_test_config(temp_dir: &std::path::Path) -> BenchmarkConfig {
    BenchmarkConfig {
        target_path: temp_dir.to_path_buf(),
        sequential_block_size: 1024 * 1024, // 1MB for faster testing
        random_block_size: 4 * 1024,        // 4KB
        test_duration_seconds: 2,           // Short duration for CI
        disable_os_cache: true,
        file_size_mb: 10, // Small file for CI
    }
}

/// Benchmark memory copy performance
fn benchmark_memory_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_copy_regression");

    for size_mb in [1, 10, 50].iter() {
        group.throughput(Throughput::Bytes(*size_mb * 1024 * 1024));
        group.bench_with_input(
            BenchmarkId::new("memory_copy", size_mb),
            size_mb,
            |b, &size_mb| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

                    let config = BenchmarkConfig {
                        target_path: temp_dir.path().to_path_buf(),
                        sequential_block_size: 1024 * 1024,
                        random_block_size: 4 * 1024,
                        test_duration_seconds: 1,
                        disable_os_cache: true,
                        file_size_mb: size_mb as usize,
                    };

                    let results = run_benchmark(config, None).expect("Benchmark failed");

                    // Verify minimum performance threshold
                    assert!(
                        results.memory_copy.avg_speed_mbps > MIN_MEMORY_COPY_SPEED,
                        "Memory copy performance regression: {:.2} MB/s < {:.2} MB/s",
                        results.memory_copy.avg_speed_mbps,
                        MIN_MEMORY_COPY_SPEED
                    );

                    black_box(results.memory_copy.avg_speed_mbps)
                });
            },
        );
    }
    group.finish();
}

/// Benchmark sequential operations performance
fn benchmark_sequential_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_operations_regression");

    for block_size_kb in [64, 256, 1024, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("sequential_write", block_size_kb),
            block_size_kb,
            |b, &block_size_kb| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

                    let config = BenchmarkConfig {
                        target_path: temp_dir.path().to_path_buf(),
                        sequential_block_size: block_size_kb * 1024,
                        random_block_size: 4 * 1024,
                        test_duration_seconds: 1,
                        disable_os_cache: true,
                        file_size_mb: 5,
                    };

                    let results = run_benchmark(config, None).expect("Benchmark failed");

                    // Verify minimum performance threshold
                    assert!(
                        results.sequential_write.avg_speed_mbps > MIN_SEQUENTIAL_WRITE_SPEED,
                        "Sequential write performance regression: {:.2} MB/s < {:.2} MB/s",
                        results.sequential_write.avg_speed_mbps,
                        MIN_SEQUENTIAL_WRITE_SPEED
                    );

                    black_box(results.sequential_write.avg_speed_mbps)
                });
            },
        );
    }
    group.finish();
}

/// Benchmark random operations performance
fn benchmark_random_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_operations_regression");

    for block_size_kb in [1, 4, 16, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("random_write", block_size_kb),
            block_size_kb,
            |b, &block_size_kb| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

                    let config = BenchmarkConfig {
                        target_path: temp_dir.path().to_path_buf(),
                        sequential_block_size: 1024 * 1024,
                        random_block_size: block_size_kb * 1024,
                        test_duration_seconds: 1,
                        disable_os_cache: true,
                        file_size_mb: 5,
                    };

                    let results = run_benchmark(config, None).expect("Benchmark failed");

                    // Random operations are typically slower, so lower threshold
                    let min_random_speed = 1.0; // 1 MB/s minimum
                    assert!(
                        results.random_write.avg_speed_mbps > min_random_speed,
                        "Random write performance regression: {:.2} MB/s < {:.2} MB/s",
                        results.random_write.avg_speed_mbps,
                        min_random_speed
                    );

                    black_box(results.random_write.avg_speed_mbps)
                });
            },
        );
    }
    group.finish();
}

/// Benchmark statistics collection overhead
fn benchmark_statistics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_overhead");

    group.bench_function("stats_collection", |b| {
        b.iter(|| {
            // Simulate collecting 1000 samples
            let mut samples = Vec::new();
            for i in 0..1000 {
                let speed = 50.0 + (i as f64 * 0.1); // Varying speed
                samples.push(speed);
            }

            // Calculate basic statistics
            let min = samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = samples.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg = samples.iter().sum::<f64>() / samples.len() as f64;

            black_box((min, max, avg))
        });
    });

    group.finish();
}

/// Benchmark configuration parsing and validation
fn benchmark_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");

    group.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = BenchmarkConfig {
                target_path: PathBuf::from("/tmp"),
                sequential_block_size: 4 * 1024 * 1024,
                random_block_size: 4 * 1024,
                test_duration_seconds: 10,
                disable_os_cache: true,
                file_size_mb: 1024,
            };
            black_box(config)
        });
    });

    group.finish();
}

/// Comprehensive regression test that runs all benchmark types
fn benchmark_full_regression_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_regression_suite");
    group.sample_size(10); // Fewer samples for comprehensive test
    group.measurement_time(Duration::from_secs(60)); // Longer measurement time

    group.bench_function("complete_benchmark_suite", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
            let config = create_regression_test_config(temp_dir.path());

            let results = run_benchmark(config, None).expect("Full benchmark suite failed");

            // Verify all results meet minimum thresholds
            assert!(results.memory_copy.avg_speed_mbps > MIN_MEMORY_COPY_SPEED);
            assert!(results.sequential_write.avg_speed_mbps > MIN_SEQUENTIAL_WRITE_SPEED);
            assert!(results.sequential_read.avg_speed_mbps > MIN_SEQUENTIAL_READ_SPEED);

            // Results are automatically tracked by criterion

            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_memory_copy,
    benchmark_sequential_operations,
    benchmark_random_operations,
    benchmark_statistics_overhead,
    benchmark_config_operations,
    benchmark_full_regression_suite
);

criterion_main!(benches);
