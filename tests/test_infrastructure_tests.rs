//! Integration tests for the test infrastructure itself
//! 
//! These tests verify that our test utilities and infrastructure work correctly
//! and can be relied upon for testing the main functionality.

#[cfg(feature = "test-utils")]
use disk_speed_test::test_utils::{
    TestEnvironment, TestEnvironmentBuilder, TestDataManager,
    test_data::{TestDataGenerator, TestDataPattern, TestDataVerifier},
    cleanup::{CleanupGuard, utils as cleanup_utils}
};
#[cfg(feature = "test-utils")]
use std::time::Duration;
#[cfg(feature = "test-utils")]
use std::fs;
#[cfg(feature = "test-utils")]
use tempfile::TempDir;

#[cfg(feature = "test-utils")]
#[test]
fn test_data_manager_lifecycle() {
    let mut manager = TestDataManager::new().unwrap();
    
    // Test file creation
    let file_path = manager.create_test_file("test.dat", 1024).unwrap();
    assert!(file_path.exists());
    assert_eq!(fs::metadata(&file_path).unwrap().len(), 1024);
    
    // Test random file creation
    let random_file = manager.create_random_test_file("random.dat", 2048).unwrap();
    assert!(random_file.exists());
    assert_eq!(fs::metadata(&random_file).unwrap().len(), 2048);
    
    // Test file listing
    let files = manager.test_files();
    assert_eq!(files.len(), 2);
    
    // Test cleanup
    manager.cleanup_file(&file_path).unwrap();
    assert!(!file_path.exists());
    assert_eq!(manager.test_files().len(), 1);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_data_generator_patterns() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test zeros pattern
    let mut generator = TestDataGenerator::new(TestDataPattern::Zeros);
    let zeros_file = temp_dir.path().join("zeros.dat");
    generator.generate_file(&zeros_file, 1024).unwrap();
    
    let verifier = TestDataVerifier::new(TestDataPattern::Zeros);
    assert!(verifier.verify_file(&zeros_file).unwrap());
    
    // Test ones pattern
    let mut generator = TestDataGenerator::new(TestDataPattern::Ones);
    let ones_file = temp_dir.path().join("ones.dat");
    generator.generate_file(&ones_file, 1024).unwrap();
    
    let verifier = TestDataVerifier::new(TestDataPattern::Ones);
    assert!(verifier.verify_file(&ones_file).unwrap());
    
    // Test sequential pattern
    let mut generator = TestDataGenerator::new(TestDataPattern::Sequential);
    let seq_file = temp_dir.path().join("sequential.dat");
    generator.generate_file(&seq_file, 512).unwrap();
    
    let verifier = TestDataVerifier::new(TestDataPattern::Sequential);
    assert!(verifier.verify_file(&seq_file).unwrap());
}

#[cfg(feature = "test-utils")]
#[test]
fn test_seeded_random_reproducibility() {
    let temp_dir = TempDir::new().unwrap();
    
    // Generate two files with the same seed
    let mut generator1 = TestDataGenerator::new(TestDataPattern::RandomSeeded(12345));
    let mut generator2 = TestDataGenerator::new(TestDataPattern::RandomSeeded(12345));
    
    let file1 = temp_dir.path().join("random1.dat");
    let file2 = temp_dir.path().join("random2.dat");
    
    generator1.generate_file(&file1, 1024).unwrap();
    generator2.generate_file(&file2, 1024).unwrap();
    
    // Files should be identical
    let data1 = fs::read(&file1).unwrap();
    let data2 = fs::read(&file2).unwrap();
    assert_eq!(data1, data2);
    
    // Generate with different seed - should be different
    let mut generator3 = TestDataGenerator::new(TestDataPattern::RandomSeeded(54321));
    let file3 = temp_dir.path().join("random3.dat");
    generator3.generate_file(&file3, 1024).unwrap();
    
    let data3 = fs::read(&file3).unwrap();
    assert_ne!(data1, data3);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_environment_builder() {
    let env = TestEnvironmentBuilder::new()
        .min_free_space(10 * 1024 * 1024) // 10MB
        .max_test_duration(Duration::from_secs(30))
        .use_small_files(true)
        .skip_privileged_tests(true)
        .build()
        .unwrap();
    
    // Test configuration
    assert_eq!(env.config().min_free_space, 10 * 1024 * 1024);
    assert_eq!(env.config().max_test_duration, Duration::from_secs(30));
    assert!(env.config().use_small_files);
    assert!(env.config().skip_privileged_tests);
    
    // Test benchmark config creation
    let benchmark_config = env.create_test_benchmark_config(None);
    assert_eq!(benchmark_config.sequential_block_size, 64 * 1024); // Small files mode
    assert_eq!(benchmark_config.test_duration_seconds, 1);
    assert_eq!(benchmark_config.file_size_mb, 1);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_environment_requirements_check() {
    let env = TestEnvironmentBuilder::new()
        .min_free_space(1024 * 1024 * 1024 * 1024) // 1TB - likely to fail
        .build()
        .unwrap();
    
    // Should fail due to insufficient space
    assert!(env.check_requirements().is_err());
    
    let env = TestEnvironmentBuilder::new()
        .min_free_space(1024) // 1KB - should pass
        .build()
        .unwrap();
    
    // Should pass
    assert!(env.check_requirements().is_ok());
}

#[cfg(feature = "test-utils")]
#[test]
fn test_cleanup_guard() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("cleanup_test.txt");
    
    // Create file
    fs::write(&test_file, "test content").unwrap();
    assert!(test_file.exists());
    
    {
        let _guard = CleanupGuard::for_file(&test_file);
        assert!(test_file.exists()); // Still exists while guard is alive
    }
    
    // File should be cleaned up after guard is dropped
    assert!(!test_file.exists());
}

#[cfg(feature = "test-utils")]
#[test]
fn test_cleanup_pattern_matching() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test files
    fs::write(temp_dir.path().join("test_file_1.tmp"), "content").unwrap();
    fs::write(temp_dir.path().join("test_file_2.tmp"), "content").unwrap();
    fs::write(temp_dir.path().join("other_file.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("test_dir_1.tmp"), "content").unwrap();
    
    // Clean up files matching pattern
    let cleaned = cleanup_utils::cleanup_pattern(temp_dir.path(), "test_file").unwrap();
    assert_eq!(cleaned, 2); // Should clean test_file_1.tmp and test_file_2.tmp
    
    // Verify cleanup
    assert!(!temp_dir.path().join("test_file_1.tmp").exists());
    assert!(!temp_dir.path().join("test_file_2.tmp").exists());
    assert!(temp_dir.path().join("other_file.txt").exists()); // Should remain
}

#[cfg(feature = "test-utils")]
#[test]
fn test_environment_result_recording() {
    use disk_speed_test::core::stats::TestResult;
    
    let env = TestEnvironment::with_defaults().unwrap();
    
    let test_result = TestResult {
        min_speed_mbps: 10.0,
        max_speed_mbps: 100.0,
        avg_speed_mbps: 50.0,
        test_duration: Duration::from_secs(5),
        sample_count: 100,
    };
    
    env.record_test_result("test_benchmark".to_string(), test_result.clone());
    env.record_test_result("another_test".to_string(), test_result.clone());
    
    let results = env.get_test_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "test_benchmark");
    assert_eq!(results[1].0, "another_test");
    assert_eq!(results[0].1.avg_speed_mbps, 50.0);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_environment_reset() {
    let mut env = TestEnvironment::with_defaults().unwrap();
    
    // Create some test files
    let _file1 = env.data_manager().create_test_file("test1.dat", 1024).unwrap();
    let _file2 = env.data_manager().create_test_file("test2.dat", 2048).unwrap();
    
    assert_eq!(env.data_manager().test_files().len(), 2);
    
    // Reset environment
    env.reset_for_next_test().unwrap();
    
    // Files should be cleaned up
    assert_eq!(env.data_manager().test_files().len(), 0);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_timeout_guard() {
    use disk_speed_test::test_utils::test_environment::TimeoutGuard;
    
    let guard = TimeoutGuard::new("test_operation".to_string(), Duration::from_millis(100));
    
    // Should not timeout immediately
    assert!(guard.check_timeout().is_ok());
    assert!(guard.remaining_time() > Duration::from_millis(50));
    
    // Wait for timeout
    std::thread::sleep(Duration::from_millis(150));
    
    // Should timeout now
    assert!(guard.check_timeout().is_err());
    assert_eq!(guard.remaining_time(), Duration::ZERO);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_sparse_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let generator = TestDataGenerator::new(TestDataPattern::Zeros);
    let sparse_file = temp_dir.path().join("sparse.dat");
    
    // Create 1MB sparse file
    generator.create_sparse_file(&sparse_file, 1024 * 1024).unwrap();
    
    assert!(sparse_file.exists());
    assert_eq!(fs::metadata(&sparse_file).unwrap().len(), 1024 * 1024);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_pattern_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let mut generator = TestDataGenerator::new(TestDataPattern::Sequential);
    let pattern_file = temp_dir.path().join("pattern.dat");
    
    // Create file with data at specific offsets
    let data_chunks = vec![
        (0, 1024),      // First 1KB
        (2048, 1024),   // 1KB starting at offset 2KB
        (4096, 512),    // 512B starting at offset 4KB
    ];
    
    generator.create_pattern_file(&pattern_file, 8192, &data_chunks).unwrap();
    
    assert!(pattern_file.exists());
    assert_eq!(fs::metadata(&pattern_file).unwrap().len(), 8192);
}

#[cfg(feature = "test-utils")]
#[test]
fn test_environment_skip_logic() {
    let env = TestEnvironmentBuilder::new()
        .skip_privileged_tests(true)
        .build()
        .unwrap();
    
    // Should skip privileged tests
    assert!(env.should_skip_test("direct_io_test"));
    assert!(env.should_skip_test("device_enumeration_test"));
    assert!(env.should_skip_test("system_drive_test"));
    
    // Should not skip regular tests
    assert!(!env.should_skip_test("regular_test"));
    assert!(!env.should_skip_test("unit_test"));
    
    let env = TestEnvironmentBuilder::new()
        .skip_privileged_tests(false)
        .build()
        .unwrap();
    
    // Should not skip any tests
    assert!(!env.should_skip_test("direct_io_test"));
    assert!(!env.should_skip_test("regular_test"));
}