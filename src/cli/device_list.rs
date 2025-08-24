//! Device listing functionality for CLI

use anyhow::Result;
use disk_speed_test::platform;

/// List available storage devices
pub fn list_devices_command() -> Result<()> {
    println!("Available storage devices:");
    
    match platform::list_storage_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("No storage devices found or device enumeration not yet implemented for this platform.");
            } else {
                for device in devices {
                    println!("  {} - {} ({:.2} GB available / {:.2} GB total)",
                             device.name,
                             device.mount_point.display(),
                             device.available_space as f64 / (1024.0 * 1024.0 * 1024.0),
                             device.total_space as f64 / (1024.0 * 1024.0 * 1024.0));
                }
            }
        }
        Err(e) => {
            println!("Error listing devices: {}", e);
            println!("Device enumeration will be implemented in platform-specific tasks.");
        }
    }
    
    Ok(())
}