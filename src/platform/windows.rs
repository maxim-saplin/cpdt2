//! Windows-specific platform operations

use std::ffi::OsString;
use std::fs::File;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::path::{Path, PathBuf};
use std::ptr;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::winerror::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{
    CreateFileW, FlushFileBuffers, GetDiskFreeSpaceW, GetDriveTypeW, GetLogicalDrives,
    CREATE_ALWAYS, OPEN_EXISTING,
};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::winbase::{
    DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOTE, DRIVE_REMOVABLE, DRIVE_UNKNOWN,
    FILE_FLAG_NO_BUFFERING, FILE_FLAG_SEQUENTIAL_SCAN, FILE_FLAG_WRITE_THROUGH,
};
use winapi::um::winnt::{
    FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE,
};

use super::{DeviceType, PlatformError, PlatformOps, StorageDevice};

/// Windows platform implementation
pub struct WindowsPlatform;

impl WindowsPlatform {
    /// Convert Windows drive type to our DeviceType enum
    fn drive_type_to_device_type(drive_type: DWORD) -> DeviceType {
        match drive_type {
            DRIVE_FIXED => DeviceType::HardDisk,
            DRIVE_REMOVABLE => DeviceType::Removable,
            DRIVE_CDROM => DeviceType::OpticalDisk,
            DRIVE_RAMDISK => DeviceType::RamDisk,
            DRIVE_REMOTE => DeviceType::Network,
            DRIVE_UNKNOWN | _ => DeviceType::Unknown,
        }
    }

    /// Get disk space information for a drive
    fn get_disk_space(drive_path: &str) -> Result<(u64, u64), PlatformError> {
        let wide_path: Vec<u16> = OsString::from(drive_path)
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut sectors_per_cluster: DWORD = 0;
        let mut bytes_per_sector: DWORD = 0;
        let mut number_of_free_clusters: DWORD = 0;
        let mut total_number_of_clusters: DWORD = 0;

        unsafe {
            let result = GetDiskFreeSpaceW(
                wide_path.as_ptr(),
                &mut sectors_per_cluster,
                &mut bytes_per_sector,
                &mut number_of_free_clusters,
                &mut total_number_of_clusters,
            );

            if result == FALSE {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        let bytes_per_cluster = (sectors_per_cluster as u64) * (bytes_per_sector as u64);
        let total_bytes = (total_number_of_clusters as u64) * bytes_per_cluster;
        let free_bytes = (number_of_free_clusters as u64) * bytes_per_cluster;

        Ok((total_bytes, free_bytes))
    }

    /// Convert a Windows handle to a Rust File
    unsafe fn handle_to_file(handle: winapi::um::winnt::HANDLE) -> Result<File, PlatformError> {
        if handle == INVALID_HANDLE_VALUE {
            return Err(PlatformError::IoError(std::io::Error::last_os_error()));
        }
        Ok(File::from_raw_handle(handle as *mut std::ffi::c_void))
    }

    /// Set file size using Windows API
    fn set_file_size(file: &File, size: u64) -> Result<(), PlatformError> {
        file.set_len(size).map_err(PlatformError::IoError)
    }
}

impl PlatformOps for WindowsPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        let mut devices = Vec::new();

        unsafe {
            let drives_mask = GetLogicalDrives();
            if drives_mask == 0 {
                return Err(PlatformError::DeviceEnumerationFailed(
                    "Failed to get logical drives".to_string(),
                ));
            }

            // Check each possible drive letter (A-Z)
            for i in 0..26 {
                if (drives_mask & (1 << i)) != 0 {
                    let drive_letter = (b'A' + i) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    let drive_root = format!("{}:\\", drive_letter);

                    // Convert to wide string for Windows API
                    let wide_path: Vec<u16> = OsString::from(&drive_root)
                        .as_os_str()
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    let drive_type = GetDriveTypeW(wide_path.as_ptr());

                    // Skip unknown drives and some system drives
                    if drive_type == DRIVE_UNKNOWN {
                        continue;
                    }

                    // Get disk space information
                    let (total_space, available_space) = match Self::get_disk_space(&drive_root) {
                        Ok((total, available)) => (total, available),
                        Err(_) => continue, // Skip drives we can't access
                    };

                    let device = StorageDevice {
                        name: format!("Drive {}", drive_letter),
                        mount_point: PathBuf::from(drive_path),
                        total_space,
                        available_space,
                        device_type: Self::drive_type_to_device_type(drive_type),
                    };

                    devices.push(device);
                }
            }
        }

        Ok(devices)
    }

    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        std::env::var("LOCALAPPDATA")
            .map(|path| PathBuf::from(path).join("disk-speed-test"))
            .map_err(|_| {
                PlatformError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "LOCALAPPDATA environment variable not found",
                ))
            })
    }

    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = CreateFileW(
                wide_path.as_ptr(),
                GENERIC_WRITE | GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL
                    | FILE_FLAG_NO_BUFFERING
                    | FILE_FLAG_WRITE_THROUGH
                    | FILE_FLAG_SEQUENTIAL_SCAN,
                ptr::null_mut(),
            );

            let file = Self::handle_to_file(handle)?;

            // Set the file size
            Self::set_file_size(&file, size)?;

            Ok(file)
        }
    }

    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let access = if write {
            GENERIC_WRITE | GENERIC_READ
        } else {
            GENERIC_READ
        };

        unsafe {
            let handle = CreateFileW(
                wide_path.as_ptr(),
                access,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL
                    | FILE_FLAG_NO_BUFFERING
                    | FILE_FLAG_WRITE_THROUGH
                    | FILE_FLAG_SEQUENTIAL_SCAN,
                ptr::null_mut(),
            );

            Self::handle_to_file(handle)
        }
    }

    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // For Windows, we'll open the file and flush it
        let file = std::fs::File::open(path).map_err(|e| PlatformError::IoError(e))?;

        let handle = file.as_raw_handle();

        unsafe {
            let result = FlushFileBuffers(handle as winapi::um::winnt::HANDLE);
            if result == FALSE {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_list_storage_devices() {
        let devices = WindowsPlatform::list_storage_devices().unwrap();

        // Should have at least one device (C: drive typically exists)
        assert!(!devices.is_empty());

        // Check that each device has valid properties
        for device in &devices {
            assert!(!device.name.is_empty());
            assert!(device.mount_point.is_absolute());
            assert!(device.total_space > 0);
            // Available space should be <= total space
            assert!(device.available_space <= device.total_space);
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_app_data_dir() {
        let app_data_dir = WindowsPlatform::get_app_data_dir().unwrap();

        // Should end with our app name
        assert!(app_data_dir.ends_with("disk-speed-test"));

        // Should be an absolute path
        assert!(app_data_dir.is_absolute());

        // Should contain LOCALAPPDATA path
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap();
        assert!(app_data_dir.starts_with(&local_app_data));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_create_and_open_direct_io_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_direct_io.bin");
        let test_size = 1024 * 1024; // 1MB

        // Create direct I/O file
        let file = WindowsPlatform::create_direct_io_file(&test_file_path, test_size).unwrap();
        drop(file); // Close the file

        // Verify file exists and has correct size
        let metadata = fs::metadata(&test_file_path).unwrap();
        assert_eq!(metadata.len(), test_size);

        // Open for reading
        let _read_file = WindowsPlatform::open_direct_io_file(&test_file_path, false).unwrap();

        // Open for writing
        let _write_file = WindowsPlatform::open_direct_io_file(&test_file_path, true).unwrap();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_sync_file_system() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_sync.txt");

        // Create a test file
        fs::write(&test_file_path, "test data").unwrap();

        // Sync should not fail
        WindowsPlatform::sync_file_system(&test_file_path).unwrap();
    }

    #[test]
    fn test_drive_type_conversion() {
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_FIXED),
            DeviceType::HardDisk
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_REMOVABLE),
            DeviceType::Removable
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_CDROM),
            DeviceType::OpticalDisk
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_RAMDISK),
            DeviceType::RamDisk
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_REMOTE),
            DeviceType::Network
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(DRIVE_UNKNOWN),
            DeviceType::Unknown
        );
        assert_eq!(
            WindowsPlatform::drive_type_to_device_type(999),
            DeviceType::Unknown
        ); // Invalid type
    }
}
