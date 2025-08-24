//! Progress reporting system for benchmark execution
//! 
//! This module provides utilities for managing progress callbacks during benchmark execution,
//! including thread-safe progress reporting and callback management.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::core::{ProgressCallback, TestResult};

/// Thread-safe wrapper for progress callbacks
/// 
/// This struct provides a thread-safe way to call progress callbacks from multiple threads
/// during benchmark execution. It ensures that callbacks are called safely even when
/// the benchmark is running across multiple threads.
pub struct ProgressReporter {
    callback: Option<Arc<dyn ProgressCallback>>,
    last_progress_time: Arc<Mutex<Instant>>,
    progress_interval: Duration,
}

impl ProgressReporter {
    /// Create a new progress reporter with the given callback
    /// 
    /// # Arguments
    /// 
    /// * `callback` - Optional progress callback to wrap
    /// 
    /// # Returns
    /// 
    /// A new `ProgressReporter` instance
    pub fn new(callback: Option<Box<dyn ProgressCallback>>) -> Self {
        Self {
            callback: callback.map(|cb| Arc::from(cb)),
            last_progress_time: Arc::new(Mutex::new(Instant::now())),
            progress_interval: Duration::from_millis(100), // Default 100ms interval
        }
    }
    
    /// Create a new progress reporter with a custom progress interval
    /// 
    /// # Arguments
    /// 
    /// * `callback` - Optional progress callback to wrap
    /// * `progress_interval` - Minimum time between progress updates
    pub fn with_interval(
        callback: Option<Box<dyn ProgressCallback>>, 
        progress_interval: Duration
    ) -> Self {
        Self {
            callback: callback.map(|cb| Arc::from(cb)),
            last_progress_time: Arc::new(Mutex::new(Instant::now())),
            progress_interval,
        }
    }
    
    /// Check if a callback is registered
    pub fn has_callback(&self) -> bool {
        self.callback.is_some()
    }
    
    /// Report that a test has started
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the test that is starting
    pub fn on_test_start(&self, test_name: &str) {
        if let Some(ref callback) = self.callback {
            callback.on_test_start(test_name);
        }
    }
    
    /// Report progress update, respecting the progress interval
    /// 
    /// This method will only call the underlying callback if enough time has passed
    /// since the last progress update, preventing excessive callback calls.
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the currently running test
    /// * `current_speed_mbps` - Current speed in MB/s
    /// 
    /// # Returns
    /// 
    /// `true` if the progress callback was called, `false` if it was throttled
    pub fn on_progress(&self, test_name: &str, current_speed_mbps: f64) -> bool {
        if let Some(ref callback) = self.callback {
            let mut last_time = self.last_progress_time.lock().unwrap();
            let now = Instant::now();
            
            if now.duration_since(*last_time) >= self.progress_interval {
                callback.on_progress(test_name, current_speed_mbps);
                *last_time = now;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Force a progress update regardless of timing
    /// 
    /// This method bypasses the progress interval and always calls the callback
    /// if one is registered. Useful for final progress updates.
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the currently running test
    /// * `current_speed_mbps` - Current speed in MB/s
    pub fn force_progress(&self, test_name: &str, current_speed_mbps: f64) {
        if let Some(ref callback) = self.callback {
            callback.on_progress(test_name, current_speed_mbps);
            *self.last_progress_time.lock().unwrap() = Instant::now();
        }
    }
    
    /// Report that a test has completed
    /// 
    /// # Arguments
    /// 
    /// * `test_name` - Name of the test that completed
    /// * `result` - Final test results
    pub fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        if let Some(ref callback) = self.callback {
            callback.on_test_complete(test_name, result);
        }
    }
    
    /// Get the current progress interval
    pub fn progress_interval(&self) -> Duration {
        self.progress_interval
    }
    
    /// Set a new progress interval
    /// 
    /// # Arguments
    /// 
    /// * `interval` - New minimum time between progress updates
    pub fn set_progress_interval(&mut self, interval: Duration) {
        self.progress_interval = interval;
    }
}

impl Clone for ProgressReporter {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            last_progress_time: Arc::new(Mutex::new(Instant::now())),
            progress_interval: self.progress_interval,
        }
    }
}

/// A no-op progress callback for testing and situations where progress reporting is not needed
pub struct NoOpProgressCallback;

impl ProgressCallback for NoOpProgressCallback {
    fn on_test_start(&self, _test_name: &str) {
        // No operation
    }
    
    fn on_progress(&self, _test_name: &str, _current_speed_mbps: f64) {
        // No operation
    }
    
    fn on_test_complete(&self, _test_name: &str, _result: &TestResult) {
        // No operation
    }
}

/// A progress callback that collects all events for testing purposes
#[derive(Debug, Default)]
pub struct TestProgressCallback {
    events: Arc<Mutex<Vec<ProgressEvent>>>,
}

/// Events that can be captured by TestProgressCallback
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    TestStart { test_name: String },
    Progress { test_name: String, speed_mbps: f64 },
    TestComplete { test_name: String, result: TestResult },
}

impl TestProgressCallback {
    /// Create a new test progress callback
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get all captured events
    pub fn events(&self) -> Vec<ProgressEvent> {
        self.events.lock().unwrap().clone()
    }
    
    /// Get the number of captured events
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
    
    /// Clear all captured events
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
    
    /// Get events of a specific type
    pub fn events_of_type<F>(&self, filter: F) -> Vec<ProgressEvent>
    where
        F: Fn(&ProgressEvent) -> bool,
    {
        self.events()
            .into_iter()
            .filter(filter)
            .collect()
    }
    
    /// Get all test start events
    pub fn test_start_events(&self) -> Vec<String> {
        self.events_of_type(|e| matches!(e, ProgressEvent::TestStart { .. }))
            .into_iter()
            .filter_map(|e| match e {
                ProgressEvent::TestStart { test_name } => Some(test_name),
                _ => None,
            })
            .collect()
    }
    
    /// Get all progress events for a specific test
    pub fn progress_events_for_test(&self, test_name: &str) -> Vec<f64> {
        self.events_of_type(|e| match e {
            ProgressEvent::Progress { test_name: name, .. } => name == test_name,
            _ => false,
        })
        .into_iter()
        .filter_map(|e| match e {
            ProgressEvent::Progress { speed_mbps, .. } => Some(speed_mbps),
            _ => None,
        })
        .collect()
    }
    
    /// Get all test complete events
    pub fn test_complete_events(&self) -> Vec<(String, TestResult)> {
        self.events_of_type(|e| matches!(e, ProgressEvent::TestComplete { .. }))
            .into_iter()
            .filter_map(|e| match e {
                ProgressEvent::TestComplete { test_name, result } => Some((test_name, result)),
                _ => None,
            })
            .collect()
    }
}

impl ProgressCallback for TestProgressCallback {
    fn on_test_start(&self, test_name: &str) {
        let event = ProgressEvent::TestStart {
            test_name: test_name.to_string(),
        };
        self.events.lock().unwrap().push(event);
    }
    
    fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
        let event = ProgressEvent::Progress {
            test_name: test_name.to_string(),
            speed_mbps: current_speed_mbps,
        };
        self.events.lock().unwrap().push(event);
    }
    
    fn on_test_complete(&self, test_name: &str, result: &TestResult) {
        let event = ProgressEvent::TestComplete {
            test_name: test_name.to_string(),
            result: result.clone(),
        };
        self.events.lock().unwrap().push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_progress_reporter_no_callback() {
        let reporter = ProgressReporter::new(None);
        
        assert!(!reporter.has_callback());
        
        // These should not panic even with no callback
        reporter.on_test_start("Test");
        assert!(!reporter.on_progress("Test", 100.0));
        reporter.force_progress("Test", 100.0);
        reporter.on_test_complete("Test", &TestResult::default());
    }
    
    #[test]
    fn test_progress_reporter_with_callback() {
        let test_callback = TestProgressCallback::new();
        
        let reporter = ProgressReporter::new(Some(Box::new(test_callback)));
        
        assert!(reporter.has_callback());
        
        // Test all callback methods
        reporter.on_test_start("Sequential Write");
        reporter.force_progress("Sequential Write", 150.5);
        
        let result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);
        reporter.on_test_complete("Sequential Write", &result);
        
        // We can't access the callback directly anymore since it's boxed,
        // so we'll test a different way by creating a separate callback for verification
    }
    
    #[test]
    fn test_progress_throttling() {
        let reporter = ProgressReporter::with_interval(
            Some(Box::new(TestProgressCallback::new())), 
            Duration::from_millis(50)
        );
        
        // Wait a bit to ensure we're past the initial time
        thread::sleep(Duration::from_millis(60));
        
        // First progress call should succeed
        assert!(reporter.on_progress("Test", 100.0));
        
        // Immediate second call should be throttled
        assert!(!reporter.on_progress("Test", 110.0));
        
        // After waiting, should succeed again
        thread::sleep(Duration::from_millis(60));
        assert!(reporter.on_progress("Test", 120.0));
    }
    
    #[test]
    fn test_force_progress_bypasses_throttling() {
        let reporter = ProgressReporter::with_interval(
            Some(Box::new(TestProgressCallback::new())), 
            Duration::from_millis(1000) // Long interval
        );
        
        // Force progress should always work - these calls should not panic
        reporter.force_progress("Test", 100.0);
        reporter.force_progress("Test", 110.0);
        reporter.force_progress("Test", 120.0);
    }
    
    #[test]
    fn test_progress_reporter_clone() {
        let reporter1 = ProgressReporter::new(Some(Box::new(TestProgressCallback::new())));
        let reporter2 = reporter1.clone();
        
        // Both reporters should work without panicking
        reporter1.on_test_start("Test1");
        reporter2.on_test_start("Test2");
        
        // Both should have callbacks
        assert!(reporter1.has_callback());
        assert!(reporter2.has_callback());
    }
    
    #[test]
    fn test_no_op_progress_callback() {
        let callback = NoOpProgressCallback;
        
        // These should not panic and should do nothing
        callback.on_test_start("Test");
        callback.on_progress("Test", 100.0);
        callback.on_test_complete("Test", &TestResult::default());
    }
    
    #[test]
    fn test_test_progress_callback_filtering() {
        let callback = TestProgressCallback::new();
        
        // Add various events
        callback.on_test_start("Sequential Write");
        callback.on_progress("Sequential Write", 100.0);
        callback.on_progress("Sequential Write", 150.0);
        callback.on_test_complete("Sequential Write", &TestResult::default());
        
        callback.on_test_start("Sequential Read");
        callback.on_progress("Sequential Read", 200.0);
        callback.on_test_complete("Sequential Read", &TestResult::default());
        
        // Test filtering
        assert_eq!(callback.event_count(), 7);
        
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 2);
        
        let write_progress = callback.progress_events_for_test("Sequential Write");
        assert_eq!(write_progress.len(), 2);
        assert_eq!(write_progress[0], 100.0);
        assert_eq!(write_progress[1], 150.0);
        
        let read_progress = callback.progress_events_for_test("Sequential Read");
        assert_eq!(read_progress.len(), 1);
        assert_eq!(read_progress[0], 200.0);
        
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 2);
    }
    
    #[test]
    fn test_test_progress_callback_clear() {
        let callback = TestProgressCallback::new();
        
        callback.on_test_start("Test");
        callback.on_progress("Test", 100.0);
        assert_eq!(callback.event_count(), 2);
        
        callback.clear();
        assert_eq!(callback.event_count(), 0);
        assert!(callback.events().is_empty());
    }
    
    #[test]
    fn test_progress_interval_management() {
        let mut reporter = ProgressReporter::new(None);
        
        // Test default interval
        assert_eq!(reporter.progress_interval(), Duration::from_millis(100));
        
        // Test setting new interval
        let new_interval = Duration::from_millis(250);
        reporter.set_progress_interval(new_interval);
        assert_eq!(reporter.progress_interval(), new_interval);
    }
    
    #[test]
    fn test_progress_event_debug_format() {
        let event1 = ProgressEvent::TestStart {
            test_name: "Test".to_string(),
        };
        let event2 = ProgressEvent::Progress {
            test_name: "Test".to_string(),
            speed_mbps: 123.45,
        };
        let event3 = ProgressEvent::TestComplete {
            test_name: "Test".to_string(),
            result: TestResult::default(),
        };
        
        // These should not panic and should produce reasonable debug output
        let _debug1 = format!("{:?}", event1);
        let _debug2 = format!("{:?}", event2);
        let _debug3 = format!("{:?}", event3);
    }
    
    #[test]
    fn test_concurrent_progress_reporting() {
        let reporter = Arc::new(ProgressReporter::new(Some(Box::new(TestProgressCallback::new()))));
        
        // Test concurrent access from multiple threads
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let reporter_clone = reporter.clone();
                thread::spawn(move || {
                    let test_name = format!("Test{}", i);
                    reporter_clone.on_test_start(&test_name);
                    reporter_clone.force_progress(&test_name, i as f64 * 10.0);
                    reporter_clone.on_test_complete(&test_name, &TestResult::default());
                })
            })
            .collect();
        
        // Wait for all threads to complete - this should not panic
        for handle in handles {
            handle.join().unwrap();
        }
        
        // The test passes if no panics occurred during concurrent access
        assert!(reporter.has_callback());
    }
}  
  #[test]
    fn test_test_progress_callback_standalone() {
        let callback = TestProgressCallback::new();
        
        // Test all callback methods
        callback.on_test_start("Sequential Write");
        callback.on_progress("Sequential Write", 150.5);
        callback.on_progress("Sequential Write", 175.2);
        
        let result = TestResult::new(100.0, 200.0, 150.0, Duration::from_secs(10), 100);
        callback.on_test_complete("Sequential Write", &result);
        
        let events = callback.events();
        assert_eq!(events.len(), 4);
        
        // Verify first event
        match &events[0] {
            ProgressEvent::TestStart { test_name } => {
                assert_eq!(test_name, "Sequential Write");
            }
            _ => panic!("Expected TestStart event"),
        }
        
        // Verify progress events
        match &events[1] {
            ProgressEvent::Progress { test_name, speed_mbps } => {
                assert_eq!(test_name, "Sequential Write");
                assert_eq!(*speed_mbps, 150.5);
            }
            _ => panic!("Expected Progress event"),
        }
        
        match &events[2] {
            ProgressEvent::Progress { test_name, speed_mbps } => {
                assert_eq!(test_name, "Sequential Write");
                assert_eq!(*speed_mbps, 175.2);
            }
            _ => panic!("Expected Progress event"),
        }
        
        // Verify completion event
        match &events[3] {
            ProgressEvent::TestComplete { test_name, result: test_result } => {
                assert_eq!(test_name, "Sequential Write");
                assert_eq!(test_result.avg_speed_mbps, 150.0);
                assert_eq!(test_result.min_speed_mbps, 100.0);
                assert_eq!(test_result.max_speed_mbps, 200.0);
            }
            _ => panic!("Expected TestComplete event"),
        }
        
        // Test filtering methods
        let progress_events = callback.progress_events_for_test("Sequential Write");
        assert_eq!(progress_events.len(), 2);
        assert_eq!(progress_events[0], 150.5);
        assert_eq!(progress_events[1], 175.2);
        
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Sequential Write");
        
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Sequential Write");
        assert_eq!(complete_events[0].1.avg_speed_mbps, 150.0);
    }
    
    #[test]
    fn test_progress_reporter_with_test_callback_integration() {
        // Create a shared callback that we can inspect
        let shared_callback = Arc::new(TestProgressCallback::new());
        
        // Create a wrapper that implements ProgressCallback and forwards to our shared callback
        struct CallbackWrapper {
            inner: Arc<TestProgressCallback>,
        }
        
        impl ProgressCallback for CallbackWrapper {
            fn on_test_start(&self, test_name: &str) {
                self.inner.on_test_start(test_name);
            }
            
            fn on_progress(&self, test_name: &str, current_speed_mbps: f64) {
                self.inner.on_progress(test_name, current_speed_mbps);
            }
            
            fn on_test_complete(&self, test_name: &str, result: &TestResult) {
                self.inner.on_test_complete(test_name, result);
            }
        }
        
        let wrapper = CallbackWrapper {
            inner: shared_callback.clone(),
        };
        
        let reporter = ProgressReporter::new(Some(Box::new(wrapper)));
        
        // Test the reporter
        reporter.on_test_start("Random Write");
        reporter.force_progress("Random Write", 85.3);
        
        let result = TestResult::new(50.0, 100.0, 75.0, Duration::from_secs(5), 50);
        reporter.on_test_complete("Random Write", &result);
        
        // Verify events were captured
        let events = shared_callback.events();
        assert_eq!(events.len(), 3);
        
        let start_events = shared_callback.test_start_events();
        assert_eq!(start_events.len(), 1);
        assert_eq!(start_events[0], "Random Write");
        
        let progress_events = shared_callback.progress_events_for_test("Random Write");
        assert_eq!(progress_events.len(), 1);
        assert_eq!(progress_events[0], 85.3);
        
        let complete_events = shared_callback.test_complete_events();
        assert_eq!(complete_events.len(), 1);
        assert_eq!(complete_events[0].0, "Random Write");
        assert_eq!(complete_events[0].1.avg_speed_mbps, 75.0);
    }