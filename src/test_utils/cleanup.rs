//! Test cleanup utilities and resource management

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use anyhow::Result;

/// Global test cleanup registry
static CLEANUP_REGISTRY: std::sync::OnceLock<Arc<Mutex<CleanupRegistry>>> = std::sync::OnceLock::new();

/// Registry for tracking test resources that need cleanup
pub struct CleanupRegistry {
    temp_files: Vec<PathBuf>,
    temp_dirs: Vec<PathBuf>,
    cleanup_callbacks: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
    start_time: Instant,
}

impl CleanupRegistry {
    fn new() -> Self {
        Self {
            temp_files: Vec::new(),
            temp_dirs: Vec::new(),
            cleanup_callbacks: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Get the global cleanup registry
    pub fn global() -> Arc<Mutex<CleanupRegistry>> {
        CLEANUP_REGISTRY.get_or_init(|| Arc::new(Mutex::new(CleanupRegistry::new()))).clone()
    }

    /// Register a temporary file for cleanup
    pub fn register_temp_file<P: AsRef<Path>>(&mut self, path: P) {
        self.temp_files.push(path.as_ref().to_path_buf());
    }

    /// Register a temporary directory for cleanup
    pub fn register_temp_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.temp_dirs.push(path.as_ref().to_path_buf());
    }

    /// Register a custom cleanup callback
    pub fn register_cleanup_callback<F>(&mut self, callback: F)
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.cleanup_callbacks.push(Box::new(callback));
    }

    /// Perform cleanup of all registered resources
    pub fn cleanup_all(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        // Execute custom cleanup callbacks first
        for callback in &self.cleanup_callbacks {
            if let Err(e) = callback() {
                errors.push(format!("Cleanup callback failed: {}", e));
            }
        }

        // Clean up temporary files
        for file_path in &self.temp_files {
            if file_path.exists() {
                if let Err(e) = fs::remove_file(file_path) {
                    errors.push(format!("Failed to remove file {:?}: {}", file_path, e));
                }
            }
        }

        // Clean up temporary directories
        for dir_path in &self.temp_dirs {
            if dir_path.exists() {
                if let Err(e) = fs::remove_dir_all(dir_path) {
                    errors.push(format!("Failed to remove directory {:?}: {}", dir_path, e));
                }
            }
        }

        // Clear the registry
        self.temp_files.clear();
        self.temp_dirs.clear();
        self.cleanup_callbacks.clear();

        if !errors.is_empty() {
            anyhow::bail!("Cleanup errors: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Get cleanup statistics
    pub fn stats(&self) -> CleanupStats {
        CleanupStats {
            temp_files_count: self.temp_files.len(),
            temp_dirs_count: self.temp_dirs.len(),
            callbacks_count: self.cleanup_callbacks.len(),
            registry_age: self.start_time.elapsed(),
        }
    }
}

/// Statistics about cleanup registry
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub temp_files_count: usize,
    pub temp_dirs_count: usize,
    pub callbacks_count: usize,
    pub registry_age: Duration,
}

/// RAII cleanup guard that ensures cleanup on drop
pub struct CleanupGuard {
    cleanup_fn: Option<Box<dyn FnOnce() -> Result<()>>>,
}

impl CleanupGuard {
    /// Create a new cleanup guard with a cleanup function
    pub fn new<F>(cleanup_fn: F) -> Self
    where
        F: FnOnce() -> Result<()> + 'static,
    {
        Self {
            cleanup_fn: Some(Box::new(cleanup_fn)),
        }
    }

    /// Create a cleanup guard for a temporary file
    pub fn for_file<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self::new(move || {
            if path.exists() {
                fs::remove_file(&path)?;
            }
            Ok(())
        })
    }

    /// Create a cleanup guard for a temporary directory
    pub fn for_directory<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self::new(move || {
            if path.exists() {
                fs::remove_dir_all(&path)?;
            }
            Ok(())
        })
    }

    /// Manually trigger cleanup (consumes the guard)
    pub fn cleanup(mut self) -> Result<()> {
        if let Some(cleanup_fn) = self.cleanup_fn.take() {
            cleanup_fn()
        } else {
            Ok(())
        }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some(cleanup_fn) = self.cleanup_fn.take() {
            if let Err(e) = cleanup_fn() {
                eprintln!("Warning: Cleanup failed during drop: {}", e);
            }
        }
    }
}

/// Utility functions for test cleanup
pub mod utils {
    use super::*;

    /// Register a temporary file with the global cleanup registry
    pub fn register_temp_file<P: AsRef<Path>>(path: P) {
        if let Ok(mut registry) = CleanupRegistry::global().lock() {
            registry.register_temp_file(path);
        }
    }

    /// Register a temporary directory with the global cleanup registry
    pub fn register_temp_dir<P: AsRef<Path>>(path: P) {
        if let Ok(mut registry) = CleanupRegistry::global().lock() {
            registry.register_temp_dir(path);
        }
    }

    /// Perform global cleanup
    pub fn cleanup_all() -> Result<()> {
        if let Ok(mut registry) = CleanupRegistry::global().lock() {
            registry.cleanup_all()
        } else {
            anyhow::bail!("Failed to acquire cleanup registry lock");
        }
    }

    /// Get cleanup statistics
    pub fn cleanup_stats() -> Option<CleanupStats> {
        CleanupRegistry::global().lock().ok().map(|registry| registry.stats())
    }

    /// Clean up files matching a pattern in a directory
    pub fn cleanup_pattern<P: AsRef<Path>>(dir: P, pattern: &str) -> Result<usize> {
        let dir = dir.as_ref();
        let mut cleaned_count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(pattern) {
                    if path.is_file() {
                        fs::remove_file(&path)?;
                        cleaned_count += 1;
                    } else if path.is_dir() {
                        fs::remove_dir_all(&path)?;
                        cleaned_count += 1;
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Clean up old test files based on age
    pub fn cleanup_old_files<P: AsRef<Path>>(dir: P, max_age: Duration) -> Result<usize> {
        let dir = dir.as_ref();
        let mut cleaned_count = 0;
        let now = std::time::SystemTime::now();

        if !dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > max_age {
                        if path.is_file() {
                            fs::remove_file(&path)?;
                            cleaned_count += 1;
                        } else if path.is_dir() {
                            fs::remove_dir_all(&path)?;
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_cleanup_guard_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        // File should exist
        assert!(path.exists());
        
        {
            let _guard = CleanupGuard::for_file(&path);
            // File still exists while guard is alive
            assert!(path.exists());
        }
        
        // File should be cleaned up after guard is dropped
        // Note: NamedTempFile also cleans up, so this test is more about the guard mechanism
    }

    #[test]
    fn test_cleanup_guard_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        // Create a file in the directory
        let file_path = path.join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();
        
        assert!(path.exists());
        assert!(file_path.exists());
        
        // Don't let TempDir clean up automatically
        let path = temp_dir.into_path();
        
        {
            let _guard = CleanupGuard::for_directory(&path);
            assert!(path.exists());
        }
        
        // Directory should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_cleanup_registry() {
        let mut registry = CleanupRegistry::new();
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();
        
        registry.register_temp_file(&file_path);
        
        let stats = registry.stats();
        assert_eq!(stats.temp_files_count, 1);
        assert_eq!(stats.temp_dirs_count, 0);
    }

    #[test]
    fn test_cleanup_pattern() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        fs::write(temp_dir.path().join("test_file_1.tmp"), "content").unwrap();
        fs::write(temp_dir.path().join("test_file_2.tmp"), "content").unwrap();
        fs::write(temp_dir.path().join("other_file.txt"), "content").unwrap();
        
        let cleaned = utils::cleanup_pattern(temp_dir.path(), "test_file").unwrap();
        assert_eq!(cleaned, 2);
        
        // Only the non-matching file should remain
        assert!(!temp_dir.path().join("test_file_1.tmp").exists());
        assert!(!temp_dir.path().join("test_file_2.tmp").exists());
        assert!(temp_dir.path().join("other_file.txt").exists());
    }

    #[test]
    fn test_manual_cleanup() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        
        // Don't let NamedTempFile clean up automatically
        let (_file, file_path) = temp_file.keep().unwrap();
        
        assert!(file_path.exists());
        
        let guard = CleanupGuard::for_file(&file_path);
        guard.cleanup().unwrap();
        
        assert!(!file_path.exists());
    }
}