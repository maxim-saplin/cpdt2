//! macOS-specific platform operations

use std::path::{Path, PathBuf};
use std::fs::File;
use super::{PlatformOps, StorageDevice, PlatformError};

/// macOS platform implementation
pub struct MacOsPlatform;

impl PlatformOps for MacOsPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        // TODO: Implement macOS device enumeration in task 7
        // Will enumerate via /Volumes and system APIs
        Ok(vec![])
    }
    
    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // TODO: Implement macOS app data directory resolution in task 7
        // Will use ~/Library/Application Support
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
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
    
    fn create_direct_io_file(_path: &Path, _size: u64) -> Result<File, PlatformError> {
        // TODO: Implement macOS direct I/O file creation in task 7
        // Will use F_NOCACHE fcntl flag
        Err(PlatformError::DirectIoNotSupported)
    }
    
    fn open_direct_io_file(_path: &Path, _write: bool) -> Result<File, PlatformError> {
        // TODO: Implement macOS direct I/O file opening in task 7
        // Will use F_NOCACHE fcntl flag
        Err(PlatformError::DirectIoNotSupported)
    }
    
    fn sync_file_system(_path: &Path) -> Result<(), PlatformError> {
        // TODO: Implement macOS file system synchronization in task 7
        // Will use F_FULLFSYNC for synchronization
        Ok(())
    }
}