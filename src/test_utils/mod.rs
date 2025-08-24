//! Test utilities for creating controlled test environments and managing test data

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, NamedTempFile};
use anyhow::Result;

pub mod test_data;
pub mod test_environment;
pub mod cleanup;

// Re-export commonly used types
pub use test_environment::{TestEnvironment, TestEnvironmentBuilder, TestEnvironmentConfig, TimeoutGuard};
pub use test_data::{TestDataGenerator, TestDataPattern, TestDataVerifier};
pub use cleanup::{CleanupGuard, CleanupRegistry, CleanupStats};

/// Test data manager for handling temporary directories and files
pub struct TestDataManager {
    temp_dir: TempDir,
    test_files: Vec<PathBuf>,
}

impl TestDataManager {
    /// Create a new test data manager with a temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self {
            temp_dir,
            test_files: Vec::new(),
        })
    }

    /// Get the path to the temporary directory
    pub fn temp_dir_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a test file with specified size in bytes
    pub fn create_test_file(&mut self, name: &str, size_bytes: u64) -> Result<PathBuf> {
        let file_path = self.temp_dir.path().join(name);
        
        // Create file with specified size
        let file = fs::File::create(&file_path)?;
        file.set_len(size_bytes)?;
        
        self.test_files.push(file_path.clone());
        Ok(file_path)
    }

    /// Create a test file with random data
    pub fn create_random_test_file(&mut self, name: &str, size_bytes: u64) -> Result<PathBuf> {
        use std::io::Write;
        use rand::RngCore;
        
        let file_path = self.temp_dir.path().join(name);
        let mut file = fs::File::create(&file_path)?;
        
        // Write random data in chunks to avoid memory issues
        let chunk_size = 1024 * 1024; // 1MB chunks
        let mut rng = rand::thread_rng();
        let mut remaining = size_bytes;
        
        while remaining > 0 {
            let current_chunk_size = std::cmp::min(chunk_size, remaining);
            let mut chunk = vec![0u8; current_chunk_size as usize];
            rng.fill_bytes(&mut chunk);
            file.write_all(&chunk)?;
            remaining -= current_chunk_size;
        }
        
        file.sync_all()?;
        self.test_files.push(file_path.clone());
        Ok(file_path)
    }

    /// Create a temporary named file
    pub fn create_named_temp_file(&mut self) -> Result<NamedTempFile> {
        let temp_file = NamedTempFile::new_in(self.temp_dir.path())?;
        self.test_files.push(temp_file.path().to_path_buf());
        Ok(temp_file)
    }

    /// Get list of all created test files
    pub fn test_files(&self) -> &[PathBuf] {
        &self.test_files
    }

    /// Clean up specific test file
    pub fn cleanup_file(&mut self, file_path: &Path) -> Result<()> {
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        self.test_files.retain(|p| p != file_path);
        Ok(())
    }

    /// Get available space in temporary directory
    pub fn available_space(&self) -> Result<u64> {
        // For testing purposes, return a reasonable amount of space
        // In a real implementation, you'd use platform-specific APIs like statvfs
        Ok(1024 * 1024 * 1024) // 1GB
    }
}

impl Drop for TestDataManager {
    fn drop(&mut self) {
        // Cleanup is handled automatically by TempDir
        // But we can add custom cleanup logic here if needed
        for file_path in &self.test_files {
            if file_path.exists() {
                let _ = fs::remove_file(file_path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_manager_creation() {
        let manager = TestDataManager::new().unwrap();
        assert!(manager.temp_dir_path().exists());
    }

    #[test]
    fn test_file_creation() {
        let mut manager = TestDataManager::new().unwrap();
        let file_path = manager.create_test_file("test.dat", 1024).unwrap();
        
        assert!(file_path.exists());
        assert_eq!(fs::metadata(&file_path).unwrap().len(), 1024);
    }

    #[test]
    fn test_random_file_creation() {
        let mut manager = TestDataManager::new().unwrap();
        let file_path = manager.create_random_test_file("random.dat", 2048).unwrap();
        
        assert!(file_path.exists());
        assert_eq!(fs::metadata(&file_path).unwrap().len(), 2048);
    }

    #[test]
    fn test_cleanup() {
        let mut manager = TestDataManager::new().unwrap();
        let file_path = manager.create_test_file("cleanup_test.dat", 512).unwrap();
        
        assert!(file_path.exists());
        manager.cleanup_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }
}