//! Platform abstraction layer for cross-platform operations

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;

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

// Mock platform for testing
#[cfg(test)]
pub mod mock_platform;

#[cfg(test)]
mod platform_test;

// Platform-specific test modules
#[cfg(test)]
mod windows_test;

#[cfg(test)]
mod macos_test;

#[cfg(test)]
mod linux_test;

#[cfg(test)]
mod error_conditions_test;

#[cfg(test)]
mod mobile_platforms_test;

/// Errors that can occur in platform-specific operations
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Device enumeration failed: {0}")]
    DeviceEnumerationFailed(String),

    #[error("Direct I/O not supported on this platform")]
    DirectIoNotSupported,

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
}

impl Clone for PlatformError {
    fn clone(&self) -> Self {
        match self {
            PlatformError::IoError(e) => {
                // Create a new io::Error with the same kind and message
                PlatformError::IoError(std::io::Error::new(e.kind(), e.to_string()))
            }
            PlatformError::UnsupportedPlatform(s) => PlatformError::UnsupportedPlatform(s.clone()),
            PlatformError::DeviceEnumerationFailed(s) => {
                PlatformError::DeviceEnumerationFailed(s.clone())
            }
            PlatformError::DirectIoNotSupported => PlatformError::DirectIoNotSupported,
            PlatformError::InsufficientPermissions(s) => {
                PlatformError::InsufficientPermissions(s.clone())
            }
        }
    }
}

/// Types of storage devices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    /// Fixed hard disk drive
    HardDisk,
    /// Solid state drive
    SolidState,
    /// Removable drive (USB, etc.)
    Removable,
    /// Network drive
    Network,
    /// RAM disk
    RamDisk,
    /// CD/DVD/Blu-ray
    OpticalDisk,
    /// Unknown device type
    Unknown,
}

/// Information about a storage device
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

/// Platform-specific operations trait
pub trait PlatformOps {
    /// List all available storage devices
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError>
    where
        Self: Sized;

    /// Get the application data directory for the current platform
    fn get_app_data_dir() -> Result<PathBuf, PlatformError>
    where
        Self: Sized;

    /// Create a file with direct I/O flags for testing
    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError>
    where
        Self: Sized;

    /// Open a file with direct I/O flags
    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError>
    where
        Self: Sized;

    /// Synchronize file system to ensure data is written to disk
    fn sync_file_system(path: &Path) -> Result<(), PlatformError>
    where
        Self: Sized;
}

/// Convenience function to list storage devices
pub fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
    #[cfg(target_os = "windows")]
    return windows::WindowsPlatform::list_storage_devices();

    #[cfg(target_os = "macos")]
    return macos::MacOsPlatform::list_storage_devices();

    #[cfg(target_os = "linux")]
    return linux::LinuxPlatform::list_storage_devices();

    #[cfg(target_os = "android")]
    return android::AndroidPlatform::list_storage_devices();

    #[cfg(target_os = "ios")]
    return ios::IosPlatform::list_storage_devices();

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}

/// Convenience function to get app data directory
pub fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
    #[cfg(target_os = "windows")]
    return windows::WindowsPlatform::get_app_data_dir();

    #[cfg(target_os = "macos")]
    return macos::MacOsPlatform::get_app_data_dir();

    #[cfg(target_os = "linux")]
    return linux::LinuxPlatform::get_app_data_dir();

    #[cfg(target_os = "android")]
    return android::AndroidPlatform::get_app_data_dir();

    #[cfg(target_os = "ios")]
    return ios::IosPlatform::get_app_data_dir();

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}

/// Convenience function to create direct I/O file
pub fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
    #[cfg(target_os = "windows")]
    return windows::WindowsPlatform::create_direct_io_file(path, size);

    #[cfg(target_os = "macos")]
    return macos::MacOsPlatform::create_direct_io_file(path, size);

    #[cfg(target_os = "linux")]
    return linux::LinuxPlatform::create_direct_io_file(path, size);

    #[cfg(target_os = "android")]
    return android::AndroidPlatform::create_direct_io_file(path, size);

    #[cfg(target_os = "ios")]
    return ios::IosPlatform::create_direct_io_file(path, size);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}

/// Convenience function to open direct I/O file
pub fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
    #[cfg(target_os = "windows")]
    return windows::WindowsPlatform::open_direct_io_file(path, write);

    #[cfg(target_os = "macos")]
    return macos::MacOsPlatform::open_direct_io_file(path, write);

    #[cfg(target_os = "linux")]
    return linux::LinuxPlatform::open_direct_io_file(path, write);

    #[cfg(target_os = "android")]
    return android::AndroidPlatform::open_direct_io_file(path, write);

    #[cfg(target_os = "ios")]
    return ios::IosPlatform::open_direct_io_file(path, write);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}

/// Convenience function to sync file system
pub fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
    #[cfg(target_os = "windows")]
    return windows::WindowsPlatform::sync_file_system(path);

    #[cfg(target_os = "macos")]
    return macos::MacOsPlatform::sync_file_system(path);

    #[cfg(target_os = "linux")]
    return linux::LinuxPlatform::sync_file_system(path);

    #[cfg(target_os = "android")]
    return android::AndroidPlatform::sync_file_system(path);

    #[cfg(target_os = "ios")]
    return ios::IosPlatform::sync_file_system(path);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android",
        target_os = "ios"
    )))]
    compile_error!("Unsupported platform");
}
