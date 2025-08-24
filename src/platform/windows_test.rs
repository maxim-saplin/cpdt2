//! Comprehensive Windows platform-specific tests

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::super::windows::WindowsPlatform;
    use super::super::{DeviceType, PlatformError};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_device_enumeration() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();
        
        // Windows should have at least C: drive
        assert!(!devices.is_empty(), "Windows should have at least one storage device");
        
        // Check for C: drive specifically
        let has_c_drive = devices.iter().any(|d| {
            d.mount_point.to_string_lossy().to_uppercase().starts_with("C:")
        });
        assert!(has_c_drive, "Windows should have C: drive");
        
        // Validate device properties
        for device in &devices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
            assert!(device.mount_point.is_absolute(), "Mount point should be absolute");
            assert!(device.total_space > 0, "Total space should be greater than 0");
            assert!(device.available_space <= device.total_space, "Available space should not exceed total space");
            
            // Check device type is reasonable
            match device.device_type {
                DeviceType::HardDisk | DeviceType::SolidState | DeviceType::Removable | 
                DeviceType::Network | DeviceType::RamDisk | DeviceType::OpticalDisk | 
                DeviceType::Unknown => {
                    // All valid device types
                }
            }
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_app_data_directory() {
        let app_data_dir = WindowsPlatform::get_app_data_dir().unwrap();
        
        // Should end with our application name
        assert!(app_data_dir.ends_with("disk-speed-test"), 
                "App data dir should end with disk-speed-test");
        
        // Should be absolute path
        assert!(app_data_dir.is_absolute(), "App data dir should be absolute");
        
        // Should contain LOCALAPPDATA
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap();
        assert!(app_data_dir.starts_with(&local_app_data), 
                "App data dir should start with LOCALAPPDATA");
        
        // Path should be valid Windows path
        let path_str = app_data_dir.to_string_lossy();
        assert!(path_str.contains("\\"), "Windows path should contain backslashes");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_direct_io_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("windows_direct_io_test.bin");
        let file_size = 2 * 1024 * 1024; // 2MB
        
        // Create direct I/O file
        let file = WindowsPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        drop(file); // Close file handle
        
        // Verify file exists and has correct size
        assert!(test_file.exists(), "Direct I/O file should exist");
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size, "File should have correct size");
        
        // Verify file can be read
        let content = fs::read(&test_file).unwrap();
        assert_eq!(content.len(), file_size as usize, "File content should match expected size");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_direct_io_file_opening() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("windows_open_test.bin");
        let file_size = 1024 * 1024; // 1MB
        
        // First create the file
        let _create_file = WindowsPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        
        // Test opening for read
        let read_file = WindowsPlatform::open_direct_io_file(&test_file, false).unwrap();
        drop(read_file);
        
        // Test opening for write
        let write_file = WindowsPlatform::open_direct_io_file(&test_file, true).unwrap();
        drop(write_file);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_file_system_sync() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("windows_sync_test.txt");
        
        // Create test file
        fs::write(&test_file, b"Windows sync test data").unwrap();
        
        // Test file system sync
        let result = WindowsPlatform::sync_file_system(&test_file);
        assert!(result.is_ok(), "File system sync should succeed");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_large_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("windows_large_file.bin");
        let large_size = 100 * 1024 * 1024; // 100MB
        
        // Create large file
        let file = WindowsPlatform::create_direct_io_file(&test_file, large_size).unwrap();
        drop(file);
        
        // Verify size
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), large_size, "Large file should have correct size");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_drive_type_detection() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();
        
        // Should have at least one device with a known type
        let has_typed_device = devices.iter().any(|d| {
            matches!(d.device_type, DeviceType::HardDisk | DeviceType::SolidState)
        });
        assert!(has_typed_device, "Should detect at least one HDD or SSD");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_device_space_consistency() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();
        
        for device in &devices {
            // Available space should never exceed total space
            assert!(device.available_space <= device.total_space,
                    "Available space ({}) should not exceed total space ({}) for device {}",
                    device.available_space, device.total_space, device.name);
            
            // Both values should be reasonable (not zero for real devices)
            if device.device_type != DeviceType::OpticalDisk {
                assert!(device.total_space > 0, "Total space should be > 0 for device {}", device.name);
            }
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_error_handling_invalid_path() {
        // Test with invalid path characters
        let invalid_path = Path::new("C:\\invalid<>path|?.bin");
        let result = WindowsPlatform::create_direct_io_file(invalid_path, 1024);
        assert!(result.is_err(), "Should fail with invalid path characters");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_error_handling_nonexistent_drive() {
        // Test with non-existent drive letter
        let nonexistent_path = Path::new("Z:\\nonexistent\\file.bin");
        let result = WindowsPlatform::create_direct_io_file(nonexistent_path, 1024);
        // This might succeed if Z: drive exists, so we just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_concurrent_file_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let temp_dir = Arc::new(TempDir::new().unwrap());
        let mut handles = vec![];
        
        // Create multiple files concurrently
        for i in 0..5 {
            let temp_dir_clone = temp_dir.clone();
            let handle = thread::spawn(move || {
                let test_file = temp_dir_clone.path().join(format!("concurrent_test_{}.bin", i));
                let file_size = 1024 * 1024; // 1MB
                
                let result = WindowsPlatform::create_direct_io_file(&test_file, file_size);
                assert!(result.is_ok(), "Concurrent file creation should succeed");
                
                // Verify file
                let metadata = fs::metadata(&test_file).unwrap();
                assert_eq!(metadata.len(), file_size);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_device_name_formatting() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();
        
        for device in &devices {
            // Device names should follow Windows convention
            assert!(device.name.starts_with("Drive "), 
                    "Windows device name should start with 'Drive '");
            
            // Should contain a drive letter
            let drive_letter = device.name.chars().last().unwrap();
            assert!(drive_letter.is_ascii_uppercase(), 
                    "Drive letter should be uppercase ASCII");
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_mount_point_format() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();
        
        for device in &devices {
            let mount_str = device.mount_point.to_string_lossy();
            
            // Should be Windows drive format (e.g., "C:\")
            assert!(mount_str.len() >= 3, "Mount point should be at least 3 characters");
            assert!(mount_str.ends_with(":\\"), "Mount point should end with :\\");
            
            let drive_letter = mount_str.chars().next().unwrap();
            assert!(drive_letter.is_ascii_uppercase(), "Drive letter should be uppercase");
        }
    }

    // Mock tests that can run on any platform
    #[test]
    fn test_windows_mock_device_enumeration() {
        // This test uses mock data to test Windows-specific logic without requiring Windows
        use super::super::mock_platform::MockPlatform;
        
        let mock = MockPlatform::new();
        mock.clear_devices();
        
        // Add Windows-style devices
        mock.add_device(super::super::StorageDevice {
            name: "Drive C".to_string(),
            mount_point: PathBuf::from("C:\\"),
            total_space: 500 * 1024 * 1024 * 1024, // 500GB
            available_space: 200 * 1024 * 1024 * 1024, // 200GB
            device_type: DeviceType::SolidState,
        });
        
        mock.add_device(super::super::StorageDevice {
            name: "Drive D".to_string(),
            mount_point: PathBuf::from("D:\\"),
            total_space: 1024 * 1024 * 1024 * 1024, // 1TB
            available_space: 800 * 1024 * 1024 * 1024, // 800GB
            device_type: DeviceType::HardDisk,
        });
        
        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 2);
        
        // Verify Windows-style properties
        let c_drive = &devices[0];
        assert_eq!(c_drive.name, "Drive C");
        assert_eq!(c_drive.mount_point, PathBuf::from("C:\\"));
        assert_eq!(c_drive.device_type, DeviceType::SolidState);
        
        let d_drive = &devices[1];
        assert_eq!(d_drive.name, "Drive D");
        assert_eq!(d_drive.mount_point, PathBuf::from("D:\\"));
        assert_eq!(d_drive.device_type, DeviceType::HardDisk);
    }

    #[test]
    fn test_windows_mock_error_conditions() {
        use super::super::mock_platform::MockPlatform;
        
        let mock = MockPlatform::new();
        
        // Test device enumeration failure
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "GetLogicalDrives failed".to_string()
        ));
        
        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::DeviceEnumerationFailed(msg) => {
                assert_eq!(msg, "GetLogicalDrives failed");
            }
            _ => panic!("Expected DeviceEnumerationFailed"),
        }
    }

    #[test]
    fn test_windows_mock_direct_io_operations() {
        use super::super::mock_platform::{MockPlatform, MockFileResult};
        
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("C:\\test\\direct_io.bin");
        
        // Test successful direct I/O
        mock.set_file_operation_result(test_path.clone(), MockFileResult::Success);
        
        // Create a temporary file for the mock to work with
        let temp_dir = TempDir::new().unwrap();
        let actual_test_path = temp_dir.path().join("direct_io.bin");
        std::fs::write(&actual_test_path, "test").unwrap();
        
        let result = mock.create_direct_io_file_instance(&actual_test_path, 1024);
        assert!(result.is_ok());
        
        // Test direct I/O error
        mock.set_file_operation_result(
            test_path.clone(),
            MockFileResult::Error(PlatformError::DirectIoNotSupported)
        );
        
        let result = mock.create_direct_io_file_instance(&test_path, 1024);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::DirectIoNotSupported => {
                // Expected
            }
            _ => panic!("Expected DirectIoNotSupported"),
        }
    }
}