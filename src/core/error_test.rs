//! Comprehensive unit tests for BenchmarkError and error handling

#[cfg(test)]
mod tests {
    use super::super::BenchmarkError;
    use crate::platform::PlatformError;
    use std::path::PathBuf;
    use std::io;

    #[test]
    fn test_benchmark_error_platform_error() {
        let platform_error = PlatformError::UnsupportedPlatform("test platform".to_string());
        let benchmark_error = BenchmarkError::PlatformError(platform_error);
        
        let error_string = benchmark_error.to_string();
        assert!(error_string.contains("Platform error"));
        assert!(error_string.contains("test platform"));
    }

    #[test]
    fn test_benchmark_error_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let benchmark_error = BenchmarkError::IoError(io_error);
        
        let error_string = benchmark_error.to_string();
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("Access denied"));
    }

    #[test]
    fn test_benchmark_error_configuration_error() {
        let config_error = BenchmarkError::ConfigurationError("Invalid block size".to_string());
        
        let error_string = config_error.to_string();
        assert!(error_string.contains("Configuration error"));
        assert!(error_string.contains("Invalid block size"));
    }

    #[test]
    fn test_benchmark_error_insufficient_space() {
        let space_error = BenchmarkError::InsufficientSpace {
            required: 1024 * 1024 * 1024, // 1GB
            available: 512 * 1024 * 1024,  // 512MB
        };
        
        let error_string = space_error.to_string();
        assert!(error_string.contains("Insufficient space"));
        assert!(error_string.contains("1073741824")); // 1GB in bytes
        assert!(error_string.contains("536870912"));  // 512MB in bytes
    }

    #[test]
    fn test_benchmark_error_permission_denied() {
        let path = PathBuf::from("/restricted/path");
        let permission_error = BenchmarkError::PermissionDenied(path.clone());
        
        let error_string = permission_error.to_string();
        assert!(error_string.contains("Permission denied"));
        assert!(error_string.contains("/restricted/path"));
    }

    #[test]
    fn test_benchmark_error_test_interrupted() {
        let interrupt_error = BenchmarkError::TestInterrupted("User cancelled".to_string());
        
        let error_string = interrupt_error.to_string();
        assert!(error_string.contains("Test interrupted"));
        assert!(error_string.contains("User cancelled"));
    }

    #[test]
    fn test_benchmark_error_from_platform_error() {
        let platform_error = PlatformError::DirectIoNotSupported;
        let benchmark_error: BenchmarkError = platform_error.into();
        
        match benchmark_error {
            BenchmarkError::PlatformError(pe) => {
                assert!(matches!(pe, PlatformError::DirectIoNotSupported));
            }
            _ => panic!("Expected PlatformError variant"),
        }
    }

    #[test]
    fn test_benchmark_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let benchmark_error: BenchmarkError = io_error.into();
        
        match benchmark_error {
            BenchmarkError::IoError(ioe) => {
                assert_eq!(ioe.kind(), io::ErrorKind::NotFound);
                assert!(ioe.to_string().contains("File not found"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_benchmark_error_debug_format() {
        let config_error = BenchmarkError::ConfigurationError("Test error".to_string());
        let debug_str = format!("{:?}", config_error);
        
        assert!(debug_str.contains("ConfigurationError"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_benchmark_error_chain() {
        // Test error chaining with nested errors
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let benchmark_error = BenchmarkError::IoError(io_error);
        
        // Test that the error chain is preserved
        let error_string = benchmark_error.to_string();
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("Access denied"));
        
        // Test error source
        let source = std::error::Error::source(&benchmark_error);
        assert!(source.is_some());
    }

    #[test]
    fn test_platform_error_variants() {
        let errors = vec![
            PlatformError::UnsupportedPlatform("Windows XP".to_string()),
            PlatformError::DeviceEnumerationFailed("No devices found".to_string()),
            PlatformError::DirectIoNotSupported,
            PlatformError::InsufficientPermissions("Need admin rights".to_string()),
        ];
        
        for error in errors {
            let benchmark_error = BenchmarkError::PlatformError(error);
            let error_string = benchmark_error.to_string();
            assert!(error_string.contains("Platform error"));
        }
    }

    #[test]
    fn test_io_error_kinds() {
        let io_error_kinds = vec![
            io::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::InvalidData,
            io::ErrorKind::TimedOut,
            io::ErrorKind::WriteZero,
            io::ErrorKind::Interrupted,
            io::ErrorKind::UnexpectedEof,
        ];
        
        for kind in io_error_kinds {
            let io_error = io::Error::new(kind, "Test error");
            let benchmark_error = BenchmarkError::IoError(io_error);
            
            let error_string = benchmark_error.to_string();
            assert!(error_string.contains("IO error"));
            assert!(error_string.contains("Test error"));
        }
    }

    #[test]
    fn test_error_with_empty_messages() {
        let config_error = BenchmarkError::ConfigurationError(String::new());
        let error_string = config_error.to_string();
        assert!(error_string.contains("Configuration error"));
        
        let interrupt_error = BenchmarkError::TestInterrupted(String::new());
        let error_string = interrupt_error.to_string();
        assert!(error_string.contains("Test interrupted"));
    }

    #[test]
    fn test_error_with_special_characters() {
        let special_message = "Error with special chars: !@#$%^&*()[]{}|\\:;\"'<>,.?/~`";
        let config_error = BenchmarkError::ConfigurationError(special_message.to_string());
        
        let error_string = config_error.to_string();
        assert!(error_string.contains(special_message));
    }

    #[test]
    fn test_error_with_unicode() {
        let unicode_message = "Error with unicode: 测试 🚀 ñoño";
        let config_error = BenchmarkError::ConfigurationError(unicode_message.to_string());
        
        let error_string = config_error.to_string();
        assert!(error_string.contains(unicode_message));
    }

    #[test]
    fn test_insufficient_space_edge_cases() {
        // Test with zero values
        let error1 = BenchmarkError::InsufficientSpace {
            required: 0,
            available: 0,
        };
        let error_string = error1.to_string();
        assert!(error_string.contains("required 0 bytes"));
        assert!(error_string.contains("available 0 bytes"));
        
        // Test with very large values
        let error2 = BenchmarkError::InsufficientSpace {
            required: u64::MAX,
            available: u64::MAX - 1,
        };
        let error_string = error2.to_string();
        assert!(error_string.contains("Insufficient space"));
    }

    #[test]
    fn test_permission_denied_with_various_paths() {
        let paths = vec![
            PathBuf::from("/"),
            PathBuf::from("/root"),
            PathBuf::from("C:\\Windows\\System32"),
            PathBuf::from("relative/path"),
            PathBuf::from(""),
            PathBuf::from("path with spaces"),
            PathBuf::from("path/with/unicode/测试"),
        ];
        
        for path in paths {
            let error = BenchmarkError::PermissionDenied(path.clone());
            let error_string = error.to_string();
            assert!(error_string.contains("Permission denied"));
            if !path.as_os_str().is_empty() {
                assert!(error_string.contains(&path.display().to_string()));
            }
        }
    }

    #[test]
    fn test_error_result_propagation() {
        // Test that errors can be properly propagated through Result types
        fn test_function() -> Result<(), BenchmarkError> {
            Err(BenchmarkError::ConfigurationError("Test".to_string()))
        }
        
        let result = test_function();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BenchmarkError::ConfigurationError(msg) => {
                assert_eq!(msg, "Test");
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_error_in_option_context() {
        // Test error handling in Option contexts
        let error = BenchmarkError::TestInterrupted("Cancelled".to_string());
        assert!(error.to_string().contains("Cancelled"));
    }

    #[test]
    fn test_error_comparison() {
        // Test that we can match on error variants
        let error1 = BenchmarkError::ConfigurationError("Test".to_string());
        let error2 = BenchmarkError::TestInterrupted("Test".to_string());
        
        match error1 {
            BenchmarkError::ConfigurationError(_) => {
                // Expected
            }
            _ => panic!("Expected ConfigurationError"),
        }
        
        match error2 {
            BenchmarkError::TestInterrupted(_) => {
                // Expected
            }
            _ => panic!("Expected TestInterrupted"),
        }
    }

    #[test]
    fn test_error_with_nested_io_errors() {
        // Test various nested IO error scenarios
        let nested_errors = vec![
            io::Error::new(io::ErrorKind::NotFound, "File not found"),
            io::Error::new(io::ErrorKind::PermissionDenied, "Access denied"),
            io::Error::new(io::ErrorKind::AlreadyExists, "File exists"),
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid input"),
        ];
        
        for io_err in nested_errors {
            let benchmark_err = BenchmarkError::IoError(io_err);
            
            // Should be able to access the inner error
            match benchmark_err {
                BenchmarkError::IoError(inner) => {
                    assert!(!inner.to_string().is_empty());
                }
                _ => panic!("Expected IoError"),
            }
        }
    }

    #[test]
    fn test_error_send_sync_traits() {
        // Test that errors implement Send and Sync for thread safety
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<BenchmarkError>();
        assert_sync::<BenchmarkError>();
        assert_send::<PlatformError>();
        assert_sync::<PlatformError>();
    }
}