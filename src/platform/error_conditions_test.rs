//! Comprehensive platform error condition tests

#[cfg(test)]
mod tests {
    use super::super::mock_platform::{MockFileResult, MockPlatform};
    use super::super::{DeviceType, PlatformError, StorageDevice};
    use std::path::PathBuf;

    #[test]
    fn test_device_enumeration_errors() {
        let mock = MockPlatform::new();

        // Test various device enumeration failure scenarios
        let error_scenarios = vec![
            PlatformError::DeviceEnumerationFailed("No devices found".to_string()),
            PlatformError::DeviceEnumerationFailed("Access denied to device list".to_string()),
            PlatformError::InsufficientPermissions("Need administrator privileges".to_string()),
            PlatformError::UnsupportedPlatform("Unknown operating system".to_string()),
        ];

        for error in error_scenarios {
            mock.simulate_error(error.clone());

            let result = mock.list_storage_devices_instance();
            assert!(result.is_err(), "Should fail with simulated error");

            let actual_error = result.unwrap_err();
            match (&error, &actual_error) {
                (
                    PlatformError::DeviceEnumerationFailed(expected),
                    PlatformError::DeviceEnumerationFailed(actual),
                ) => {
                    assert_eq!(expected, actual);
                }
                (
                    PlatformError::InsufficientPermissions(expected),
                    PlatformError::InsufficientPermissions(actual),
                ) => {
                    assert_eq!(expected, actual);
                }
                (
                    PlatformError::UnsupportedPlatform(expected),
                    PlatformError::UnsupportedPlatform(actual),
                ) => {
                    assert_eq!(expected, actual);
                }
                _ => panic!("Error type mismatch"),
            }

            mock.disable_error_simulation();
        }
    }

    #[test]
    fn test_app_data_directory_errors() {
        let mock = MockPlatform::new();

        // Test app data directory access failures
        let error_scenarios = vec![
            PlatformError::InsufficientPermissions("Cannot create app data directory".to_string()),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not found",
            )),
            PlatformError::UnsupportedPlatform("App data directory not supported".to_string()),
        ];

        for error in error_scenarios {
            mock.simulate_error(error.clone());

            let result = mock.get_app_data_dir_instance();
            assert!(result.is_err(), "Should fail with simulated error");

            mock.disable_error_simulation();
        }
    }

    #[test]
    fn test_direct_io_file_creation_errors() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/test/direct_io_error.bin");

        // Test various file creation error scenarios
        let error_scenarios = vec![
            PlatformError::DirectIoNotSupported,
            PlatformError::InsufficientPermissions("Cannot create file".to_string()),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Access denied",
            )),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Directory not found",
            )),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File already exists",
            )),
        ];

        for error in error_scenarios {
            mock.set_file_operation_result(test_path.clone(), MockFileResult::Error(error.clone()));

            let result = mock.create_direct_io_file_instance(&test_path, 1024);
            assert!(result.is_err(), "Should fail with simulated error");

            let actual_error = result.unwrap_err();
            match (&error, &actual_error) {
                (PlatformError::DirectIoNotSupported, PlatformError::DirectIoNotSupported) => {
                    // Expected
                }
                (
                    PlatformError::InsufficientPermissions(expected),
                    PlatformError::InsufficientPermissions(actual),
                ) => {
                    assert_eq!(expected, actual);
                }
                (PlatformError::IoError(_), PlatformError::IoError(_)) => {
                    // IO errors are expected to match in kind
                }
                _ => panic!(
                    "Error type mismatch: expected {:?}, got {:?}",
                    error, actual_error
                ),
            }
        }
    }

    #[test]
    fn test_direct_io_file_opening_errors() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/test/open_error.bin");

        // Test file opening error scenarios
        let error_scenarios = vec![
            PlatformError::DirectIoNotSupported,
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission denied",
            )),
        ];

        for error in error_scenarios {
            mock.set_file_operation_result(test_path.clone(), MockFileResult::Error(error.clone()));

            // Test both read and write opening
            let read_result = mock.open_direct_io_file_instance(&test_path, false);
            assert!(
                read_result.is_err(),
                "Read opening should fail with simulated error"
            );

            let write_result = mock.open_direct_io_file_instance(&test_path, true);
            assert!(
                write_result.is_err(),
                "Write opening should fail with simulated error"
            );
        }
    }

    #[test]
    fn test_file_system_sync_errors() {
        let mock = MockPlatform::new();

        // Test file system sync error scenarios
        let error_scenarios = vec![
            PlatformError::InsufficientPermissions("Cannot sync filesystem".to_string()),
            PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found for sync",
            )),
            PlatformError::UnsupportedPlatform("Sync not supported".to_string()),
        ];

        for error in error_scenarios {
            mock.simulate_error(error.clone());

            let result = mock.sync_file_system_instance(&PathBuf::from("/test"));
            assert!(result.is_err(), "Should fail with simulated error");

            mock.disable_error_simulation();
        }
    }

    #[test]
    fn test_error_recovery_scenarios() {
        let mock = MockPlatform::new();

        // Test that operations can recover after errors
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "Temporary failure".to_string(),
        ));

        // First call should fail
        let result1 = mock.list_storage_devices_instance();
        assert!(result1.is_err());

        // Disable error simulation
        mock.disable_error_simulation();

        // Second call should succeed
        let result2 = mock.list_storage_devices_instance();
        assert!(result2.is_ok());
        assert!(!result2.unwrap().is_empty());
    }

    #[test]
    fn test_concurrent_error_handling() {
        use std::sync::Arc;
        use std::thread;

        let mock = Arc::new(MockPlatform::new());

        // Simulate error in one thread
        mock.simulate_error(PlatformError::DirectIoNotSupported);

        let mut handles = vec![];

        // Start multiple threads that should all see the error
        for _ in 0..5 {
            let mock_clone = mock.clone();
            let handle = thread::spawn(move || {
                let result = mock_clone.list_storage_devices_instance();
                assert!(result.is_err(), "All threads should see the error");
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Disable error and verify recovery
        mock.disable_error_simulation();
        let result = mock.list_storage_devices_instance();
        assert!(
            result.is_ok(),
            "Should recover after disabling error simulation"
        );
    }

    #[test]
    fn test_platform_specific_error_messages() {
        let mock = MockPlatform::new();

        // Test Windows-style error messages
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "GetLogicalDrives() failed with error code 5".to_string(),
        ));

        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("GetLogicalDrives"));

        mock.disable_error_simulation();

        // Test macOS-style error messages
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "Failed to read /Volumes: Operation not permitted".to_string(),
        ));

        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("/Volumes"));

        mock.disable_error_simulation();

        // Test Linux-style error messages
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "Failed to open /proc/mounts: Permission denied".to_string(),
        ));

        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("/proc/mounts"));
    }

    #[test]
    fn test_error_chain_propagation() {
        let mock = MockPlatform::new();

        // Create a nested IO error
        let io_error = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Low-level permission error",
        );
        let platform_error = PlatformError::IoError(io_error);

        mock.simulate_error(platform_error);

        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            PlatformError::IoError(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::PermissionDenied);
                assert!(io_err.to_string().contains("Low-level permission error"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_invalid_device_data_handling() {
        let mock = MockPlatform::new();
        mock.clear_devices();

        // Add device with invalid data
        mock.add_device(StorageDevice {
            name: String::new(),         // Empty name
            mount_point: PathBuf::new(), // Empty path
            total_space: 0,
            available_space: 1024, // Available > total (invalid)
            device_type: DeviceType::Unknown,
        });

        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 1);

        let device = &devices[0];
        assert!(device.name.is_empty());
        assert!(device.available_space > device.total_space); // Invalid but preserved
    }

    #[test]
    fn test_large_file_size_errors() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/test/huge_file.bin");

        // Simulate error for very large file
        mock.set_file_operation_result(
            test_path.clone(),
            MockFileResult::Error(PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "File too large",
            ))),
        );

        let huge_size = u64::MAX; // Maximum possible size
        let result = mock.create_direct_io_file_instance(&test_path, huge_size);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::IoError(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::OutOfMemory);
            }
            _ => panic!("Expected IoError with OutOfMemory"),
        }
    }

    #[test]
    fn test_unicode_path_errors() {
        let mock = MockPlatform::new();
        let unicode_path = PathBuf::from("/test/测试文件/файл.bin");

        // Simulate unicode handling error
        mock.set_file_operation_result(
            unicode_path.clone(),
            MockFileResult::Error(PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid unicode in path",
            ))),
        );

        let result = mock.create_direct_io_file_instance(&unicode_path, 1024);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::IoError(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidInput);
                assert!(io_err.to_string().contains("unicode"));
            }
            _ => panic!("Expected IoError with InvalidInput"),
        }
    }

    #[test]
    fn test_network_filesystem_errors() {
        let mock = MockPlatform::new();
        mock.clear_devices();

        // Add network device that might have connectivity issues
        mock.add_device(StorageDevice {
            name: "Network Share".to_string(),
            mount_point: PathBuf::from("/mnt/network"),
            total_space: 1024 * 1024 * 1024 * 1024,    // 1TB
            available_space: 512 * 1024 * 1024 * 1024, // 512GB
            device_type: DeviceType::Network,
        });

        // Simulate network error
        mock.simulate_error(PlatformError::IoError(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Network timeout",
        )));

        let result = mock.get_app_data_dir_instance();
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::IoError(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::TimedOut);
            }
            _ => panic!("Expected IoError with TimedOut"),
        }
    }

    #[test]
    fn test_removable_device_errors() {
        let mock = MockPlatform::new();
        let usb_path = PathBuf::from("/media/usb/test.bin");

        // Simulate removable device disconnection
        mock.set_file_operation_result(
            usb_path.clone(),
            MockFileResult::Error(PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Device not found",
            ))),
        );

        let result = mock.create_direct_io_file_instance(&usb_path, 1024);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::IoError(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
                assert!(io_err.to_string().contains("Device not found"));
            }
            _ => panic!("Expected IoError with NotFound"),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        // Test that all error types have proper Display implementations
        let errors = vec![
            PlatformError::UnsupportedPlatform("Test platform".to_string()),
            PlatformError::DeviceEnumerationFailed("Test enumeration".to_string()),
            PlatformError::DirectIoNotSupported,
            PlatformError::InsufficientPermissions("Test permissions".to_string()),
            PlatformError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Test IO")),
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(
                !error_string.is_empty(),
                "Error should have non-empty display string"
            );
            assert!(error_string.len() > 5, "Error string should be descriptive");

            // Test debug formatting
            let debug_string = format!("{:?}", error);
            assert!(
                !debug_string.is_empty(),
                "Error should have debug representation"
            );
        }
    }

    #[test]
    fn test_error_source_chain() {
        // Test error source chain for IoError
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let platform_error = PlatformError::IoError(io_error);

        // Test that we can access the source error
        let source = std::error::Error::source(&platform_error);
        assert!(source.is_some(), "IoError should have a source");

        let source_str = source.unwrap().to_string();
        assert!(source_str.contains("Access denied"));
    }
}
