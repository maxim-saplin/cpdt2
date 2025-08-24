//! Android-specific platform implementation

use super::{PlatformOps, PlatformError, StorageDevice, DeviceType};
use std::path::{Path, PathBuf};
use std::fs::File;

/// Android platform implementation
pub struct AndroidPlatform;

impl PlatformOps for AndroidPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // Stub implementation - Android support will be added in future tasks
        todo!("Android device enumeration will be implemented in future mobile support tasks")
    }
    
    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // Stub implementation - Android support will be added in future tasks
        todo!("Android app data directory resolution will be implemented in future mobile support tasks")
    }
    
    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // Stub implementation - Android support will be added in future tasks
        todo!("Android direct I/O file creation will be implemented in future mobile support tasks")
    }
    
    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        // Stub implementation - Android support will be added in future tasks
        todo!("Android direct I/O file opening will be implemented in future mobile support tasks")
    }
    
    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // Stub implementation - Android support will be added in future tasks
        todo!("Android file system sync will be implemented in future mobile support tasks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android_platform_exists() {
        // Basic test to ensure the struct exists
        let _platform = AndroidPlatform;
        assert!(true);
    }
}