//! Test data generation and management utilities

use anyhow::Result;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

/// Patterns for test data generation
#[derive(Debug, Clone)]
pub enum TestDataPattern {
    /// All zeros
    Zeros,
    /// All ones (0xFF)
    Ones,
    /// Random data with fixed seed for reproducibility
    RandomSeeded(u64),
    /// Truly random data
    Random,
    /// Alternating pattern (0xAA, 0x55, ...)
    Alternating,
    /// Sequential bytes (0x00, 0x01, 0x02, ...)
    Sequential,
}

/// Test data generator for creating controlled test files
pub struct TestDataGenerator {
    pattern: TestDataPattern,
    rng: Option<StdRng>,
}

impl TestDataGenerator {
    /// Create a new test data generator with specified pattern
    pub fn new(pattern: TestDataPattern) -> Self {
        let rng = match pattern {
            TestDataPattern::RandomSeeded(seed) => Some(StdRng::seed_from_u64(seed)),
            TestDataPattern::Random => Some(StdRng::from_entropy()),
            _ => None,
        };

        Self { pattern, rng }
    }

    /// Generate test data and write to file
    pub fn generate_file(&mut self, file_path: &Path, size_bytes: u64) -> Result<()> {
        let mut file = File::create(file_path)?;
        self.write_data(&mut file, size_bytes)?;
        file.sync_all()?;
        Ok(())
    }

    /// Write test data to an existing file handle
    pub fn write_data(&mut self, file: &mut File, size_bytes: u64) -> Result<()> {
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
        let mut remaining = size_bytes;
        let mut byte_counter = 0u8;

        while remaining > 0 {
            let current_chunk_size = std::cmp::min(CHUNK_SIZE, remaining as usize);
            let mut chunk = vec![0u8; current_chunk_size];

            match &self.pattern {
                TestDataPattern::Zeros => {
                    // chunk is already filled with zeros
                }
                TestDataPattern::Ones => {
                    chunk.fill(0xFF);
                }
                TestDataPattern::RandomSeeded(_) | TestDataPattern::Random => {
                    if let Some(ref mut rng) = self.rng {
                        rng.fill_bytes(&mut chunk);
                    }
                }
                TestDataPattern::Alternating => {
                    for (i, byte) in chunk.iter_mut().enumerate() {
                        *byte = if i % 2 == 0 { 0xAA } else { 0x55 };
                    }
                }
                TestDataPattern::Sequential => {
                    for byte in chunk.iter_mut() {
                        *byte = byte_counter;
                        byte_counter = byte_counter.wrapping_add(1);
                    }
                }
            }

            file.write_all(&chunk)?;
            remaining -= current_chunk_size as u64;
        }

        Ok(())
    }

    /// Create a sparse file (file with holes)
    pub fn create_sparse_file(&self, file_path: &Path, size_bytes: u64) -> Result<()> {
        let file = File::create(file_path)?;
        file.set_len(size_bytes)?;
        Ok(())
    }

    /// Create a file with specific data at specific offsets
    pub fn create_pattern_file(
        &mut self,
        file_path: &Path,
        total_size: u64,
        data_chunks: &[(u64, u64)],
    ) -> Result<()> {
        let mut file = File::create(file_path)?;
        file.set_len(total_size)?;

        for &(offset, size) in data_chunks {
            file.seek(SeekFrom::Start(offset))?;
            self.write_data(&mut file, size)?;
        }

        file.sync_all()?;
        Ok(())
    }
}

/// Verify test data integrity
pub struct TestDataVerifier {
    pattern: TestDataPattern,
}

impl TestDataVerifier {
    pub fn new(pattern: TestDataPattern) -> Self {
        Self { pattern }
    }

    /// Verify that file contains expected pattern
    pub fn verify_file(&self, file_path: &Path) -> Result<bool> {
        use std::io::Read;

        let mut file = File::open(file_path)?;
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
        let mut byte_counter = 0u8;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            for &byte in &buffer[..bytes_read] {
                let expected = match &self.pattern {
                    TestDataPattern::Zeros => 0x00,
                    TestDataPattern::Ones => 0xFF,
                    TestDataPattern::Alternating => {
                        if byte_counter % 2 == 0 {
                            0xAA
                        } else {
                            0x55
                        }
                    }
                    TestDataPattern::Sequential => byte_counter,
                    TestDataPattern::RandomSeeded(_) | TestDataPattern::Random => {
                        // Can't verify random data without regenerating
                        byte_counter = byte_counter.wrapping_add(1);
                        continue;
                    }
                };

                if byte != expected {
                    return Ok(false);
                }
                byte_counter = byte_counter.wrapping_add(1);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_zeros_pattern() {
        let mut generator = TestDataGenerator::new(TestDataPattern::Zeros);
        let temp_file = NamedTempFile::new().unwrap();

        generator.generate_file(temp_file.path(), 1024).unwrap();

        let verifier = TestDataVerifier::new(TestDataPattern::Zeros);
        assert!(verifier.verify_file(temp_file.path()).unwrap());
    }

    #[test]
    fn test_ones_pattern() {
        let mut generator = TestDataGenerator::new(TestDataPattern::Ones);
        let temp_file = NamedTempFile::new().unwrap();

        generator.generate_file(temp_file.path(), 1024).unwrap();

        let verifier = TestDataVerifier::new(TestDataPattern::Ones);
        assert!(verifier.verify_file(temp_file.path()).unwrap());
    }

    #[test]
    fn test_sequential_pattern() {
        let mut generator = TestDataGenerator::new(TestDataPattern::Sequential);
        let temp_file = NamedTempFile::new().unwrap();

        generator.generate_file(temp_file.path(), 512).unwrap();

        let verifier = TestDataVerifier::new(TestDataPattern::Sequential);
        assert!(verifier.verify_file(temp_file.path()).unwrap());
    }

    #[test]
    fn test_seeded_random_reproducibility() {
        let mut generator1 = TestDataGenerator::new(TestDataPattern::RandomSeeded(12345));
        let mut generator2 = TestDataGenerator::new(TestDataPattern::RandomSeeded(12345));

        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();

        generator1.generate_file(temp_file1.path(), 1024).unwrap();
        generator2.generate_file(temp_file2.path(), 1024).unwrap();

        let data1 = std::fs::read(temp_file1.path()).unwrap();
        let data2 = std::fs::read(temp_file2.path()).unwrap();

        assert_eq!(data1, data2);
    }

    #[test]
    fn test_sparse_file_creation() {
        let generator = TestDataGenerator::new(TestDataPattern::Zeros);
        let temp_file = NamedTempFile::new().unwrap();

        generator
            .create_sparse_file(temp_file.path(), 1024 * 1024)
            .unwrap();

        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert_eq!(metadata.len(), 1024 * 1024);
    }
}
