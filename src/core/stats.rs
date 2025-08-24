//! Statistics collection and calculation for benchmark results

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Results from a single test execution
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
    
    /// Number of samples collected
    pub sample_count: usize,
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            min_speed_mbps: 0.0,
            max_speed_mbps: 0.0,
            avg_speed_mbps: 0.0,
            test_duration: Duration::from_secs(0),
            sample_count: 0,
        }
    }
}

impl TestResult {
    /// Create a new test result with the given values
    pub fn new(
        min_speed_mbps: f64,
        max_speed_mbps: f64,
        avg_speed_mbps: f64,
        test_duration: Duration,
        sample_count: usize,
    ) -> Self {
        Self {
            min_speed_mbps,
            max_speed_mbps,
            avg_speed_mbps,
            test_duration,
            sample_count,
        }
    }
}

/// Collects performance statistics during test execution
pub struct StatisticsCollector {
    samples: Vec<f64>,
    start_time: Instant,
    last_sample_time: Instant,
}

impl StatisticsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            samples: Vec::new(),
            start_time: now,
            last_sample_time: now,
        }
    }
    
    /// Add a speed sample (in MB/s)
    pub fn add_sample(&mut self, speed_mbps: f64) {
        self.samples.push(speed_mbps);
        self.last_sample_time = Instant::now();
    }
    
    /// Get the current average speed
    pub fn current_average(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }
    
    /// Get the current minimum speed
    pub fn current_min(&self) -> f64 {
        self.samples.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
    
    /// Get the current maximum speed
    pub fn current_max(&self) -> f64 {
        self.samples.iter().fold(0.0, |a, &b| a.max(b))
    }
    
    /// Get the elapsed time since collection started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Finalize collection and return test results
    pub fn finalize(self) -> TestResult {
        if self.samples.is_empty() {
            return TestResult::default();
        }
        
        let min_speed = self.samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_speed = self.samples.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg_speed = self.samples.iter().sum::<f64>() / self.samples.len() as f64;
        
        TestResult::new(
            min_speed,
            max_speed,
            avg_speed,
            self.elapsed(),
            self.samples.len(),
        )
    }
    
    /// Calculate speed in MB/s from bytes and duration
    pub fn calculate_speed_mbps(bytes: u64, duration: Duration) -> f64 {
        if duration.is_zero() {
            return 0.0;
        }
        
        let seconds = duration.as_secs_f64();
        let megabytes = bytes as f64 / (1024.0 * 1024.0);
        megabytes / seconds
    }
}

impl Default for StatisticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_statistics_collector() {
        let mut collector = StatisticsCollector::new();
        
        // Add some samples
        collector.add_sample(100.0);
        collector.add_sample(200.0);
        collector.add_sample(150.0);
        
        assert_eq!(collector.current_min(), 100.0);
        assert_eq!(collector.current_max(), 200.0);
        assert_eq!(collector.current_average(), 150.0);
        
        let result = collector.finalize();
        assert_eq!(result.min_speed_mbps, 100.0);
        assert_eq!(result.max_speed_mbps, 200.0);
        assert_eq!(result.avg_speed_mbps, 150.0);
        assert_eq!(result.sample_count, 3);
    }
    
    #[test]
    fn test_calculate_speed_mbps() {
        // 1MB in 1 second = 1 MB/s
        let speed = StatisticsCollector::calculate_speed_mbps(
            1024 * 1024, 
            Duration::from_secs(1)
        );
        assert!((speed - 1.0).abs() < 0.001);
        
        // 2MB in 1 second = 2 MB/s
        let speed = StatisticsCollector::calculate_speed_mbps(
            2 * 1024 * 1024, 
            Duration::from_secs(1)
        );
        assert!((speed - 2.0).abs() < 0.001);
    }
    
    #[test]
    fn test_empty_collector() {
        let collector = StatisticsCollector::new();
        let result = collector.finalize();
        
        assert_eq!(result.min_speed_mbps, 0.0);
        assert_eq!(result.max_speed_mbps, 0.0);
        assert_eq!(result.avg_speed_mbps, 0.0);
        assert_eq!(result.sample_count, 0);
    }
}