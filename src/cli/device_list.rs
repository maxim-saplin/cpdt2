//! Device listing functionality for CLI

use crate::platform::{get_platform_ops, StorageDevice};
use crate::BenchmarkResult;

/// List all available storage devices
pub fn list_devices() -> BenchmarkResult<Vec<StorageDevice>> {
    let platform_ops = get_platform_ops();
    let devices = platform_ops.list_storage_devices()?;
    Ok(devices)
}

/// Format device list for display
pub fn format_device_list(devices: &[StorageDevice]) -> String {
    if devices.is_empty() {
        return "No storage devices found.".to_string();
    }
    
    let mut output = String::new();
    output.push_str("Available Storage Devices:\n");
    output.push_str("=========================\n\n");
    
    for device in devices {
        output.push_str(&format!("Name: {}\n", device.name));
        output.push_str(&format!("Mount Point: {}\n", device.mount_point.display()));
        output.push_str(&format!("Type: {:?}\n", device.device_type));
        output.push_str(&format!("Total Space: {:.2} GB\n", device.total_space as f64 / 1024.0 / 1024.0 / 1024.0));
        output.push_str(&format!("Available Space: {:.2} GB\n", device.available_space as f64 / 1024.0 / 1024.0 / 1024.0));
        output.push_str("\n");
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{DeviceType};
    use std::path::PathBuf;

    #[test]
    fn test_format_empty_device_list() {
        let devices = vec![];
        let output = format_device_list(&devices);
        assert!(output.contains("No storage devices found"));
    }
    
    #[test]
    fn test_format_device_list() {
        let devices = vec![
            StorageDevice {
                name: "Test Drive".to_string(),
                mount_point: PathBuf::from("/test"),
                total_space: 1000000000, // ~1GB
                available_space: 500000000, // ~500MB
                device_type: DeviceType::Fixed,
            }
        ];
        
        let output = format_device_list(&devices);
        assert!(output.contains("Test Drive"));
        assert!(output.contains("/test"));
        assert!(output.contains("Fixed"));
    }
}