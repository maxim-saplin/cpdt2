//! Comprehensive unit tests for platform abstraction layer

#[cfg(test)]
mod tests {
    use super::super::{PlatformOps, StorageDevice, DeviceType, PlatformError};
    use super::super::mock_platform::{MockPlatform, MockFileResult};
    use std::path::PathBuf;
    use std::io;

    #[test]
    fn test_storage_device_creation() {
        let device = StorageDevice {
            name: "Test Drive".to_string(),
            mount_point: PathBuf::from("/test"),
            total_space: 1024 * 1024 * 1024, // 1GB
            available_space: 512 * 1024 * 1024, // 512MB
            device_type: DeviceType::SolidState,
        };
        
        assert_eq!(device.name, "Test Drive");
        assert_eq!(device.mount_point, PathBuf::from("/test"));
        assert_eq!(device.total_space, 1024 * 1024 * 1024);
        assert_eq!(device.available_space, 512 * 1024 * 1024);
        assert_eq!(device.device_type, DeviceType::SolidState);
    }

    #[test]
    fn test_device_type_variants() {
        let device_types = vec![
            DeviceType::HardDisk,
            DeviceType::SolidState,
            DeviceType::Removable,
            DeviceType::Network,
            DeviceType::RamDisk,
            DeviceType::OpticalDisk,
            DeviceType::Unknown,
        ];
        
        for device_type in device_types {
            let device = StorageDevice {
                name: "Test".to_string(),
                mount_point: PathBuf::from("/test"),
                total_space: 1024,
                available_space: 512,
                device_type: device_type.clone(),
            };
            
            assert_eq!(device.device_type, device_type);
        }
    }

    #[test]
    fn test_device_type_equality() {
        assert_eq!(DeviceType::HardDisk, DeviceType::HardDisk);
        assert_eq!(DeviceType::SolidState, DeviceType::SolidState);
        assert_ne!(DeviceType::HardDisk, DeviceType::SolidState);
        assert_ne!(DeviceType::Removable, DeviceType::Network);
    }

    #[test]
    fn test_storage_device_serialization() {
        let device = StorageDevice {
            name: "Test Drive".to_string(),
            mount_point: PathBuf::from("/test"),
            total_space: 1024 * 1024 * 1024,
            available_space: 512 * 1024 * 1024,
            device_type: DeviceType::SolidState,
        };
        
        // Test JSON serialization
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("Test Drive"));
        assert!(json.contains("SolidState"));
        
        // Test deserialization
        let deserialized: StorageDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, device.name);
        assert_eq!(deserialized.mount_point, device.mount_point);
        assert_eq!(deserialized.total_space, device.total_space);
        assert_eq!(deserialized.available_space, device.available_space);
        assert_eq!(deserialized.device_type, device.device_type);
    }

    #[test]
    fn test_storage_device_clone() {
        let device = StorageDevice {
            name: "Original".to_string(),
            mount_point: PathBuf::from("/original"),
            total_space: 1024,
            available_space: 512,
            device_type: DeviceType::HardDisk,
        };
        
        let cloned = device.clone();
        assert_eq!(cloned.name, device.name);
        assert_eq!(cloned.mount_point, device.mount_point);
        assert_eq!(cloned.total_space, device.total_space);
        assert_eq!(cloned.available_space, device.available_space);
        assert_eq!(cloned.device_type, device.device_type);
    }

    #[test]
    fn test_storage_device_debug_format() {
        let device = StorageDevice {
            name: "Debug Test".to_string(),
            mount_point: PathBuf::from("/debug"),
            total_space: 2048,
            available_space: 1024,
            device_type: DeviceType::Unknown,
        };
        
        let debug_str = format!("{:?}", device);
        assert!(debug_str.contains("StorageDevice"));
        assert!(debug_str.contains("Debug Test"));
        assert!(debug_str.contains("/debug"));
        assert!(debug_str.contains("2048"));
        assert!(debug_str.contains("1024"));
        assert!(debug_str.contains("Unknown"));
    }

    #[test]
    fn test_platform_error_variants() {
        let errors = vec![
            PlatformError::UnsupportedPlatform("Test platform".to_string()),
            PlatformError::DeviceEnumerationFailed("No devices".to_string()),
            PlatformError::DirectIoNotSupported,
            PlatformError::InsufficientPermissions("Need admin".to_string()),
        ];
        
        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_platform_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let platform_error = PlatformError::IoError(io_error);
        
        let error_string = platform_error.to_string();
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("Access denied"));
    }

    #[test]
    fn test_platform_error_debug_format() {
        let error = PlatformError::UnsupportedPlatform("Test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("UnsupportedPlatform"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_mock_platform_ops_trait() {
        // Test that MockPlatform implements PlatformOps
        let devices = MockPlatform::list_storage_devices().unwrap();
        assert!(!devices.is_empty());
        
        let app_dir = MockPlatform::get_app_data_dir().unwrap();
        assert!(!app_dir.as_os_str().is_empty());
        
        let sync_result = MockPlatform::sync_file_system(&PathBuf::from("/test"));
        assert!(sync_result.is_ok());
    }

    #[test]
    fn test_mock_platform_device_enumeration() {
        let mock = MockPlatform::new();
        let devices = mock.list_storage_devices_instance().unwrap();
        
        assert_eq!(devices.len(), 3);
        
        // Check default devices
        assert_eq!(devices[0].name, "System Drive");
        assert_eq!(devices[0].device_type, DeviceType::SolidState);
        
        assert_eq!(devices[1].name, "Data Drive");
        assert_eq!(devices[1].device_type, DeviceType::HardDisk);
        
        assert_eq!(devices[2].name, "USB Drive");
        assert_eq!(devices[2].device_type, DeviceType::Removable);
    }

    #[test]
    fn test_mock_platform_error_simulation() {
        let mock = MockPlatform::new();
        
        // Test normal operation
        assert!(mock.list_storage_devices_instance().is_ok());
        assert!(mock.get_app_data_dir_instance().is_ok());
        
        // Simulate device enumeration error
        mock.simulate_error(PlatformError::DeviceEnumerationFailed("Test error".to_string()));
        
        let devices_result = mock.list_storage_devices_instance();
        assert!(devices_result.is_err());
        match devices_result.unwrap_err() {
            PlatformError::DeviceEnumerationFailed(msg) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected DeviceEnumerationFailed"),
        }
        
        // App data dir should also fail with the same error
        let app_dir_result = mock.get_app_data_dir_instance();
        assert!(app_dir_result.is_err());
    }

    #[test]
    fn test_mock_platform_file_operations_success() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/tmp/test_file.tmp");
        
        // Set up successful file operation
        mock.set_file_operation_result(test_path.clone(), MockFileResult::Success);
        
        // Create the actual file for the test
        std::fs::write(&test_path, "test content").unwrap();
        
        let create_result = mock.create_direct_io_file_instance(&test_path, 1024);
        assert!(create_result.is_ok());
        
        let open_result = mock.open_direct_io_file_instance(&test_path, false);
        assert!(open_result.is_ok());
        
        let open_write_result = mock.open_direct_io_file_instance(&test_path, true);
        assert!(open_write_result.is_ok());
        
        // Cleanup
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_mock_platform_file_operations_error() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/tmp/error_test_file.tmp");
        
        // Set up error for file operation
        mock.set_file_operation_result(
            test_path.clone(),
            MockFileResult::Error(PlatformError::DirectIoNotSupported)
        );
        
        let create_result = mock.create_direct_io_file_instance(&test_path, 1024);
        assert!(create_result.is_err());
        match create_result.unwrap_err() {
            PlatformError::DirectIoNotSupported => {
                // Expected
            }
            _ => panic!("Expected DirectIoNotSupported"),
        }
        
        let open_result = mock.open_direct_io_file_instance(&test_path, false);
        assert!(open_result.is_err());
    }

    #[test]
    fn test_mock_platform_sync_operations() {
        let mock = MockPlatform::new();
        
        // Test normal sync
        let sync_result = mock.sync_file_system_instance(&PathBuf::from("/test"));
        assert!(sync_result.is_ok());
        
        // Test sync with error simulation
        mock.simulate_error(PlatformError::InsufficientPermissions("No sync permission".to_string()));
        
        let sync_error_result = mock.sync_file_system_instance(&PathBuf::from("/test"));
        assert!(sync_error_result.is_err());
        match sync_error_result.unwrap_err() {
            PlatformError::InsufficientPermissions(msg) => {
                assert_eq!(msg, "No sync permission");
            }
            _ => panic!("Expected InsufficientPermissions"),
        }
    }

    #[test]
    fn test_device_space_calculations() {
        let device = StorageDevice {
            name: "Space Test".to_string(),
            mount_point: PathBuf::from("/space"),
            total_space: 1000,
            available_space: 600,
            device_type: DeviceType::HardDisk,
        };
        
        let used_space = device.total_space - device.available_space;
        assert_eq!(used_space, 400);
        
        let usage_percentage = (used_space as f64 / device.total_space as f64) * 100.0;
        assert!((usage_percentage - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_device_space_edge_cases() {
        // Test device with no available space
        let full_device = StorageDevice {
            name: "Full Drive".to_string(),
            mount_point: PathBuf::from("/full"),
            total_space: 1000,
            available_space: 0,
            device_type: DeviceType::HardDisk,
        };
        
        assert_eq!(full_device.available_space, 0);
        assert!(full_device.total_space > full_device.available_space);
        
        // Test device with all space available
        let empty_device = StorageDevice {
            name: "Empty Drive".to_string(),
            mount_point: PathBuf::from("/empty"),
            total_space: 1000,
            available_space: 1000,
            device_type: DeviceType::HardDisk,
        };
        
        assert_eq!(empty_device.available_space, empty_device.total_space);
        
        // Test zero-size device
        let zero_device = StorageDevice {
            name: "Zero Drive".to_string(),
            mount_point: PathBuf::from("/zero"),
            total_space: 0,
            available_space: 0,
            device_type: DeviceType::Unknown,
        };
        
        assert_eq!(zero_device.total_space, 0);
        assert_eq!(zero_device.available_space, 0);
    }

    #[test]
    fn test_platform_error_chain() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let platform_error = PlatformError::IoError(io_error);
        
        // Test error source chain
        let source = std::error::Error::source(&platform_error);
        assert!(source.is_some());
        
        let source_str = source.unwrap().to_string();
        assert!(source_str.contains("File not found"));
    }

    #[test]
    fn test_device_type_serialization() {
        let device_types = vec![
            DeviceType::HardDisk,
            DeviceType::SolidState,
            DeviceType::Removable,
            DeviceType::Network,
            DeviceType::RamDisk,
            DeviceType::OpticalDisk,
            DeviceType::Unknown,
        ];
        
        for device_type in device_types {
            let json = serde_json::to_string(&device_type).unwrap();
            let deserialized: DeviceType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, device_type);
        }
    }

    #[test]
    fn test_mock_platform_concurrent_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let mock = Arc::new(MockPlatform::new());
        let mut handles = vec![];
        
        // Test concurrent device listing
        for _ in 0..5 {
            let mock_clone = mock.clone();
            let handle = thread::spawn(move || {
                let devices = mock_clone.list_storage_devices_instance().unwrap();
                assert!(!devices.is_empty());
            });
            handles.push(handle);
        }
        
        // Test concurrent app data dir access
        for _ in 0..5 {
            let mock_clone = mock.clone();
            let handle = thread::spawn(move || {
                let app_dir = mock_clone.get_app_data_dir_instance().unwrap();
                assert!(!app_dir.as_os_str().is_empty());
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_platform_ops_trait_bounds() {
        // Test that PlatformOps has the right trait bounds
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        // These should compile if the traits are properly implemented
        assert_send::<StorageDevice>();
        assert_sync::<StorageDevice>();
        assert_send::<DeviceType>();
        assert_sync::<DeviceType>();
        assert_send::<PlatformError>();
        assert_sync::<PlatformError>();
    }

    #[test]
    fn test_storage_device_with_unicode_paths() {
        let device = StorageDevice {
            name: "Unicode Drive 测试".to_string(),
            mount_point: PathBuf::from("/mnt/测试驱动器"),
            total_space: 1024,
            available_space: 512,
            device_type: DeviceType::Removable,
        };
        
        assert!(device.name.contains("测试"));
        assert!(device.mount_point.to_string_lossy().contains("测试驱动器"));
        
        // Test serialization with unicode
        let json = serde_json::to_string(&device).unwrap();
        let deserialized: StorageDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, device.name);
        assert_eq!(deserialized.mount_point, device.mount_point);
    }

    #[test]
    fn test_storage_device_with_special_characters() {
        let device = StorageDevice {
            name: "Drive with spaces & symbols!@#$%".to_string(),
            mount_point: PathBuf::from("/mnt/drive with spaces"),
            total_space: 1024,
            available_space: 512,
            device_type: DeviceType::Network,
        };
        
        assert!(device.name.contains("spaces & symbols"));
        
        // Test that serialization handles special characters
        let json = serde_json::to_string(&device).unwrap();
        let deserialized: StorageDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, device.name);
    }

    #[test]
    fn test_platform_error_with_empty_messages() {
        let errors = vec![
            PlatformError::UnsupportedPlatform(String::new()),
            PlatformError::DeviceEnumerationFailed(String::new()),
            PlatformError::InsufficientPermissions(String::new()),
        ];
        
        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty()); // Should still have the error type description
        }
    }

    #[test]
    fn test_mock_platform_state_isolation() {
        // Test that different mock instances don't interfere with each other
        let mock1 = MockPlatform::new();
        let mock2 = MockPlatform::new();
        
        // Modify mock1
        mock1.clear_devices();
        mock1.simulate_error(PlatformError::DirectIoNotSupported);
        
        // mock2 should be unaffected
        let devices2 = mock2.list_storage_devices_instance().unwrap();
        assert_eq!(devices2.len(), 3); // Should have default devices
        assert!(!mock2.is_simulating_errors());
    }
}