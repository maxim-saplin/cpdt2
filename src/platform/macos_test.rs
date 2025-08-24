//! Comprehensive macOS platform-specific tests

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use super::super::macos::MacOsPlatform;
    use super::super::{PlatformOps, StorageDevice, DeviceType, PlatformError};
    use std::path::{Path, PathBuf};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_device_enumeration() {
        let devices = MacOsPlatform::list_storage_devices().unwrap();
        
        // macOS should have at least the root filesystem
        assert!(!devices.is_empty(), "macOS should have at least one storage device");
        
        // Check for root filesystem
        let has_root = devices.iter().any(|d| d.mount_point == Path::new("/"));
        assert!(has_root, "macOS should have root filesystem (/)");
        
        // Validate device properties
        for device in &devices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
            assert!(device.mount_point.is_absolute(), "Mount point should be absolute");
            
            // For root filesystem, space should be > 0
            if device.mount_point == Path::new("/") {
                assert!(device.total_space > 0, "Root filesystem should have total space > 0");
                assert!(device.available_space <= device.total_space, 
                        "Available space should not exceed total space");
            }
            
            // Check device type is reasonable
            match device.device_type {
                DeviceType::HardDisk | DeviceType::SolidState | DeviceType::Removable | 
                DeviceType::Network | DeviceType::RamDisk | DeviceType::Unknown => {
                    // All valid device types for macOS
                }
                DeviceType::OpticalDisk => {
                    // Less common but valid
                }
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_app_data_directory() {
        let app_data_dir = MacOsPlatform::get_app_data_dir().unwrap();
        
        // Should end with our application name
        assert!(app_data_dir.ends_with("disk-speed-test"), 
                "App data dir should end with disk-speed-test");
        
        // Should be absolute path
        assert!(app_data_dir.is_absolute(), "App data dir should be absolute");
        
        // Should contain Library/Application Support
        let path_str = app_data_dir.to_string_lossy();
        assert!(path_str.contains("Library/Application Support"), 
                "macOS app data dir should contain Library/Application Support");
        
        // Should exist after calling get_app_data_dir
        assert!(app_data_dir.exists(), "App data directory should be created");
        assert!(app_data_dir.is_dir(), "App data path should be a directory");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_direct_io_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("macos_direct_io_test.bin");
        let file_size = 2 * 1024 * 1024; // 2MB
        
        // Create direct I/O file
        let file = MacOsPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        drop(file); // Close file handle
        
        // Verify file exists and has correct size
        assert!(test_file.exists(), "Direct I/O file should exist");
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size, "File should have correct size");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_direct_io_file_opening() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("macos_open_test.bin");
        let file_size = 1024 * 1024; // 1MB
        
        // First create the file
        let _create_file = MacOsPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        
        // Test opening for read
        let read_file = MacOsPlatform::open_direct_io_file(&test_file, false).unwrap();
        drop(read_file);
        
        // Test opening for write
        let write_file = MacOsPlatform::open_direct_io_file(&test_file, true).unwrap();
        drop(write_file);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_file_system_sync() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("macos_sync_test.txt");
        
        // Create test file
        fs::write(&test_file, b"macOS sync test data").unwrap();
        
        // Test file system sync
        let result = MacOsPlatform::sync_file_system(&test_file);
        assert!(result.is_ok(), "File system sync should succeed");
        
        // Test sync on directory
        let dir_result = MacOsPlatform::sync_file_system(temp_dir.path());
        assert!(dir_result.is_ok(), "Directory sync should succeed");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_volumes_enumeration() {
        let devices = MacOsPlatform::list_storage_devices().unwrap();
        
        // Check if we have any /Volumes entries
        let volumes_devices: Vec<_> = devices.iter()
            .filter(|d| d.mount_point.starts_with("/Volumes"))
            .collect();
        
        // Validate /Volumes entries if they exist
        for device in volumes_devices {
            assert!(device.mount_point.starts_with("/Volumes"), 
                    "Volume device should be mounted under /Volumes");
            assert!(!device.name.is_empty(), "Volume should have a name");
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_root_filesystem_properties() {
        let devices = MacOsPlatform::list_storage_devices().unwrap();
        
        let root_device = devices.iter()
            .find(|d| d.mount_point == Path::new("/"))
            .expect("Should have root filesystem");
        
        // Root filesystem should be named "Macintosh HD" by default
        assert_eq!(root_device.name, "Macintosh HD", "Root filesystem should be named Macintosh HD");
        
        // Should be classified as SSD on modern Macs
        assert_eq!(root_device.device_type, DeviceType::SolidState, 
                  "Root filesystem should be classified as SSD");
        
        // Should have reasonable space values
        assert!(root_device.total_space > 1024 * 1024 * 1024, // > 1GB
                "Root filesystem should have > 1GB total space");
        assert!(root_device.available_space <= root_device.total_space,
                "Available space should not exceed total space");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_fcntl_flags() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("fcntl_test.bin");
        let file_size = 1024 * 1024; // 1MB
        
        // Create file with F_NOCACHE flag
        let file = MacOsPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        
        // File should be created successfully with direct I/O flags
        assert!(test_file.exists(), "File with F_NOCACHE should be created");
        
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size, "File should have correct size");
        
        drop(file);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_large_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("macos_large_file.bin");
        let large_size = 100 * 1024 * 1024; // 100MB
        
        // Create large file
        let file = MacOsPlatform::create_direct_io_file(&test_file, large_size).unwrap();
        drop(file);
        
        // Verify size
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), large_size, "Large file should have correct size");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_device_type_detection() {
        let devices = MacOsPlatform::list_storage_devices().unwrap();
        
        // Should have at least one device with SolidState type (root filesystem)
        let has_ssd = devices.iter().any(|d| d.device_type == DeviceType::SolidState);
        assert!(has_ssd, "Should detect at least one SSD device");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_error_handling_invalid_path() {
        // Test with path that doesn't exist
        let invalid_path = Path::new("/nonexistent/directory/file.bin");
        let result = MacOsPlatform::open_direct_io_file(invalid_path, false);
        assert!(result.is_err(), "Should fail with nonexistent path");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_concurrent_file_operations() {
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
                
                let result = MacOsPlatform::create_direct_io_file(&test_file, file_size);
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
    #[cfg(target_os = "macos")]
    fn test_macos_unicode_paths() {
        let temp_dir = TempDir::new().unwrap();
        let unicode_file = temp_dir.path().join("测试文件.bin");
        let file_size = 1024; // 1KB
        
        // Create file with unicode name
        let result = MacOsPlatform::create_direct_io_file(&unicode_file, file_size);
        assert!(result.is_ok(), "Should handle unicode file names");
        
        // Verify file exists
        assert!(unicode_file.exists(), "Unicode file should exist");
        let metadata = fs::metadata(&unicode_file).unwrap();
        assert_eq!(metadata.len(), file_size);
    }

    // Mock tests that can run on any platform
    #[test]
    fn test_macos_mock_device_enumeration() {
        use super::super::mock_platform::MockPlatform;
        
        let mock = MockPlatform::new();
        mock.clear_devices();
        
        // Add macOS-style devices
        mock.add_device(super::super::StorageDevice {
            name: "Macintosh HD".to_string(),
            mount_point: PathBuf::from("/"),
            total_space: 500 * 1024 * 1024 * 1024, // 500GB
            available_space: 200 * 1024 * 1024 * 1024, // 200GB
            device_type: DeviceType::SolidState,
        });
        
        mock.add_device(super::super::StorageDevice {
            name: "External Drive".to_string(),
            mount_point: PathBuf::from("/Volumes/External Drive"),
            total_space: 1024 * 1024 * 1024 * 1024, // 1TB
            available_space: 800 * 1024 * 1024 * 1024, // 800GB
            device_type: DeviceType::Removable,
        });
        
        mock.add_device(super::super::StorageDevice {
            name: "Network Share".to_string(),
            mount_point: PathBuf::from("/Volumes/Network Share"),
            total_space: 2 * 1024 * 1024 * 1024 * 1024, // 2TB
            available_space: 1024 * 1024 * 1024 * 1024, // 1TB
            device_type: DeviceType::Network,
        });
        
        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 3);
        
        // Verify macOS-style properties
        let root_device = &devices[0];
        assert_eq!(root_device.name, "Macintosh HD");
        assert_eq!(root_device.mount_point, PathBuf::from("/"));
        assert_eq!(root_device.device_type, DeviceType::SolidState);
        
        let external_device = &devices[1];
        assert_eq!(external_device.name, "External Drive");
        assert!(external_device.mount_point.starts_with("/Volumes"));
        assert_eq!(external_device.device_type, DeviceType::Removable);
        
        let network_device = &devices[2];
        assert_eq!(network_device.name, "Network Share");
        assert!(network_device.mount_point.starts_with("/Volumes"));
        assert_eq!(network_device.device_type, DeviceType::Network);
    }

    #[test]
    fn test_macos_mock_error_conditions() {
        use super::super::mock_platform::MockPlatform;
        
        let mock = MockPlatform::new();
        
        // Test device enumeration failure
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "Failed to read /Volumes".to_string()
        ));
        
        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::DeviceEnumerationFailed(msg) => {
                assert_eq!(msg, "Failed to read /Volumes");
            }
            _ => panic!("Expected DeviceEnumerationFailed"),
        }
        
        // Test insufficient permissions
        mock.simulate_error(PlatformError::InsufficientPermissions(
            "Cannot access volume information".to_string()
        ));
        
        let result = mock.get_app_data_dir_instance();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InsufficientPermissions(msg) => {
                assert_eq!(msg, "Cannot access volume information");
            }
            _ => panic!("Expected InsufficientPermissions"),
        }
    }

    #[test]
    fn test_macos_mock_fcntl_operations() {
        use super::super::mock_platform::{MockPlatform, MockFileResult};
        
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/tmp/fcntl_test.bin");
        
        // Test successful F_NOCACHE operation
        mock.set_file_operation_result(test_path.clone(), MockFileResult::Success);
        
        // Create a temporary file for the mock to work with
        let temp_dir = TempDir::new().unwrap();
        let actual_test_path = temp_dir.path().join("fcntl_test.bin");
        std::fs::write(&actual_test_path, "test").unwrap();
        
        let result = mock.create_direct_io_file_instance(&actual_test_path, 1024);
        assert!(result.is_ok());
        
        // Test F_NOCACHE failure
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

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_device_type_determination() {
        // Test the device type determination logic with mock paths
        use super::super::macos::determine_device_type;
        
        let statvfs: libc::statvfs = unsafe { std::mem::zeroed() };
        
        // Test root filesystem
        assert_eq!(determine_device_type(Path::new("/"), &statvfs), DeviceType::SolidState);
        
        // Test network volumes
        assert_eq!(determine_device_type(Path::new("/Volumes/Network Share"), &statvfs), DeviceType::Network);
        assert_eq!(determine_device_type(Path::new("/Volumes/net-drive"), &statvfs), DeviceType::Network);
        
        // Test removable volumes
        assert_eq!(determine_device_type(Path::new("/Volumes/USB Drive"), &statvfs), DeviceType::Removable);
        assert_eq!(determine_device_type(Path::new("/Volumes/External Disk"), &statvfs), DeviceType::Removable);
        
        // Test RAM disk
        assert_eq!(determine_device_type(Path::new("/Volumes/RAM Disk"), &statvfs), DeviceType::RamDisk);
        
        // Test unknown
        assert_eq!(determine_device_type(Path::new("/Volumes/Some Volume"), &statvfs), DeviceType::Unknown);
    }
}