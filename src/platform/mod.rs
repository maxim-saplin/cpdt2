//! Platform abstraction layer for cross-platform operations

use std::path::{Path, PathBuf};
use std::fs::File;
use thiserror::Error;
use serde::{Deserialize, Serialize};

// Platform-specific modules
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

/// Platform-specific error types
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Device enumeration failed: {0}")]
    DeviceEnumerationFailed(String),
    
    #[error("App data directory not found: {0}")]
    AppDataDirNotFound(String),
    
    #[error("Direct I/O not supported: {0}")]
    DirectIoNotSupported(String),
    
    #[error("File system sync failed: {0}")]
    SyncFailed(String),
    
    #[error("Platform operation failed: {0}")]
    OperationFailed(String),
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Human-readable device name
    pub name: String,
    
    /// Mount point or drive letter
    pub mount_point: PathBuf,
    
    /// Total space in bytes
    pub total_space: u64,
    
    /// Available space in bytes
    pub available_space: u64,
    
    /// Type of storage device
    pub device_type: DeviceType,
}

/// Types of storage devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    /// Fixed disk (HDD, SSD)
    Fixed,
    
    /// Removable disk (USB, SD card)
    Removable,
    
    /// Network drive
    Network,
    
    /// RAM disk
    Ram,
    
    /// Unknown type
    Unknown,
}

/// Platform operations trait
pub trait PlatformOps {
    /// List all available storage devices
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError>;
    
    /// Get the application data directory for the current platform
    fn get_app_data_dir() -> Result<PathBuf, PlatformError>;
    
    /// Create a file with direct I/O flags for testing
    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError>;
    
    /// Open a file with direct I/O flags
    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError>;
    
    /// Synchronize file system to ensure data is written to disk
    fn sync_file_system(path: &Path) -> Result<(), PlatformError>;
}

/// Get the platform-specific implementation
pub fn get_platform_ops() -> Box<dyn PlatformOps> {
    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsPlatform);
    
    #[cfg(target_os = "macos")]
    return Box::new(macos::MacOsPlatform);
    
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxPlatform);
    
    #[cfg(target_os = "android")]
    return Box::new(android::AndroidPlatform);
    
    #[cfg(target_os = "ios")]
    return Box::new(ios::IosPlatform);
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos", 
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_device_serialization() {
        let device = StorageDevice {
            name: "Test Drive".to_string(),
            mount_point: PathBuf::from("/test"),
            total_space: 1000000000,
            available_space: 500000000,
            device_type: DeviceType::Fixed,
        };
        
        let json = serde_json::to_string(&device).unwrap();
        let deserialized: StorageDevice = serde_json::from_str(&json).unwrap();
        
        assert_eq!(device.name, deserialized.name);
        assert_eq!(device.total_space, deserialized.total_space);
    }
    
    #[test]
    fn test_platform_ops_available() {
        // This test ensures we can get a platform implementation
        let _ops = get_platform_ops();
        assert!(true);
    }
}