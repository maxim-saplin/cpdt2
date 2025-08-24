//! iOS-specific platform implementation

use super::{PlatformOps, PlatformError, StorageDevice, DeviceType};
use std::path::{Path, PathBuf};
use std::fs::File;

/// iOS platform implementation
pub struct IosPlatform;

impl PlatformOps for IosPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // Stub implementation - iOS support will be added in future tasks
        todo!("iOS device enumeration will be implemented in future mobile support tasks")
    }
    
    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // Stub implementation - iOS support will be added in future tasks
        todo!("iOS app data directory resolution will be implemented in future mobile support tasks")
    }
    
    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // Stub implementation - iOS support will be added in future tasks
        todo!("iOS direct I/O file creation will be implemented in future mobile support tasks")
    }
    
    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        // Stub implementation - iOS support will be added in future tasks
        todo!("iOS direct I/O file opening will be implemented in future mobile support tasks")
    }
    
    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // Stub implementation - iOS support will be added in future tasks
        todo!("iOS file system sync will be implemented in future mobile support tasks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_platform_exists() {
        // Basic test to ensure the struct exists
        let _platform = IosPlatform;
        assert!(true);
    }
}