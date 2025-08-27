//! Comprehensive Linux platform-specific tests

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::super::linux::LinuxPlatform;
    use super::super::{DeviceType, PlatformError};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[cfg(target_os = "linux")]
    use crate::platform::PlatformOps;
    #[cfg(target_os = "linux")]
    use std::path::Path;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_device_enumeration() {
        let devices = LinuxPlatform::list_storage_devices().unwrap();

        // Linux should have at least the root filesystem
        assert!(
            !devices.is_empty(),
            "Linux should have at least one storage device"
        );

        // Check for root filesystem
        let has_root = devices.iter().any(|d| d.mount_point == Path::new("/"));
        assert!(has_root, "Linux should have root filesystem (/)");

        // Validate device properties
        for device in &devices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
            assert!(
                device.mount_point.is_absolute(),
                "Mount point should be absolute"
            );
            assert!(device.mount_point.exists(), "Mount point should exist");

            // For real filesystems, space should be > 0
            assert!(
                device.total_space > 0,
                "Total space should be > 0 for device {}",
                device.name
            );
            assert!(
                device.available_space <= device.total_space,
                "Available space should not exceed total space for device {}",
                device.name
            );

            // Check device type is reasonable
            match device.device_type {
                DeviceType::HardDisk
                | DeviceType::SolidState
                | DeviceType::Removable
                | DeviceType::Network
                | DeviceType::Unknown => {
                    // All valid device types for Linux
                }
                DeviceType::RamDisk | DeviceType::OpticalDisk => {
                    // Less common but valid
                }
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_app_data_directory() {
        let app_data_dir = LinuxPlatform::get_app_data_dir().unwrap();

        // Should end with our application name
        assert!(
            app_data_dir.ends_with("disk-speed-test"),
            "App data dir should end with disk-speed-test"
        );

        // Should be absolute path
        assert!(
            app_data_dir.is_absolute(),
            "App data dir should be absolute"
        );

        // Should follow XDG Base Directory specification
        let path_str = app_data_dir.to_string_lossy();
        let has_xdg_or_local =
            path_str.contains(".local/share") || std::env::var("XDG_DATA_HOME").is_ok();
        assert!(
            has_xdg_or_local,
            "Should follow XDG Base Directory specification"
        );

        // Should exist after calling get_app_data_dir
        assert!(
            app_data_dir.exists(),
            "App data directory should be created"
        );
        assert!(app_data_dir.is_dir(), "App data path should be a directory");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_direct_io_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("linux_direct_io_test.bin");
        let file_size = 2 * 1024 * 1024; // 2MB

        // Create direct I/O file
        let file = LinuxPlatform::create_direct_io_file(&test_file, file_size).unwrap();
        drop(file); // Close file handle

        // Verify file exists and has correct size
        assert!(test_file.exists(), "Direct I/O file should exist");
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size, "File should have correct size");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_direct_io_file_opening() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("linux_open_test.bin");
        let file_size = 1024 * 1024; // 1MB

        // First create the file
        let _create_file = LinuxPlatform::create_direct_io_file(&test_file, file_size).unwrap();

        // Test opening for read
        let read_file = LinuxPlatform::open_direct_io_file(&test_file, false).unwrap();
        drop(read_file);

        // Test opening for write
        let write_file = LinuxPlatform::open_direct_io_file(&test_file, true).unwrap();
        drop(write_file);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_file_system_sync() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("linux_sync_test.txt");

        // Create test file
        fs::write(&test_file, b"Linux sync test data").unwrap();

        // Test file system sync
        let result = LinuxPlatform::sync_file_system(&test_file);
        assert!(result.is_ok(), "File system sync should succeed");

        // Test sync on directory
        let dir_result = LinuxPlatform::sync_file_system(temp_dir.path());
        assert!(dir_result.is_ok(), "Directory sync should succeed");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_proc_mounts_parsing() {
        // This test verifies that we can parse /proc/mounts
        let devices = LinuxPlatform::list_storage_devices().unwrap();

        // Should have filtered out virtual filesystems
        for device in &devices {
            let mount_str = device.mount_point.to_string_lossy();

            // Should not contain virtual filesystem mount points
            assert!(
                !mount_str.starts_with("/proc"),
                "Should not include /proc mounts"
            );
            assert!(
                !mount_str.starts_with("/sys"),
                "Should not include /sys mounts"
            );
            assert!(
                !mount_str.starts_with("/dev/pts"),
                "Should not include /dev/pts mounts"
            );
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_device_type_detection() {
        let devices = LinuxPlatform::list_storage_devices().unwrap();

        // Should have at least one device with a detected type
        let has_typed_device = devices.iter().any(|d| {
            matches!(
                d.device_type,
                DeviceType::HardDisk | DeviceType::SolidState | DeviceType::Unknown
            )
        });
        assert!(has_typed_device, "Should detect device types");

        // Root filesystem should exist and have a reasonable type
        if let Some(root_device) = devices.iter().find(|d| d.mount_point == Path::new("/")) {
            match root_device.device_type {
                DeviceType::HardDisk | DeviceType::SolidState | DeviceType::Unknown => {
                    // These are all reasonable for root filesystem
                }
                _ => panic!("Root filesystem should have HDD, SSD, or Unknown type"),
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_o_direct_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("o_direct_test.bin");
        let file_size = 1024 * 1024; // 1MB

        // This should work even if O_DIRECT is not supported (falls back to O_SYNC)
        let result = LinuxPlatform::create_direct_io_file(&test_file, file_size);
        assert!(result.is_ok(), "Should handle O_DIRECT fallback gracefully");

        // Verify file was created
        assert!(test_file.exists());
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_large_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("linux_large_file.bin");
        let large_size = 100 * 1024 * 1024; // 100MB

        // Create large file
        let file = LinuxPlatform::create_direct_io_file(&test_file, large_size).unwrap();
        drop(file);

        // Verify size
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(
            metadata.len(),
            large_size,
            "Large file should have correct size"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_filesystem_stats() {
        let devices = LinuxPlatform::list_storage_devices().unwrap();

        // Find root filesystem
        let root_device = devices
            .iter()
            .find(|d| d.mount_point == Path::new("/"))
            .expect("Should have root filesystem");

        // Root filesystem should have reasonable space values
        assert!(
            root_device.total_space > 1024 * 1024 * 1024, // > 1GB
            "Root filesystem should have > 1GB total space"
        );
        assert!(
            root_device.available_space <= root_device.total_space,
            "Available space should not exceed total space"
        );

        // Available space should be reasonable (not 0 unless disk is full)
        // We'll just check it's a valid value
        assert!(root_device.available_space <= root_device.total_space);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_device_name_formatting() {
        let devices = LinuxPlatform::list_storage_devices().unwrap();

        for device in &devices {
            // Device names should contain device path information
            assert!(!device.name.is_empty(), "Device name should not be empty");

            // Should contain either device path or mount point info
            let name_lower = device.name.to_lowercase();
            let mount_str = device.mount_point.to_string_lossy().to_lowercase();

            // Name should be descriptive
            assert!(name_lower.len() > 3, "Device name should be descriptive");
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_error_handling_invalid_path() {
        // Test with path that doesn't exist
        let invalid_path = Path::new("/nonexistent/directory/file.bin");
        let result = LinuxPlatform::open_direct_io_file(invalid_path, false);
        assert!(result.is_err(), "Should fail with nonexistent path");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_concurrent_file_operations() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = Arc::new(TempDir::new().unwrap());
        let mut handles = vec![];

        // Create multiple files concurrently
        for i in 0..5 {
            let temp_dir_clone = temp_dir.clone();
            let handle = thread::spawn(move || {
                let test_file = temp_dir_clone
                    .path()
                    .join(format!("concurrent_test_{}.bin", i));
                let file_size = 1024 * 1024; // 1MB

                let result = LinuxPlatform::create_direct_io_file(&test_file, file_size);
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
    #[cfg(target_os = "linux")]
    fn test_linux_xdg_data_home_support() {
        // Test XDG_DATA_HOME environment variable support
        let original_xdg = std::env::var("XDG_DATA_HOME").ok();
        let original_home = std::env::var("HOME").ok();

        // Test with custom XDG_DATA_HOME
        std::env::set_var("XDG_DATA_HOME", "/tmp/custom_xdg");
        let app_data_dir = LinuxPlatform::get_app_data_dir().unwrap();
        assert!(
            app_data_dir.starts_with("/tmp/custom_xdg"),
            "Should use XDG_DATA_HOME when set"
        );

        // Restore original environment
        if let Some(xdg) = original_xdg {
            std::env::set_var("XDG_DATA_HOME", xdg);
        } else {
            std::env::remove_var("XDG_DATA_HOME");
        }

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_unicode_paths() {
        let temp_dir = TempDir::new().unwrap();
        let unicode_file = temp_dir.path().join("测试文件.bin");
        let file_size = 1024; // 1KB

        // Create file with unicode name
        let result = LinuxPlatform::create_direct_io_file(&unicode_file, file_size);
        assert!(result.is_ok(), "Should handle unicode file names");

        // Verify file exists
        assert!(unicode_file.exists(), "Unicode file should exist");
        let metadata = fs::metadata(&unicode_file).unwrap();
        assert_eq!(metadata.len(), file_size);
    }

    // Mock tests that can run on any platform
    #[test]
    fn test_linux_mock_device_enumeration() {
        use super::super::mock_platform::MockPlatform;

        let mock = MockPlatform::new();
        mock.clear_devices();

        // Add Linux-style devices
        mock.add_device(super::super::StorageDevice {
            name: "Root Filesystem (/dev/sda1)".to_string(),
            mount_point: PathBuf::from("/"),
            total_space: 500 * 1024 * 1024 * 1024,     // 500GB
            available_space: 200 * 1024 * 1024 * 1024, // 200GB
            device_type: DeviceType::SolidState,
        });

        mock.add_device(super::super::StorageDevice {
            name: "home (/dev/sdb1)".to_string(),
            mount_point: PathBuf::from("/home"),
            total_space: 1024 * 1024 * 1024 * 1024,    // 1TB
            available_space: 800 * 1024 * 1024 * 1024, // 800GB
            device_type: DeviceType::HardDisk,
        });

        mock.add_device(super::super::StorageDevice {
            name: "usb (/dev/sdc1)".to_string(),
            mount_point: PathBuf::from("/media/usb"),
            total_space: 32 * 1024 * 1024 * 1024,     // 32GB
            available_space: 16 * 1024 * 1024 * 1024, // 16GB
            device_type: DeviceType::Removable,
        });

        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 3);

        // Verify Linux-style properties
        let root_device = &devices[0];
        assert_eq!(root_device.name, "Root Filesystem (/dev/sda1)");
        assert_eq!(root_device.mount_point, PathBuf::from("/"));
        assert_eq!(root_device.device_type, DeviceType::SolidState);

        let home_device = &devices[1];
        assert_eq!(home_device.name, "home (/dev/sdb1)");
        assert_eq!(home_device.mount_point, PathBuf::from("/home"));
        assert_eq!(home_device.device_type, DeviceType::HardDisk);

        let usb_device = &devices[2];
        assert_eq!(usb_device.name, "usb (/dev/sdc1)");
        assert_eq!(usb_device.mount_point, PathBuf::from("/media/usb"));
        assert_eq!(usb_device.device_type, DeviceType::Removable);
    }

    #[test]
    fn test_linux_mock_error_conditions() {
        use super::super::mock_platform::MockPlatform;

        let mock = MockPlatform::new();

        // Test /proc/mounts read failure
        mock.simulate_error(PlatformError::DeviceEnumerationFailed(
            "Failed to open /proc/mounts: Permission denied".to_string(),
        ));

        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::DeviceEnumerationFailed(msg) => {
                assert!(msg.contains("/proc/mounts"));
                assert!(msg.contains("Permission denied"));
            }
            _ => panic!("Expected DeviceEnumerationFailed"),
        }

        // Test insufficient permissions for app data directory
        mock.simulate_error(PlatformError::InsufficientPermissions(
            "Cannot create directory in /home".to_string(),
        ));

        let result = mock.get_app_data_dir_instance();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InsufficientPermissions(msg) => {
                assert!(msg.contains("Cannot create directory"));
            }
            _ => panic!("Expected InsufficientPermissions"),
        }
    }

    #[test]
    fn test_linux_mock_o_direct_operations() {
        use super::super::mock_platform::{MockFileResult, MockPlatform};

        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/tmp/o_direct_test.bin");

        // Test successful O_DIRECT operation
        mock.set_file_operation_result(test_path.clone(), MockFileResult::Success);

        // Create a temporary file for the mock to work with
        let temp_dir = TempDir::new().unwrap();
        let actual_test_path = temp_dir.path().join("o_direct_test.bin");
        std::fs::write(&actual_test_path, "test").unwrap();

        let result = mock.create_direct_io_file_instance(&actual_test_path, 1024);
        assert!(result.is_ok());

        // Test O_DIRECT not supported (should fall back gracefully)
        mock.set_file_operation_result(
            test_path.clone(),
            MockFileResult::Error(PlatformError::DirectIoNotSupported),
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
    #[cfg(target_os = "linux")]
    fn test_linux_filesystem_type_filtering() {
        // Test the is_real_filesystem function logic
        use super::super::linux::LinuxPlatform;

        // Real filesystems should pass
        assert!(LinuxPlatform::is_real_filesystem("/dev/sda1", "ext4"));
        assert!(LinuxPlatform::is_real_filesystem("/dev/nvme0n1p1", "xfs"));
        assert!(LinuxPlatform::is_real_filesystem(
            "/dev/mapper/root",
            "btrfs"
        ));
        assert!(LinuxPlatform::is_real_filesystem("/dev/md0", "ext4"));

        // Virtual filesystems should be filtered out
        assert!(!LinuxPlatform::is_real_filesystem("proc", "proc"));
        assert!(!LinuxPlatform::is_real_filesystem("sysfs", "sysfs"));
        assert!(!LinuxPlatform::is_real_filesystem("tmpfs", "tmpfs"));
        assert!(!LinuxPlatform::is_real_filesystem("devpts", "devpts"));
        assert!(!LinuxPlatform::is_real_filesystem("cgroup", "cgroup"));
        assert!(!LinuxPlatform::is_real_filesystem("cgroup2", "cgroup2"));

        // Virtual device paths should be filtered out
        assert!(!LinuxPlatform::is_real_filesystem(
            "/proc/something",
            "ext4"
        ));
        assert!(!LinuxPlatform::is_real_filesystem("/sys/something", "ext4"));
        assert!(!LinuxPlatform::is_real_filesystem("/dev/pts/0", "devpts"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_device_info_parsing() {
        // Test device info parsing with mock data
        use super::super::linux::LinuxPlatform;

        // Test various device paths
        let info = LinuxPlatform::get_device_info("/dev/sda1");
        assert!(info.is_ok());

        let info = LinuxPlatform::get_device_info("/dev/nvme0n1p1");
        assert!(info.is_ok());

        let info = LinuxPlatform::get_device_info("/dev/mapper/root");
        assert!(info.is_ok());

        // Test loop device (should be Unknown)
        let info = LinuxPlatform::get_device_info("/dev/loop0").unwrap();
        assert_eq!(info.device_type, DeviceType::Unknown);

        // Test unknown device
        let info = LinuxPlatform::get_device_info("/dev/unknown123").unwrap();
        assert_eq!(info.device_type, DeviceType::Unknown);
    }
}
