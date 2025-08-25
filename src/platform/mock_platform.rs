//! Mock platform implementation for testing platform abstraction layer

use crate::platform::{DeviceType, PlatformError, PlatformOps, StorageDevice};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Mock platform implementation for testing
pub struct MockPlatform {
    /// Simulated storage devices
    devices: Arc<Mutex<Vec<StorageDevice>>>,
    /// Simulated app data directory
    app_data_dir: Arc<Mutex<Option<PathBuf>>>,
    /// Simulated file operations results
    file_operations: Arc<Mutex<HashMap<PathBuf, MockFileResult>>>,
    /// Whether to simulate errors
    simulate_errors: Arc<Mutex<bool>>,
    /// Error to simulate
    simulated_error: Arc<Mutex<Option<PlatformError>>>,
}

/// Result of mock file operations
#[derive(Debug, Clone)]
pub enum MockFileResult {
    Success,
    Error(PlatformError),
}

impl Default for MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform {
    /// Create a new mock platform with default test data
    pub fn new() -> Self {
        let devices = vec![
            StorageDevice {
                name: "System Drive".to_string(),
                mount_point: PathBuf::from("/"),
                total_space: 1024 * 1024 * 1024 * 1024, // 1TB
                available_space: 512 * 1024 * 1024 * 1024, // 512GB
                device_type: DeviceType::SolidState,
            },
            StorageDevice {
                name: "Data Drive".to_string(),
                mount_point: PathBuf::from("/data"),
                total_space: 2 * 1024 * 1024 * 1024 * 1024, // 2TB
                available_space: 1024 * 1024 * 1024 * 1024, // 1TB
                device_type: DeviceType::HardDisk,
            },
            StorageDevice {
                name: "USB Drive".to_string(),
                mount_point: PathBuf::from("/media/usb"),
                total_space: 32 * 1024 * 1024 * 1024,     // 32GB
                available_space: 16 * 1024 * 1024 * 1024, // 16GB
                device_type: DeviceType::Removable,
            },
        ];

        Self {
            devices: Arc::new(Mutex::new(devices)),
            app_data_dir: Arc::new(Mutex::new(Some(PathBuf::from("/tmp/app_data")))),
            file_operations: Arc::new(Mutex::new(HashMap::new())),
            simulate_errors: Arc::new(Mutex::new(false)),
            simulated_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Add a device to the mock platform
    pub fn add_device(&self, device: StorageDevice) {
        self.devices.lock().unwrap().push(device);
    }

    /// Remove all devices from the mock platform
    pub fn clear_devices(&self) {
        self.devices.lock().unwrap().clear();
    }

    /// Set the app data directory
    pub fn set_app_data_dir(&self, path: Option<PathBuf>) {
        *self.app_data_dir.lock().unwrap() = path;
    }

    /// Set the result for a specific file operation
    pub fn set_file_operation_result(&self, path: PathBuf, result: MockFileResult) {
        self.file_operations.lock().unwrap().insert(path, result);
    }

    /// Enable error simulation
    pub fn simulate_error(&self, error: PlatformError) {
        *self.simulate_errors.lock().unwrap() = true;
        *self.simulated_error.lock().unwrap() = Some(error);
    }

    /// Disable error simulation
    pub fn disable_error_simulation(&self) {
        *self.simulate_errors.lock().unwrap() = false;
        *self.simulated_error.lock().unwrap() = None;
    }

    /// Check if errors are being simulated
    pub fn is_simulating_errors(&self) -> bool {
        *self.simulate_errors.lock().unwrap()
    }

    /// Get the current simulated error
    pub fn get_simulated_error(&self) -> Option<PlatformError> {
        self.simulated_error.lock().unwrap().clone()
    }

    /// Get the number of devices
    pub fn device_count(&self) -> usize {
        self.devices.lock().unwrap().len()
    }

    /// Get a specific device by index
    pub fn get_device(&self, index: usize) -> Option<StorageDevice> {
        self.devices.lock().unwrap().get(index).cloned()
    }

    /// Update device available space (simulate space changes)
    pub fn update_device_space(&self, index: usize, available_space: u64) {
        if let Some(device) = self.devices.lock().unwrap().get_mut(index) {
            device.available_space = available_space;
        }
    }
}

impl PlatformOps for MockPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // For static method, create a default instance
        let mock = MockPlatform::new();
        mock.list_storage_devices_instance()
    }

    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        let mock = MockPlatform::new();
        mock.get_app_data_dir_instance()
    }

    fn create_direct_io_file(path: &Path, _size: u64) -> Result<File, PlatformError> {
        let mock = MockPlatform::new();
        mock.create_direct_io_file_instance(path, _size)
    }

    fn open_direct_io_file(path: &Path, _write: bool) -> Result<File, PlatformError> {
        let mock = MockPlatform::new();
        mock.open_direct_io_file_instance(path, _write)
    }

    fn sync_file_system(_path: &Path) -> Result<(), PlatformError> {
        let mock = MockPlatform::new();
        mock.sync_file_system_instance(_path)
    }
}

impl MockPlatform {
    /// Instance method for listing storage devices
    pub fn list_storage_devices_instance(&self) -> Result<Vec<StorageDevice>, PlatformError> {
        if *self.simulate_errors.lock().unwrap() {
            if let Some(error) = self.simulated_error.lock().unwrap().clone() {
                return Err(error);
            }
        }

        Ok(self.devices.lock().unwrap().clone())
    }

    /// Instance method for getting app data directory
    pub fn get_app_data_dir_instance(&self) -> Result<PathBuf, PlatformError> {
        if *self.simulate_errors.lock().unwrap() {
            if let Some(error) = self.simulated_error.lock().unwrap().clone() {
                return Err(error);
            }
        }

        match self.app_data_dir.lock().unwrap().clone() {
            Some(path) => Ok(path),
            None => Err(PlatformError::UnsupportedPlatform(
                "No app data dir configured".to_string(),
            )),
        }
    }

    /// Instance method for creating direct I/O file
    pub fn create_direct_io_file_instance(
        &self,
        path: &Path,
        _size: u64,
    ) -> Result<File, PlatformError> {
        if *self.simulate_errors.lock().unwrap() {
            if let Some(error) = self.simulated_error.lock().unwrap().clone() {
                return Err(error);
            }
        }

        // Check for specific file operation results
        if let Some(result) = self.file_operations.lock().unwrap().get(path) {
            match result {
                MockFileResult::Success => {
                    // Create a temporary file for testing
                    std::fs::File::create(path).map_err(PlatformError::IoError)
                }
                MockFileResult::Error(error) => Err(error.clone()),
            }
        } else {
            // Default behavior: create a temporary file
            std::fs::File::create(path).map_err(PlatformError::IoError)
        }
    }

    /// Instance method for opening direct I/O file
    pub fn open_direct_io_file_instance(
        &self,
        path: &Path,
        write: bool,
    ) -> Result<File, PlatformError> {
        if *self.simulate_errors.lock().unwrap() {
            if let Some(error) = self.simulated_error.lock().unwrap().clone() {
                return Err(error);
            }
        }

        // Check for specific file operation results
        if let Some(result) = self.file_operations.lock().unwrap().get(path) {
            match result {
                MockFileResult::Success => {
                    if write {
                        std::fs::OpenOptions::new()
                            .write(true)
                            .open(path)
                            .map_err(PlatformError::IoError)
                    } else {
                        std::fs::File::open(path).map_err(PlatformError::IoError)
                    }
                }
                MockFileResult::Error(error) => Err(error.clone()),
            }
        } else {
            // Default behavior
            if write {
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(path)
                    .map_err(PlatformError::IoError)
            } else {
                std::fs::File::open(path).map_err(PlatformError::IoError)
            }
        }
    }

    /// Instance method for syncing file system
    pub fn sync_file_system_instance(&self, _path: &Path) -> Result<(), PlatformError> {
        if *self.simulate_errors.lock().unwrap() {
            if let Some(error) = self.simulated_error.lock().unwrap().clone() {
                return Err(error);
            }
        }

        // Mock implementation - always succeeds unless error is simulated
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_platform_creation() {
        let mock = MockPlatform::new();
        assert_eq!(mock.device_count(), 3); // Default devices
        assert!(!mock.is_simulating_errors());
    }

    #[test]
    fn test_mock_platform_device_management() {
        let mock = MockPlatform::new();

        // Test getting devices
        let device = mock.get_device(0).unwrap();
        assert_eq!(device.name, "System Drive");
        assert_eq!(device.device_type, DeviceType::SolidState);

        // Test adding device
        let new_device = StorageDevice {
            name: "Test Drive".to_string(),
            mount_point: PathBuf::from("/test"),
            total_space: 1024,
            available_space: 512,
            device_type: DeviceType::RamDisk,
        };
        mock.add_device(new_device.clone());
        assert_eq!(mock.device_count(), 4);

        let added_device = mock.get_device(3).unwrap();
        assert_eq!(added_device.name, "Test Drive");
        assert_eq!(added_device.device_type, DeviceType::RamDisk);

        // Test clearing devices
        mock.clear_devices();
        assert_eq!(mock.device_count(), 0);
    }

    #[test]
    fn test_mock_platform_app_data_dir() {
        let mock = MockPlatform::new();

        // Test default app data dir
        let app_dir = mock.get_app_data_dir_instance().unwrap();
        assert_eq!(app_dir, PathBuf::from("/tmp/app_data"));

        // Test setting custom app data dir
        let custom_dir = PathBuf::from("/custom/app/data");
        mock.set_app_data_dir(Some(custom_dir.clone()));
        let result = mock.get_app_data_dir_instance().unwrap();
        assert_eq!(result, custom_dir);

        // Test setting None
        mock.set_app_data_dir(None);
        let result = mock.get_app_data_dir_instance();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_platform_error_simulation() {
        let mock = MockPlatform::new();

        // Test normal operation
        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 3);

        // Test error simulation
        let error = PlatformError::DeviceEnumerationFailed("Test error".to_string());
        mock.simulate_error(error.clone());

        assert!(mock.is_simulating_errors());
        let result = mock.list_storage_devices_instance();
        assert!(result.is_err());

        // Test disabling error simulation
        mock.disable_error_simulation();
        assert!(!mock.is_simulating_errors());
        let devices = mock.list_storage_devices_instance().unwrap();
        assert_eq!(devices.len(), 3);
    }

    #[test]
    fn test_mock_platform_file_operations() {
        let mock = MockPlatform::new();
        let test_path = PathBuf::from("/test/file.tmp");

        // Test setting file operation result
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
            _ => panic!("Expected DirectIoNotSupported error"),
        }
    }

    #[test]
    fn test_mock_platform_device_space_updates() {
        let mock = MockPlatform::new();

        // Get initial device
        let device = mock.get_device(0).unwrap();
        let initial_space = device.available_space;

        // Update space
        let new_space = initial_space / 2;
        mock.update_device_space(0, new_space);

        // Verify update
        let updated_device = mock.get_device(0).unwrap();
        assert_eq!(updated_device.available_space, new_space);
    }

    #[test]
    fn test_mock_platform_static_methods() {
        // Test that static methods work (they create their own instances)
        let devices = MockPlatform::list_storage_devices().unwrap();
        assert_eq!(devices.len(), 3);

        let app_dir = MockPlatform::get_app_data_dir().unwrap();
        assert_eq!(app_dir, PathBuf::from("/tmp/app_data"));

        let sync_result = MockPlatform::sync_file_system(&PathBuf::from("/test"));
        assert!(sync_result.is_ok());
    }

    #[test]
    fn test_mock_platform_various_device_types() {
        let mock = MockPlatform::new();
        mock.clear_devices();

        // Add devices of different types
        let device_types = [
            DeviceType::HardDisk,
            DeviceType::SolidState,
            DeviceType::Removable,
            DeviceType::Network,
            DeviceType::RamDisk,
            DeviceType::OpticalDisk,
            DeviceType::Unknown,
        ];

        for (i, device_type) in device_types.iter().enumerate() {
            let device = StorageDevice {
                name: format!("Device {}", i),
                mount_point: PathBuf::from(format!("/dev{}", i)),
                total_space: 1024 * (i as u64 + 1),
                available_space: 512 * (i as u64 + 1),
                device_type: device_type.clone(),
            };
            mock.add_device(device);
        }

        assert_eq!(mock.device_count(), device_types.len());

        // Verify all device types are present
        for (i, device_type) in device_types.iter().enumerate() {
            let device = mock.get_device(i).unwrap();
            assert_eq!(device.device_type, *device_type);
        }
    }

    #[test]
    fn test_mock_platform_edge_cases() {
        let mock = MockPlatform::new();

        // Test getting non-existent device
        let result = mock.get_device(999);
        assert!(result.is_none());

        // Test updating non-existent device
        mock.update_device_space(999, 1024); // Should not panic

        // Test with empty device list
        mock.clear_devices();
        let devices = mock.list_storage_devices_instance().unwrap();
        assert!(devices.is_empty());
    }

    #[test]
    fn test_mock_platform_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let mock = Arc::new(MockPlatform::new());
        let mut handles = vec![];

        // Test concurrent device access
        for i in 0..5 {
            let mock_clone = mock.clone();
            let handle = thread::spawn(move || {
                let device = StorageDevice {
                    name: format!("Concurrent Device {}", i),
                    mount_point: PathBuf::from(format!("/concurrent{}", i)),
                    total_space: 1024,
                    available_space: 512,
                    device_type: DeviceType::Unknown,
                };
                mock_clone.add_device(device);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have original 3 devices + 5 concurrent devices
        assert_eq!(mock.device_count(), 8);
    }
}
