//! Linux-specific platform operations

use super::{DeviceType, PlatformError, PlatformOps, StorageDevice};
use libc::{fsync, sync, O_DIRECT, O_SYNC};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Linux platform implementation
pub struct LinuxPlatform;

impl LinuxPlatform {
    /// Get the logical sector size for alignment (typically 512 bytes)
    const SECTOR_SIZE: u64 = 512;

    /// Align a size to sector boundaries for direct I/O compatibility
    fn align_to_sector_size(size: u64) -> u64 {
        let remainder = size % Self::SECTOR_SIZE;
        if remainder == 0 {
            size
        } else {
            size + (Self::SECTOR_SIZE - remainder)
        }
    }

    /// Ensure block size is compatible with direct I/O (multiple of sector size)
    pub fn align_block_size_for_direct_io(block_size: usize) -> usize {
        Self::align_to_sector_size(block_size as u64) as usize
    }

    /// Parse /proc/mounts to get mounted filesystems
    fn parse_proc_mounts() -> Result<Vec<MountInfo>, PlatformError> {
        let file = File::open("/proc/mounts").map_err(|e| {
            PlatformError::DeviceEnumerationFailed(format!("Failed to open /proc/mounts: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut mounts = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| {
                PlatformError::DeviceEnumerationFailed(format!(
                    "Failed to read /proc/mounts: {}",
                    e
                ))
            })?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 4 {
                let device = parts[0];
                let mount_point = parts[1];
                let fs_type = parts[2];

                // Skip virtual filesystems and special mounts
                if Self::is_real_filesystem(device, fs_type) {
                    mounts.push(MountInfo {
                        device: device.to_string(),
                        mount_point: PathBuf::from(mount_point),
                        _fs_type: fs_type.to_string(),
                    });
                }
            }
        }

        Ok(mounts)
    }

    /// Check if this is a real filesystem we should include
    pub fn is_real_filesystem(device: &str, fs_type: &str) -> bool {
        // Skip virtual filesystems
        let virtual_fs = [
            "proc",
            "sysfs",
            "devfs",
            "tmpfs",
            "devpts",
            "cgroup",
            "cgroup2",
            "pstore",
            "bpf",
            "tracefs",
            "debugfs",
            "securityfs",
            "hugetlbfs",
            "mqueue",
            "configfs",
            "fusectl",
            "selinuxfs",
            "binfmt_misc",
        ];

        if virtual_fs.contains(&fs_type) {
            return false;
        }

        // Skip devices that don't look like real block devices
        if device.starts_with("/proc")
            || device.starts_with("/sys")
            || device.starts_with("/dev/pts")
        {
            return false;
        }

        true
    }

    /// Get device information from /sys/block
    pub(crate) fn get_device_info(device_path: &str) -> Result<DeviceInfo, PlatformError> {
        // Extract device name from path like /dev/sda1 -> sda
        let device_name = if let Some(name) = device_path.strip_prefix("/dev/") {
            // Remove partition numbers (e.g., sda1 -> sda, nvme0n1p1 -> nvme0n1)
            if name.contains("nvme") {
                // NVMe devices: nvme0n1p1 -> nvme0n1
                if let Some(pos) = name.rfind('p') {
                    &name[..pos]
                } else {
                    name
                }
            } else {
                // Regular devices: sda1 -> sda
                name.trim_end_matches(char::is_numeric)
            }
        } else {
            return Ok(DeviceInfo {
                device_type: DeviceType::Unknown,
                _rotational: None,
            });
        };

        let sys_block_path = format!("/sys/block/{}", device_name);

        // Check if device is rotational (HDD vs SSD)
        let rotational = Self::read_sys_file(&format!("{}/queue/rotational", sys_block_path))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .map(|r| r != 0);

        // Determine device type
        let device_type = if device_path.starts_with("/dev/loop") {
            DeviceType::Unknown
        } else if device_path.contains("usb") || Self::is_removable_device(&sys_block_path) {
            DeviceType::Removable
        } else if rotational == Some(true) {
            DeviceType::HardDisk
        } else if rotational == Some(false) {
            DeviceType::SolidState
        } else {
            DeviceType::Unknown
        };

        Ok(DeviceInfo {
            device_type,
            _rotational: rotational,
        })
    }

    /// Check if device is removable
    fn is_removable_device(sys_block_path: &str) -> bool {
        Self::read_sys_file(&format!("{}/removable", sys_block_path))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .map(|r| r != 0)
            .unwrap_or(false)
    }

    /// Read a file from /sys
    fn read_sys_file(path: &str) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }

    /// Get filesystem statistics using statvfs
    fn get_filesystem_stats(path: &Path) -> Result<FilesystemStats, PlatformError> {
        use std::ffi::CString;
        use std::mem::MaybeUninit;

        let path_cstr = CString::new(path.to_string_lossy().as_bytes()).map_err(|e| {
            PlatformError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })?;

        let mut statvfs = MaybeUninit::<libc::statvfs>::uninit();

        let result = unsafe { libc::statvfs(path_cstr.as_ptr(), statvfs.as_mut_ptr()) };

        if result != 0 {
            return Err(PlatformError::IoError(std::io::Error::last_os_error()));
        }

        let statvfs = unsafe { statvfs.assume_init() };

        let block_size = statvfs.f_frsize;
        let total_blocks = statvfs.f_blocks;
        let available_blocks = statvfs.f_bavail;

        Ok(FilesystemStats {
            total_space: total_blocks * block_size,
            available_space: available_blocks * block_size,
        })
    }

    /// Create directory if it doesn't exist
    fn ensure_directory_exists(path: &Path) -> Result<(), PlatformError> {
        if !path.exists() {
            std::fs::create_dir_all(path).map_err(PlatformError::IoError)?;
        }
        Ok(())
    }
}

impl PlatformOps for LinuxPlatform {
    fn list_storage_devices() -> Result<Vec<StorageDevice>, PlatformError> {
        let mounts = Self::parse_proc_mounts()?;
        let mut devices = Vec::new();
        let mut seen_devices = HashMap::new();

        for mount in mounts {
            // Skip if we've already processed this device
            if seen_devices.contains_key(&mount.device) {
                continue;
            }

            let device_info = Self::get_device_info(&mount.device)?;
            let fs_stats = Self::get_filesystem_stats(&mount.mount_point)?;

            // Create a human-readable name
            let name = if mount.mount_point == PathBuf::from("/") {
                format!("Root Filesystem ({})", mount.device)
            } else if let Some(mount_name) = mount.mount_point.file_name() {
                format!("{} ({})", mount_name.to_string_lossy(), mount.device)
            } else {
                format!("{} ({})", mount.mount_point.display(), mount.device)
            };

            let storage_device = StorageDevice {
                name,
                mount_point: mount.mount_point,
                total_space: fs_stats.total_space,
                available_space: fs_stats.available_space,
                device_type: device_info.device_type,
            };

            devices.push(storage_device);
            seen_devices.insert(mount.device, ());
        }

        // Sort devices by mount point for consistent ordering
        devices.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));

        Ok(devices)
    }

    fn get_app_data_dir() -> Result<PathBuf, PlatformError> {
        // Use XDG Base Directory specification
        // First try XDG_DATA_HOME, then fall back to ~/.local/share
        let data_dir = if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home)
        } else if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local");
            path.push("share");
            path
        } else {
            return Err(PlatformError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Neither XDG_DATA_HOME nor HOME environment variable found",
            )));
        };

        let mut app_data_dir = data_dir;
        app_data_dir.push("disk-speed-test");

        // Ensure the directory exists
        Self::ensure_directory_exists(&app_data_dir)?;

        Ok(app_data_dir)
    }

    fn create_direct_io_file(path: &Path, size: u64) -> Result<File, PlatformError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            Self::ensure_directory_exists(parent)?;
        }

        // Try to create file with O_DIRECT and O_SYNC for direct I/O
        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .custom_flags(O_DIRECT | O_SYNC)
            .open(path)
        {
            Ok(file) => file,
            Err(e) if e.raw_os_error() == Some(libc::EINVAL) => {
                // O_DIRECT not supported on this filesystem
                return Err(PlatformError::DirectIoNotSupported);
            }
            Err(e) => return Err(PlatformError::IoError(e)),
        };

        // Set file size - ensure it's aligned to sector boundaries for direct I/O
        let aligned_size = Self::align_to_sector_size(size);
        file.set_len(aligned_size).map_err(PlatformError::IoError)?;

        Ok(file)
    }

    fn open_direct_io_file(path: &Path, write: bool) -> Result<File, PlatformError> {
        let mut options = OpenOptions::new();

        if write {
            options.write(true);
        } else {
            options.read(true);
        }

        // Try to open file with O_DIRECT
        let file = match options.custom_flags(O_DIRECT | O_SYNC).open(path) {
            Ok(file) => file,
            Err(e) if e.raw_os_error() == Some(libc::EINVAL) => {
                // O_DIRECT not supported on this filesystem
                return Err(PlatformError::DirectIoNotSupported);
            }
            Err(e) => return Err(PlatformError::IoError(e)),
        };

        Ok(file)
    }

    fn sync_file_system(path: &Path) -> Result<(), PlatformError> {
        // First try to sync the specific file if it exists
        if path.is_file() {
            let file = File::open(path).map_err(PlatformError::IoError)?;

            let fd = std::os::unix::io::AsRawFd::as_raw_fd(&file);
            let result = unsafe { fsync(fd) };

            if result != 0 {
                return Err(PlatformError::IoError(std::io::Error::last_os_error()));
            }
        }

        // Then sync the entire filesystem
        unsafe {
            sync();
        }

        Ok(())
    }
}

/// Information about a mounted filesystem
#[derive(Debug)]
struct MountInfo {
    device: String,
    mount_point: PathBuf,
    _fs_type: String,
}

/// Device information from /sys/block
#[derive(Debug)]
pub(crate) struct DeviceInfo {
    pub(crate) device_type: DeviceType,
    pub(crate) _rotational: Option<bool>,
}

/// Filesystem statistics
#[derive(Debug)]
struct FilesystemStats {
    total_space: u64,
    available_space: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_get_app_data_dir() {
        let result = LinuxPlatform::get_app_data_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("disk-speed-test"));

        // Should contain either .local/share or XDG_DATA_HOME
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".local/share") || std::env::var("XDG_DATA_HOME").is_ok());
    }

    #[test]
    fn test_list_storage_devices() {
        let result = LinuxPlatform::list_storage_devices();

        // This should work on any Linux system
        assert!(result.is_ok());

        let devices = result.unwrap();
        // Should have at least the root filesystem
        assert!(!devices.is_empty());

        // Check that we have reasonable device information
        for device in &devices {
            assert!(!device.name.is_empty());
            assert!(device.mount_point.exists());
            // Total space should be reasonable (> 0)
            assert!(device.total_space > 0);
        }
    }

    #[test]
    fn test_is_real_filesystem() {
        // Real filesystems
        assert!(LinuxPlatform::is_real_filesystem("/dev/sda1", "ext4"));
        assert!(LinuxPlatform::is_real_filesystem("/dev/nvme0n1p1", "xfs"));
        assert!(LinuxPlatform::is_real_filesystem(
            "/dev/mapper/root",
            "btrfs"
        ));

        // Virtual filesystems should be filtered out
        assert!(!LinuxPlatform::is_real_filesystem("proc", "proc"));
        assert!(!LinuxPlatform::is_real_filesystem("sysfs", "sysfs"));
        assert!(!LinuxPlatform::is_real_filesystem("tmpfs", "tmpfs"));
        assert!(!LinuxPlatform::is_real_filesystem("devpts", "devpts"));
        assert!(!LinuxPlatform::is_real_filesystem(
            "binfmt_misc",
            "binfmt_misc"
        ));
    }

    #[test]
    fn test_create_and_open_direct_io_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_direct_io.dat");
        let file_size = 1024 * 1024; // 1MB

        // Test file creation
        let result = LinuxPlatform::create_direct_io_file(&test_file, file_size);
        assert!(result.is_ok());

        // Verify file exists and has correct size
        assert!(test_file.exists());
        let metadata = std::fs::metadata(&test_file).unwrap();
        assert_eq!(metadata.len(), file_size);

        // Test opening for read
        let read_result = LinuxPlatform::open_direct_io_file(&test_file, false);
        assert!(read_result.is_ok());

        // Test opening for write
        let write_result = LinuxPlatform::open_direct_io_file(&test_file, true);
        assert!(write_result.is_ok());
    }

    #[test]
    fn test_sync_file_system() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_sync.dat");

        // Create a test file
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test data").unwrap();
        drop(file);

        // Test sync
        let result = LinuxPlatform::sync_file_system(&test_file);
        assert!(result.is_ok());

        // Test sync on directory
        let dir_result = LinuxPlatform::sync_file_system(temp_dir.path());
        assert!(dir_result.is_ok());
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("directory");

        // Directory shouldn't exist initially
        assert!(!nested_dir.exists());

        // Create it
        let result = LinuxPlatform::ensure_directory_exists(&nested_dir);
        assert!(result.is_ok());

        // Should exist now
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());

        // Calling again should be fine
        let result2 = LinuxPlatform::ensure_directory_exists(&nested_dir);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_device_info_parsing() {
        // Test device type determination
        let info = LinuxPlatform::get_device_info("/dev/sda1");
        assert!(info.is_ok());

        let info = LinuxPlatform::get_device_info("/dev/nvme0n1p1");
        assert!(info.is_ok());

        // Test unknown device
        let info = LinuxPlatform::get_device_info("/dev/unknown123");
        assert!(info.is_ok());
        let device_info = info.unwrap();
        assert_eq!(device_info.device_type, DeviceType::Unknown);
    }

    #[test]
    fn test_read_sys_file() {
        // Test reading a file that should exist on most Linux systems
        if std::path::Path::new("/sys/kernel/hostname").exists() {
            let result = LinuxPlatform::read_sys_file("/sys/kernel/hostname");
            assert!(result.is_ok());
            assert!(!result.unwrap().trim().is_empty());
        }

        // Test reading non-existent file
        let result = LinuxPlatform::read_sys_file("/sys/nonexistent/file");
        assert!(result.is_err());
    }

    #[test]
    fn test_platform_integration() {
        // Test that the Linux platform can be called through the platform abstraction
        // This test will only run on Linux, but ensures the integration works

        // Test app data directory
        let app_data_result = LinuxPlatform::get_app_data_dir();
        assert!(app_data_result.is_ok());

        // Test device listing (should work even if no devices found)
        let devices_result = LinuxPlatform::list_storage_devices();
        assert!(devices_result.is_ok());
    }
}
