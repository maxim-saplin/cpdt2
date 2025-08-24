//! Comprehensive unit tests for BenchmarkConfig

#[cfg(test)]
mod tests {
    use super::super::config::*;
    use super::super::BenchmarkError;
    use std::path::PathBuf;
    use std::env;
    use std::fs;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        
        assert_eq!(config.target_path, PathBuf::from("."));
        assert_eq!(config.sequential_block_size, 4 * 1024 * 1024); // 4MB
        assert_eq!(config.random_block_size, 4 * 1024); // 4KB
        assert_eq!(config.test_duration_seconds, 10);
        assert!(config.disable_os_cache);
        assert_eq!(config.file_size_mb, 1024); // 1GB
    }

    #[test]
    fn test_benchmark_config_new() {
        let test_path = PathBuf::from("/tmp/test");
        let config = BenchmarkConfig::new(test_path.clone());
        
        assert_eq!(config.target_path, test_path);
        // Other fields should be defaults
        assert_eq!(config.sequential_block_size, 4 * 1024 * 1024);
        assert_eq!(config.random_block_size, 4 * 1024);
        assert_eq!(config.test_duration_seconds, 10);
        assert!(config.disable_os_cache);
        assert_eq!(config.file_size_mb, 1024);
    }

    #[test]
    fn test_file_size_bytes_calculation() {
        let mut config = BenchmarkConfig::default();
        
        config.file_size_mb = 1;
        assert_eq!(config.file_size_bytes(), 1024 * 1024);
        
        config.file_size_mb = 100;
        assert_eq!(config.file_size_bytes(), 100 * 1024 * 1024);
        
        config.file_size_mb = 1024;
        assert_eq!(config.file_size_bytes(), 1024 * 1024 * 1024);
        
        // Test edge case
        config.file_size_mb = 0;
        assert_eq!(config.file_size_bytes(), 0);
    }

    #[test]
    fn test_config_validation_success() {
        let temp_dir = env::temp_dir();
        let config = BenchmarkConfig::new(temp_dir);
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_zero_sequential_block_size() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        config.sequential_block_size = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("Sequential block size must be greater than 0"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_validation_zero_random_block_size() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        config.random_block_size = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("Random block size must be greater than 0"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_validation_zero_test_duration() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        config.test_duration_seconds = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("Test duration must be greater than 0"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_validation_zero_file_size() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        config.file_size_mb = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("File size must be greater than 0"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_validation_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let config = BenchmarkConfig::new(nonexistent_path.clone());
        
        let result = config.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("Target path does not exist"));
                assert!(msg.contains(&nonexistent_path.display().to_string()));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_validation_multiple_errors() {
        let nonexistent_path = PathBuf::from("/nonexistent/path");
        let mut config = BenchmarkConfig::new(nonexistent_path);
        config.sequential_block_size = 0;
        config.random_block_size = 0;
        config.test_duration_seconds = 0;
        config.file_size_mb = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        // Should fail on the first validation error (sequential block size)
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert!(msg.contains("Sequential block size must be greater than 0"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_config_with_custom_values() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        
        config.sequential_block_size = 8 * 1024 * 1024; // 8MB
        config.random_block_size = 8 * 1024; // 8KB
        config.test_duration_seconds = 30;
        config.disable_os_cache = false;
        config.file_size_mb = 2048; // 2GB
        
        assert!(config.validate().is_ok());
        assert_eq!(config.sequential_block_size, 8 * 1024 * 1024);
        assert_eq!(config.random_block_size, 8 * 1024);
        assert_eq!(config.test_duration_seconds, 30);
        assert!(!config.disable_os_cache);
        assert_eq!(config.file_size_mb, 2048);
        assert_eq!(config.file_size_bytes(), 2048 * 1024 * 1024);
    }

    #[test]
    fn test_config_extreme_values() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        
        // Test with very large values
        config.sequential_block_size = 1024 * 1024 * 1024; // 1GB block
        config.random_block_size = 1024 * 1024; // 1MB block
        config.test_duration_seconds = 3600; // 1 hour
        config.file_size_mb = 10240; // 10GB
        
        assert!(config.validate().is_ok());
        
        // Test with very small values
        config.sequential_block_size = 1; // 1 byte
        config.random_block_size = 1; // 1 byte
        config.test_duration_seconds = 1; // 1 second
        config.file_size_mb = 1; // 1MB
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let temp_dir = env::temp_dir();
        let config = BenchmarkConfig::new(temp_dir.clone());
        
        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(&temp_dir.display().to_string()));
        assert!(json.contains("4194304")); // 4MB in bytes
        assert!(json.contains("4096")); // 4KB in bytes
        
        // Test deserialization
        let deserialized: BenchmarkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.target_path, config.target_path);
        assert_eq!(deserialized.sequential_block_size, config.sequential_block_size);
        assert_eq!(deserialized.random_block_size, config.random_block_size);
        assert_eq!(deserialized.test_duration_seconds, config.test_duration_seconds);
        assert_eq!(deserialized.disable_os_cache, config.disable_os_cache);
        assert_eq!(deserialized.file_size_mb, config.file_size_mb);
    }

    #[test]
    fn test_config_clone() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir.clone());
        config.sequential_block_size = 8 * 1024 * 1024;
        config.test_duration_seconds = 30;
        
        let cloned = config.clone();
        
        assert_eq!(cloned.target_path, config.target_path);
        assert_eq!(cloned.sequential_block_size, config.sequential_block_size);
        assert_eq!(cloned.random_block_size, config.random_block_size);
        assert_eq!(cloned.test_duration_seconds, config.test_duration_seconds);
        assert_eq!(cloned.disable_os_cache, config.disable_os_cache);
        assert_eq!(cloned.file_size_mb, config.file_size_mb);
    }

    #[test]
    fn test_config_debug_format() {
        let temp_dir = env::temp_dir();
        let config = BenchmarkConfig::new(temp_dir);
        
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("BenchmarkConfig"));
        assert!(debug_str.contains("target_path"));
        assert!(debug_str.contains("sequential_block_size"));
        assert!(debug_str.contains("random_block_size"));
    }

    #[test]
    fn test_config_with_temporary_directory() {
        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("disk_speed_test_config_test");
        fs::create_dir_all(&temp_dir).unwrap();
        
        let config = BenchmarkConfig::new(temp_dir.clone());
        assert!(config.validate().is_ok());
        
        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_config_validation_with_file_as_target() {
        // Create a temporary file
        let temp_file = env::temp_dir().join("test_file.tmp");
        fs::write(&temp_file, "test content").unwrap();
        
        let config = BenchmarkConfig::new(temp_file.clone());
        // Should be valid even if target is a file (the benchmark will use the parent directory)
        assert!(config.validate().is_ok());
        
        // Cleanup
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_config_edge_case_block_sizes() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        
        // Test when random block size > sequential block size
        config.sequential_block_size = 1024; // 1KB
        config.random_block_size = 4096; // 4KB
        
        assert!(config.validate().is_ok());
        
        // Test when sequential block size > file size
        config.sequential_block_size = 2 * 1024 * 1024 * 1024; // 2GB
        config.file_size_mb = 1024; // 1GB
        
        // This should still be valid - the implementation should handle it
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_boundary_values() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        
        // Test boundary values that should be valid
        config.sequential_block_size = 1;
        config.random_block_size = 1;
        config.test_duration_seconds = 1;
        config.file_size_mb = 1;
        
        assert!(config.validate().is_ok());
        
        // Test maximum reasonable values
        config.sequential_block_size = usize::MAX / 2; // Large but not overflow-prone
        config.random_block_size = usize::MAX / 2;
        config.test_duration_seconds = u64::MAX / 2;
        config.file_size_mb = usize::MAX / (1024 * 1024 * 2); // Avoid overflow in file_size_bytes
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_file_size_bytes_overflow_protection() {
        let temp_dir = env::temp_dir();
        let mut config = BenchmarkConfig::new(temp_dir);
        
        // Test with a reasonable large value that won't overflow
        config.file_size_mb = 1000000; // 1TB
        
        // The calculation should work without overflow
        let result = config.file_size_bytes();
        assert_eq!(result, 1000000 * 1024 * 1024);
        
        // The validation should still work
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
    }
}