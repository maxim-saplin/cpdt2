//! Statistics collection and calculation for benchmark results

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Results from a single test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Low-percentile speed (P5) recorded during the test (MB/s)
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
    bytes_transferred: u64,
    sample_interval: Duration,
}

/// Real-time statistics tracker for ongoing operations
pub struct RealTimeStatsTracker {
    collector: StatisticsCollector,
    last_bytes: u64,
    last_sample_time: Instant,
}

impl StatisticsCollector {
    /// Create a new statistics collector with default 100ms sampling interval
    pub fn new() -> Self {
        Self::with_sample_interval(Duration::from_millis(100))
    }
    
    /// Create a new statistics collector with custom sampling interval
    pub fn with_sample_interval(sample_interval: Duration) -> Self {
        let now = Instant::now();
        Self {
            samples: Vec::new(),
            start_time: now,
            last_sample_time: now,
            bytes_transferred: 0,
            sample_interval,
        }
    }
    
    /// Update the total bytes transferred
    pub fn update_bytes_transferred(&mut self, bytes: u64) {
        self.bytes_transferred = bytes;
    }
    
    /// Add bytes to the total transferred
    pub fn add_bytes_transferred(&mut self, bytes: u64) {
        self.bytes_transferred += bytes;
    }
    
    /// Get current bytes transferred
    pub fn get_bytes_transferred(&self) -> u64 {
        self.bytes_transferred
    }
    
    /// Calculate current real-time speed based on bytes transferred and elapsed time
    pub fn current_speed_mbps(&self) -> f64 {
        let elapsed = self.elapsed();
        Self::calculate_speed_mbps(self.bytes_transferred, elapsed)
    }
    
    /// Check if it's time to take a sample based on the sampling interval
    pub fn should_sample(&self) -> bool {
        self.last_sample_time.elapsed() >= self.sample_interval
    }
    
    /// Take a sample if the sampling interval has elapsed
    /// Returns true if a sample was taken, false otherwise
    pub fn sample_if_ready(&mut self) -> bool {
        if self.should_sample() {
            let speed = self.current_speed_mbps();
            self.add_sample(speed);
            true
        } else {
            false
        }
    }
    
    /// Force a sample to be taken regardless of timing
    pub fn force_sample(&mut self) {
        let speed = self.current_speed_mbps();
        self.add_sample(speed);
    }
    
    /// Add a speed sample (in MB/s)
    pub fn add_sample(&mut self, speed_mbps: f64) {
        self.samples.push(speed_mbps);
        self.last_sample_time = Instant::now();
    }
    
    /// Get the number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
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
    
    /// Compute percentile using the nearest-rank method
    /// p is in [0, 100]
    fn percentile_nearest_rank(mut samples: Vec<f64>, p: f64) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let clamped_p = if p.is_finite() { p.clamp(0.0, 100.0) } else { 0.0 };
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if clamped_p == 0.0 {
            return samples[0];
        }
        if clamped_p == 100.0 {
            return samples[samples.len() - 1];
        }
        let n = samples.len();
        let rank = (clamped_p / 100.0 * n as f64).ceil() as usize;
        let idx = rank.max(1) - 1; // nearest-rank index
        samples[idx]
    }

    /// Finalize collection and return test results
    pub fn finalize(self) -> TestResult {
        if self.samples.is_empty() {
            return TestResult::default();
        }
        
        // Use P5 instead of absolute minimum to reduce sensitivity to outliers
        let min_speed = Self::percentile_nearest_rank(self.samples.clone(), 5.0);
        // Use P95 instead of absolute maximum
        let max_speed = Self::percentile_nearest_rank(self.samples.clone(), 95.0);
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

impl RealTimeStatsTracker {
    /// Create a new real-time statistics tracker
    pub fn new() -> Self {
        Self {
            collector: StatisticsCollector::new(),
            last_bytes: 0,
            last_sample_time: Instant::now(),
        }
    }
    
    /// Create a new tracker with custom sampling interval
    pub fn with_sample_interval(sample_interval: Duration) -> Self {
        Self {
            collector: StatisticsCollector::with_sample_interval(sample_interval),
            last_bytes: 0,
            last_sample_time: Instant::now(),
        }
    }
    
    /// Record a single block operation as a sample using its size and duration
    /// Returns the block speed (MB/s) if it's time to report progress, otherwise None
    pub fn record_block(&mut self, bytes: usize, duration: Duration) -> Option<f64> {
        if duration.is_zero() || bytes == 0 {
            return None;
        }
        let speed = StatisticsCollector::calculate_speed_mbps(bytes as u64, duration);
        self.collector.add_sample(speed);
        if self.last_sample_time.elapsed() >= self.collector.sample_interval {
            self.last_sample_time = Instant::now();
            Some(speed)
        } else {
            None
        }
    }
    
    /// Update progress and automatically sample if interval has elapsed
    /// Returns the current instantaneous speed if a sample was taken
    pub fn update_progress(&mut self, total_bytes: u64) -> Option<f64> {
        self.collector.update_bytes_transferred(total_bytes);
        
        if self.collector.should_sample() {
            // Calculate instantaneous speed since last sample
            let bytes_since_last = total_bytes.saturating_sub(self.last_bytes);
            let time_since_last = self.last_sample_time.elapsed();
            let instantaneous_speed = StatisticsCollector::calculate_speed_mbps(bytes_since_last, time_since_last);
            
            // Add the instantaneous speed as a sample
            self.collector.add_sample(instantaneous_speed);
            
            // Update tracking variables
            self.last_bytes = total_bytes;
            self.last_sample_time = Instant::now();
            
            Some(instantaneous_speed)
        } else {
            None
        }
    }
    
    /// Get the current overall average speed
    pub fn current_average_speed(&self) -> f64 {
        self.collector.current_speed_mbps()
    }
    
    /// Get the current instantaneous speed (since last sample)
    pub fn current_instantaneous_speed(&self) -> f64 {
        let bytes_since_last = self.collector.get_bytes_transferred().saturating_sub(self.last_bytes);
        let time_since_last = self.last_sample_time.elapsed();
        StatisticsCollector::calculate_speed_mbps(bytes_since_last, time_since_last)
    }
    
    /// Get current statistics
    pub fn current_stats(&self) -> (f64, f64, f64, usize) {
        (
            self.collector.current_min(),
            self.collector.current_max(),
            self.collector.current_average(),
            self.collector.sample_count(),
        )
    }
    
    /// Finalize and get test results
    pub fn finalize(self) -> TestResult {
        // Do not force a synthetic final sample; rely on recorded block samples
        self.collector.finalize()
    }
}

impl Default for RealTimeStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_statistics_collector_basic() {
        let mut collector = StatisticsCollector::new();
        
        // Add some samples
        collector.add_sample(100.0);
        collector.add_sample(200.0);
        collector.add_sample(150.0);
        
        assert_eq!(collector.current_min(), 100.0);
        assert_eq!(collector.current_max(), 200.0);
        assert_eq!(collector.current_average(), 150.0);
        assert_eq!(collector.sample_count(), 3);
        
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
        
        // Test with fractional seconds
        let speed = StatisticsCollector::calculate_speed_mbps(
            1024 * 1024, 
            Duration::from_millis(500)
        );
        assert!((speed - 2.0).abs() < 0.001);
        
        // Test zero duration edge case
        let speed = StatisticsCollector::calculate_speed_mbps(
            1024 * 1024, 
            Duration::from_nanos(0)
        );
        assert_eq!(speed, 0.0);
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
    
    #[test]
    fn test_single_sample() {
        let mut collector = StatisticsCollector::new();
        collector.add_sample(42.5);
        
        assert_eq!(collector.current_min(), 42.5);
        assert_eq!(collector.current_max(), 42.5);
        assert_eq!(collector.current_average(), 42.5);
        assert_eq!(collector.sample_count(), 1);
    }
    
    #[test]
    fn test_bytes_transferred_tracking() {
        let mut collector = StatisticsCollector::new();
        
        // Test initial state
        assert_eq!(collector.get_bytes_transferred(), 0);
        
        // Test updating bytes
        collector.update_bytes_transferred(1024);
        assert_eq!(collector.get_bytes_transferred(), 1024);
        
        // Test adding bytes
        collector.add_bytes_transferred(512);
        assert_eq!(collector.get_bytes_transferred(), 1536);
    }
    
    #[test]
    fn test_current_speed_calculation() {
        let mut collector = StatisticsCollector::new();
        
        // Initially should be 0
        assert_eq!(collector.current_speed_mbps(), 0.0);
        
        // Add some bytes and check speed calculation
        collector.update_bytes_transferred(1024 * 1024); // 1MB
        
        // Sleep a bit to ensure elapsed time > 0
        thread::sleep(Duration::from_millis(10));
        
        let speed = collector.current_speed_mbps();
        assert!(speed > 0.0);
        assert!(speed < 1000.0); // Should be reasonable
    }
    
    #[test]
    fn test_sampling_interval() {
        let mut collector = StatisticsCollector::with_sample_interval(Duration::from_millis(50));
        
        // Initially should not be ready to sample
        assert!(!collector.should_sample());
        
        // After sleeping, should be ready
        thread::sleep(Duration::from_millis(60));
        assert!(collector.should_sample());
        
        // After sampling, should reset
        collector.sample_if_ready();
        assert!(!collector.should_sample());
    }
    
    #[test]
    fn test_sample_if_ready() {
        let mut collector = StatisticsCollector::with_sample_interval(Duration::from_millis(10));
        collector.update_bytes_transferred(1024 * 1024);
        
        // Should not sample immediately
        assert!(!collector.sample_if_ready());
        assert_eq!(collector.sample_count(), 0);
        
        // After waiting, should sample
        thread::sleep(Duration::from_millis(15));
        assert!(collector.sample_if_ready());
        assert_eq!(collector.sample_count(), 1);
    }
    
    #[test]
    fn test_force_sample() {
        let mut collector = StatisticsCollector::new();
        collector.update_bytes_transferred(1024 * 1024);
        
        // Force sample should work regardless of timing
        collector.force_sample();
        assert_eq!(collector.sample_count(), 1);
        
        // Should be able to force another immediately
        collector.force_sample();
        assert_eq!(collector.sample_count(), 2);
    }
    
    #[test]
    fn test_real_time_stats_tracker() {
        let mut tracker = RealTimeStatsTracker::with_sample_interval(Duration::from_millis(10));
        
        // Initial state
        assert_eq!(tracker.current_average_speed(), 0.0);
        let (_min, _max, _avg, count) = tracker.current_stats();
        assert_eq!(count, 0);
        
        // Update progress - should not sample immediately
        let sample_taken = tracker.update_progress(1024 * 1024);
        assert!(sample_taken.is_none());
        
        // After waiting, should sample
        thread::sleep(Duration::from_millis(15));
        let sample_taken = tracker.update_progress(2 * 1024 * 1024);
        assert!(sample_taken.is_some());
        
        let (_min, _max, _avg, count) = tracker.current_stats();
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_instantaneous_speed_calculation() {
        let mut tracker = RealTimeStatsTracker::with_sample_interval(Duration::from_millis(10));
        
        // Set initial bytes
        tracker.update_progress(0);
        thread::sleep(Duration::from_millis(15));
        
        // Add 1MB over the sampling period
        let sample_speed = tracker.update_progress(1024 * 1024);
        assert!(sample_speed.is_some());
        
        let speed = sample_speed.unwrap();
        assert!(speed > 0.0);
        assert!(speed < 10000.0); // Should be reasonable
    }
    
    #[test]
    fn test_edge_case_zero_bytes() {
        let mut collector = StatisticsCollector::new();
        
        // Test with zero bytes
        collector.update_bytes_transferred(0);
        thread::sleep(Duration::from_millis(10));
        
        let speed = collector.current_speed_mbps();
        assert_eq!(speed, 0.0);
    }
    
    #[test]
    fn test_edge_case_large_numbers() {
        let mut collector = StatisticsCollector::new();
        
        // Test with very large byte counts
        let large_bytes = u64::MAX / 2;
        collector.update_bytes_transferred(large_bytes);
        thread::sleep(Duration::from_millis(10));
        
        let speed = collector.current_speed_mbps();
        assert!(speed > 0.0);
        assert!(speed.is_finite());
    }
    
    #[test]
    fn test_statistics_accuracy() {
        let mut collector = StatisticsCollector::new();
        
        // Add known samples
        let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for sample in &samples {
            collector.add_sample(*sample);
        }
        
        // Verify statistics
        assert_eq!(collector.current_min(), 10.0);
        assert_eq!(collector.current_max(), 50.0);
        assert_eq!(collector.current_average(), 30.0); // (10+20+30+40+50)/5 = 30
        assert_eq!(collector.sample_count(), 5);
        
        let result = collector.finalize();
        assert_eq!(result.min_speed_mbps, 10.0);
        assert_eq!(result.max_speed_mbps, 50.0);
        assert_eq!(result.avg_speed_mbps, 30.0);
        assert_eq!(result.sample_count, 5);
    }
    
    #[test]
    fn test_negative_and_zero_samples() {
        let mut collector = StatisticsCollector::new();
        
        // Add samples including zero and negative (edge case)
        collector.add_sample(0.0);
        collector.add_sample(-5.0); // Shouldn't happen in practice but test robustness
        collector.add_sample(10.0);
        
        assert_eq!(collector.current_min(), -5.0);
        assert_eq!(collector.current_max(), 10.0);
        assert!((collector.current_average() - 5.0/3.0).abs() < 0.001);
    }
}