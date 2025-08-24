//! Statistics collection and calculation for benchmark tests

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Result from a single test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Minimum speed recorded during the test (MB/s)
    pub min_speed_mbps: f64,
    
    /// Maximum speed recorded during the test (MB/s)
    pub max_speed_mbps: f64,
    
    /// Average speed across all samples (MB/s)
    pub avg_speed_mbps: f64,
    
    /// Total duration of the test
    pub test_duration: Duration,
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            min_speed_mbps: 0.0,
            max_speed_mbps: 0.0,
            avg_speed_mbps: 0.0,
            test_duration: Duration::from_secs(0),
        }
    }
}

/// Statistics collector for gathering performance samples during tests
#[derive(Debug)]
pub struct StatsCollector {
    samples: Vec<f64>,
    start_time: std::time::Instant,
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Add a speed sample (in MB/s)
    pub fn add_sample(&mut self, speed_mbps: f64) {
        self.samples.push(speed_mbps);
    }
    
    /// Calculate final test result from collected samples
    pub fn finalize(self) -> TestResult {
        if self.samples.is_empty() {
            return TestResult::default();
        }
        
        let min_speed = self.samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_speed = self.samples.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_speed = self.samples.iter().sum::<f64>() / self.samples.len() as f64;
        
        TestResult {
            min_speed_mbps: min_speed,
            max_speed_mbps: max_speed,
            avg_speed_mbps: avg_speed,
            test_duration: self.start_time.elapsed(),
        }
    }
    
    /// Get the current average speed
    pub fn current_avg(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }
    
    /// Get the number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_collector_empty() {
        let collector = StatsCollector::new();
        let result = collector.finalize();
        assert_eq!(result.min_speed_mbps, 0.0);
        assert_eq!(result.max_speed_mbps, 0.0);
        assert_eq!(result.avg_speed_mbps, 0.0);
    }
    
    #[test]
    fn test_stats_collector_with_samples() {
        let mut collector = StatsCollector::new();
        collector.add_sample(100.0);
        collector.add_sample(200.0);
        collector.add_sample(150.0);
        
        assert_eq!(collector.current_avg(), 150.0);
        assert_eq!(collector.sample_count(), 3);
        
        let result = collector.finalize();
        assert_eq!(result.min_speed_mbps, 100.0);
        assert_eq!(result.max_speed_mbps, 200.0);
        assert_eq!(result.avg_speed_mbps, 150.0);
    }
    
    #[test]
    fn test_stats_collector_single_sample() {
        let mut collector = StatsCollector::new();
        collector.add_sample(42.5);
        
        let result = collector.finalize();
        assert_eq!(result.min_speed_mbps, 42.5);
        assert_eq!(result.max_speed_mbps, 42.5);
        assert_eq!(result.avg_speed_mbps, 42.5);
    }
}