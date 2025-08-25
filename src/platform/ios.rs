//! iOS-specific platform operations

use super::{DeviceType, PlatformError, PlatformOps, StorageDevice};
use std::fs::File;
use std::path::{Path, PathBuf};

/// iOS platform implementation
pub struct IosPlatform;

impl PlatformOps for IosPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // TODO: Implement iOS device enumeration
        // Will work within app sandbox constraints
        Ok(vec![])
    }

    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // TODO: Implement iOS app data directory resolution
        // Will use iOS-specific app directories
        Err(PlatformError::UnsupportedPlatform(
            "iOS app data directory resolution not yet implemented".to_string(),
        ))
    }

    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // TODO: Implement iOS direct I/O file creation
        // Will work within iOS sandbox constraints
        Err(PlatformError::DirectIoNotSupported)
    }

    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        // TODO: Implement iOS direct I/O file opening
        // Will work within iOS sandbox constraints
        Err(PlatformError::DirectIoNotSupported)
    }

    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // TODO: Implement iOS file system synchronization
        // Will use iOS-specific sync mechanisms
        Ok(())
    }
}
