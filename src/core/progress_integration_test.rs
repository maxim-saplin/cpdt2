//! Integration tests for progress reporting system
//! 
//! These tests verify that the progress reporting system works correctly
//! in realistic scenarios that simulate actual benchmark execution.

#[cfg(test)]
mod integration_tests {
    use crate::core::{ProgressReporter, TestProgressCallback, ProgressCallback, TestResult};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Test that simulates a complete benchmark run with progress reporting
    #[test]
    fn test_complete_benchmark_simulation() {
        let callback = Arc::new(TestProgressCallback::new());
        
        // Create a wrapper for the callback
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
            inner: callback.clone(),
        };
        
        let reporter = ProgressReporter::with_interval(
            Some(Box::new(wrapper)),
            Duration::from_millis(10) // Fast interval for testing
        );
        
        // Simulate all five benchmark tests as per requirements
        let test_names = [
            "Sequential Write",
            "Sequential Read", 
            "Random Write",
            "Random Read",
            "Memory Copy"
        ];
        
        for (i, test_name) in test_names.iter().enumerate() {
            // Start test
            reporter.on_test_start(test_name);
            
            // Simulate progress updates during test execution
            let base_speed = (i + 1) as f64 * 50.0; // Different speeds for each test
            for j in 0..5 {
                thread::sleep(Duration::from_millis(15)); // Ensure throttling works
                let speed = base_speed + (j as f64 * 10.0);
                reporter.on_progress(test_name, speed);
            }
            
            // Complete test
            let result = TestResult::new(
                base_speed,
                base_speed + 40.0,
                base_speed + 20.0,
                Duration::from_millis(100),
                5
            );
            reporter.on_test_complete(test_name, &result);
        }
        
        // Verify all events were captured correctly
        let events = callback.events();
        
        // Should have: 5 starts + 5 progress updates + 5 completions = 15 events minimum
        // (Some progress updates might be throttled)
        assert!(events.len() >= 15);
        
        // Verify we have all test start events
        let start_events = callback.test_start_events();
        assert_eq!(start_events.len(), 5);
        for test_name in &test_names {
            assert!(start_events.contains(&test_name.to_string()));
        }
        
        // Verify we have all test completion events
        let complete_events = callback.test_complete_events();
        assert_eq!(complete_events.len(), 5);
        for test_name in &test_names {
            let found = complete_events.iter().any(|(name, _)| name == test_name);
            assert!(found, "Missing completion event for {}", test_name);
        }
        
        // Verify progress events exist for each test
        for test_name in &test_names {
            let progress_events = callback.progress_events_for_test(test_name);
            assert!(!progress_events.is_empty(), "No progress events for {}", test_name);
        }
    }
    
    /// Test progress reporting with realistic timing and throttling
    #[test]
    fn test_realistic_progress_timing() {
        let callback = TestProgressCallback::new();
        let reporter = ProgressReporter::with_interval(
            Some(Box::new(callback)),
            Duration::from_millis(100) // Standard 100ms interval
        );
        
        reporter.on_test_start("Sequential Write");
        
        // Simulate rapid progress updates (faster than throttling interval)
        let start_time = std::time::Instant::now();
        let mut successful_updates = 0;
        
        while start_time.elapsed() < Duration::from_millis(500) {
            if reporter.on_progress("Sequential Write", 150.0) {
                successful_updates += 1;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        // Should have approximately 5 successful updates (500ms / 100ms interval)
        assert!((4..=6).contains(&successful_updates), 
                "Expected 4-6 successful updates, got {}", successful_updates);
        
        // Force final progress update
        reporter.force_progress("Sequential Write", 200.0);
        
        let result = TestResult::new(100.0, 200.0, 150.0, Duration::from_millis(500), 50);
        reporter.on_test_complete("Sequential Write", &result);
    }
    
    /// Test that progress reporting works correctly with no callback
    #[test]
    fn test_no_callback_scenario() {
        let reporter = ProgressReporter::new(None);
        
        // All these operations should work without panicking
        reporter.on_test_start("Test");
        assert!(!reporter.on_progress("Test", 100.0)); // Should return false (no callback)
        reporter.force_progress("Test", 150.0);
        reporter.on_test_complete("Test", &TestResult::default());
        
        assert!(!reporter.has_callback());
    }
    
    /// Test error handling and edge cases in progress reporting
    #[test]
    fn test_progress_reporting_edge_cases() {
        let callback = TestProgressCallback::new();
        let reporter = ProgressReporter::new(Some(Box::new(callback)));
        
        // Test with empty test name
        reporter.on_test_start("");
        reporter.on_progress("", 0.0);
        reporter.on_test_complete("", &TestResult::default());
        
        // Test with very long test name
        let long_name = "A".repeat(1000);
        reporter.on_test_start(&long_name);
        reporter.on_progress(&long_name, 1000.0);
        reporter.on_test_complete(&long_name, &TestResult::default());
        
        // Test with extreme speed values
        reporter.on_progress("Test", f64::MAX);
        reporter.on_progress("Test", f64::MIN);
        reporter.on_progress("Test", 0.0);
        reporter.on_progress("Test", -1.0); // Shouldn't happen in practice but test robustness
        
        // Test with NaN and infinity
        reporter.on_progress("Test", f64::NAN);
        reporter.on_progress("Test", f64::INFINITY);
        reporter.on_progress("Test", f64::NEG_INFINITY);
        
        // All operations should complete without panicking
        assert!(reporter.has_callback());
    }
}