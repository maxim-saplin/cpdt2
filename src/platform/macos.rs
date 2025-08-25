//! macOS-specific platform operations

use super::{DeviceType, PlatformError, PlatformOps, StorageDevice};
use libc::c_int;
use std::fs::{read_dir, File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

// macOS-specific fcntl flags
const F_NOCACHE: c_int = 48;
const F_FULLFSYNC: c_int = 51;

/// macOS platform implementation
pub struct MacOsPlatform;

impl PlatformOps for MacOsPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        let mut devices = Vec::new();

        // Enumerate devices via /Volumes
        let volumes_path = Path::new("/Volumes");
        if volumes_path.exists() {
            match read_dir(volumes_path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Ok(device) = create_storage_device_from_path(&path) {
                                devices.push(device);
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(PlatformError::DeviceEnumerationFailed(format!(
                        "Failed to read /Volumes: {}",
                        e
                    )));
                }
            }
        }

        // Add root filesystem
        if let Ok(root_device) = create_storage_device_from_path(Path::new("/")) {
            devices.push(root_device);
        }

        Ok(devices)
    }

    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
            path.push("disk-speed-test");

            // Create the directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&path) {
                return Err(PlatformError::IoError(e));
            }

            Ok(path)
        } else {
            Err(PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not found",
            )))
        }
    }

    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // Create the parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(PlatformError::IoError(e));
            }
        }

        // Create the file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(PlatformError::IoError)?;

        // Set the file size
        file.set_len(size).map_err(PlatformError::IoError)?;

        // Apply F_NOCACHE flag to disable OS caching
        let fd = file.as_raw_fd();
        unsafe {
            if libc::fcntl(fd, F_NOCACHE, 1) == -1 {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        Ok(file)
    }

    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        let file = if write {
            OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(PlatformError::IoError)?
        } else {
            OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(PlatformError::IoError)?
        };

        // Apply F_NOCACHE flag to disable OS caching
        let fd = file.as_raw_fd();
        unsafe {
            if libc::fcntl(fd, F_NOCACHE, 1) == -1 {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        Ok(file)
    }

    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // Open the file or directory for synchronization
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(PlatformError::IoError)?;

        let fd = file.as_raw_fd();

        // Use F_FULLFSYNC for complete synchronization on macOS
        unsafe {
            if libc::fcntl(fd, F_FULLFSYNC) == -1 {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        Ok(())
    }
}

/// Create a StorageDevice from a filesystem path
fn create_storage_device_from_path(path: &Path) -> Result<StorageDevice, PlatformError> {
    use std::ffi::CString;

    let path_cstr = CString::new(path.to_string_lossy().as_bytes()).map_err(|e| {
        PlatformError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
    })?;

    let mut statvfs: libc::statvfs = unsafe { std::mem::zeroed() };

    unsafe {
        if libc::statvfs(path_cstr.as_ptr(), &mut statvfs) != 0 {
            return Err(PlatformError::IoError(std::io::Error::last_os_error()));
        }
    }

    let block_size = statvfs.f_frsize as u64;
    let total_blocks = statvfs.f_blocks as u64;
    let available_blocks = statvfs.f_bavail as u64;

    let total_space = total_blocks * block_size;
    let available_space = available_blocks * block_size;

    // Determine device type based on path and filesystem characteristics
    let device_type = determine_device_type(path, &statvfs);

    // Generate a human-readable name
    let name = if path == Path::new("/") {
        "Macintosh HD".to_string()
    } else if let Some(name) = path.file_name() {
        name.to_string_lossy().to_string()
    } else {
        path.to_string_lossy().to_string()
    };

    Ok(StorageDevice {
        name,
        mount_point: path.to_path_buf(),
        total_space,
        available_space,
        device_type,
    })
}

/// Determine the device type based on path and filesystem information
pub fn determine_device_type(path: &Path, _statvfs: &libc::statvfs) -> DeviceType {
    let path_str = path.to_string_lossy();

    // Check for common patterns to determine device type
    if path_str.contains("Network") || path_str.contains("net") {
        DeviceType::Network
    } else if path_str.contains("USB") || path_str.contains("External") {
        DeviceType::Removable
    } else if path == Path::new("/") {
        // Root filesystem - likely internal storage
        // On modern Macs, this is typically SSD
        DeviceType::SolidState
    } else if path_str.contains("RAM") || path_str.contains("ram") {
        DeviceType::RamDisk
    } else {
        // Default to unknown for mounted volumes we can't classify
        DeviceType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_app_data_dir() {
        let result = MacOsPlatform::get_app_data_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path
            .to_string_lossy()
            .contains("Library/Application Support/disk-speed-test"));

        // Verify the directory was created
        assert!(path.exists());
    }

    #[test]
    fn test_list_storage_devices() {
        let result = MacOsPlatform::list_storage_devices();
        assert!(result.is_ok());

        let devices = result.unwrap();
        // Should have at least the root filesystem
        assert!(!devices.is_empty());

        // Check that root filesystem is included
        let has_root = devices.iter().any(|d| d.mount_point == Path::new("/"));
        assert!(
            has_root,
            "Root filesystem should be included in device list"
        );

        // Verify device properties
        for device in &devices {
            assert!(!device.name.is_empty());
            assert!(device.mount_point.exists());
            // Total space should be greater than 0 for real devices
            if device.mount_point == Path::new("/") {
                assert!(device.total_space > 0);
            }
        }
    }

    #[test]
    fn test_create_and_open_direct_io_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_direct_io.dat");
        let test_size = 1024 * 1024; // 1MB

        // Test file creation with direct I/O
        let result = MacOsPlatform::create_direct_io_file(&test_file, test_size);
        assert!(result.is_ok());

        // Verify file exists and has correct size
        assert!(test_file.exists());
        let metadata = fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), test_size);

        // Test opening for read
        let read_result = MacOsPlatform::open_direct_io_file(&test_file, false);
        assert!(read_result.is_ok());

        // Test opening for write
        let write_result = MacOsPlatform::open_direct_io_file(&test_file, true);
        assert!(write_result.is_ok());
    }

    #[test]
    fn test_sync_file_system() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_sync.dat");

        // Create a test file
        fs::write(&test_file, b"test data").unwrap();

        // Test file system sync
        let result = MacOsPlatform::sync_file_system(&test_file);
        assert!(result.is_ok());

        // Test sync on directory
        let dir_result = MacOsPlatform::sync_file_system(temp_dir.path());
        assert!(dir_result.is_ok());
    }

    #[test]
    fn test_determine_device_type() {
        // Test root filesystem
        let statvfs: libc::statvfs = unsafe { std::mem::zeroed() };
        assert_eq!(
            determine_device_type(Path::new("/"), &statvfs),
            DeviceType::SolidState
        );

        // Test network paths
        assert_eq!(
            determine_device_type(Path::new("/Volumes/Network"), &statvfs),
            DeviceType::Network
        );

        // Test USB/External paths
        assert_eq!(
            determine_device_type(Path::new("/Volumes/USB Drive"), &statvfs),
            DeviceType::Removable
        );
        assert_eq!(
            determine_device_type(Path::new("/Volumes/External"), &statvfs),
            DeviceType::Removable
        );

        // Test RAM disk
        assert_eq!(
            determine_device_type(Path::new("/Volumes/RAM Disk"), &statvfs),
            DeviceType::RamDisk
        );

        // Test unknown
        assert_eq!(
            determine_device_type(Path::new("/Volumes/SomeVolume"), &statvfs),
            DeviceType::Unknown
        );
    }

    #[test]
    fn test_create_storage_device_from_path() {
        // Test with root filesystem
        let result = create_storage_device_from_path(Path::new("/"));
        assert!(result.is_ok());

        let device = result.unwrap();
        assert_eq!(device.name, "Macintosh HD");
        assert_eq!(device.mount_point, Path::new("/"));
        assert_eq!(device.device_type, DeviceType::SolidState);
        assert!(device.total_space > 0);
    }
}
