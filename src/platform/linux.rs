//! Linux-specific platform operations

use std::path::{Path, PathBuf};
use std::fs::File;
use super::{PlatformOps, StorageDevice, DeviceType, PlatformError};

/// Linux platform implementation
pub struct LinuxPlatform;

impl PlatformOps for LinuxPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // TODO: Implement Linux device enumeration in task 8
        // Will parse /proc/mounts and /sys/block
        Ok(vec![])
    }
    
    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // TODO: Implement Linux app data directory resolution in task 8
        // Will use ~/.local/share
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local");
            path.push("share");
            path.push("disk-speed-test");
            Ok(path)
        } else {
            Err(PlatformError::IoError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "HOME environment variable not found"
                )
            ))
        }
    }
    
    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // TODO: Implement Linux direct I/O file creation in task 8
        // Will use O_DIRECT and O_SYNC flags
        Err(PlatformError::DirectIoNotSupported)
    }
    
    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        // TODO: Implement Linux direct I/O file opening in task 8
        // Will use O_DIRECT and O_SYNC flags
        Err(PlatformError::DirectIoNotSupported)
    }
    
    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // TODO: Implement Linux file system synchronization in task 8
        // Will use fsync and other Linux-specific calls
        Ok(())
    }
}