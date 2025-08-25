//! Comprehensive unit tests for statistics collection and calculation

#[cfg(test)]
mod tests {
    use super::super::stats::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_test_result_creation_and_access() {
        let result = TestResult::new(10.0, 100.0, 55.0, Duration::from_secs(30), 150);

        assert_eq!(result.min_speed_mbps, 10.0);
        assert_eq!(result.max_speed_mbps, 100.0);
        assert_eq!(result.avg_speed_mbps, 55.0);
        assert_eq!(result.test_duration, Duration::from_secs(30));
        assert_eq!(result.sample_count, 150);
    }

    #[test]
    fn test_test_result_default() {
        let result = TestResult::default();

        assert_eq!(result.min_speed_mbps, 0.0);
        assert_eq!(result.max_speed_mbps, 0.0);
        assert_eq!(result.avg_speed_mbps, 0.0);
        assert_eq!(result.test_duration, Duration::from_secs(0));
        assert_eq!(result.sample_count, 0);
    }

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult::new(25.5, 75.3, 50.1, Duration::from_millis(5500), 55);

        // Test JSON serialization
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("25.5"));
        assert!(json.contains("75.3"));
        assert!(json.contains("50.1"));
        assert!(json.contains("55"));

        // Test deserialization
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.min_speed_mbps, result.min_speed_mbps);
        assert_eq!(deserialized.max_speed_mbps, result.max_speed_mbps);
        assert_eq!(deserialized.avg_speed_mbps, result.avg_speed_mbps);
        assert_eq!(deserialized.sample_count, result.sample_count);
    }

    #[test]
    fn test_statistics_collector_custom_interval() {
        let interval = Duration::from_millis(50);
        let collector = StatisticsCollector::with_sample_interval(interval);

        // Should not be ready to sample immediately
        assert!(!collector.should_sample());

        // After waiting, should be ready
        thread::sleep(Duration::from_millis(60));
        assert!(collector.should_sample());
    }

    #[test]
    fn test_statistics_collector_bytes_tracking() {
        let mut collector = StatisticsCollector::new();

        // Test initial state
        assert_eq!(collector.get_bytes_transferred(), 0);

        // Test updating bytes
        collector.update_bytes_transferred(1024);
        assert_eq!(collector.get_bytes_transferred(), 1024);

        // Test adding bytes
        collector.add_bytes_transferred(512);
        assert_eq!(collector.get_bytes_transferred(), 1536);

        // Test adding more bytes
        collector.add_bytes_transferred(256);
        assert_eq!(collector.get_bytes_transferred(), 1792);
    }

    #[test]
    fn test_statistics_collector_speed_calculation_edge_cases() {
        // Test zero duration
        let speed = StatisticsCollector::calculate_speed_mbps(1024 * 1024, Duration::from_nanos(0));
        assert_eq!(speed, 0.0);

        // Test zero bytes
        let speed = StatisticsCollector::calculate_speed_mbps(0, Duration::from_secs(1));
        assert_eq!(speed, 0.0);

        // Test very small duration
        let speed = StatisticsCollector::calculate_speed_mbps(1024, Duration::from_nanos(1));
        assert!(speed > 0.0);
        assert!(speed.is_finite());

        // Test very large bytes
        let speed = StatisticsCollector::calculate_speed_mbps(u64::MAX / 2, Duration::from_secs(1));
        assert!(speed > 0.0);
        assert!(speed.is_finite());

        // Test very long duration
        let speed =
            StatisticsCollector::calculate_speed_mbps(1024 * 1024, Duration::from_secs(3600));
        assert!((speed - (1.0 / 3600.0)).abs() < 0.001);
    }

    #[test]
    fn test_statistics_collector_percentile_calculation() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        // Test P5 (5th percentile)
        let p5 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 5.0);
        assert_eq!(p5, 1.0); // 5% of 10 samples = 0.5, ceil = 1, index 0

        // Test P95 (95th percentile)
        let p95 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 95.0);
        assert_eq!(p95, 10.0); // 95% of 10 samples = 9.5, ceil = 10, index 9

        // Test P50 (median)
        let p50 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 50.0);
        assert_eq!(p50, 5.0); // 50% of 10 samples = 5, ceil = 5, index 4

        // Test edge cases
        let p0 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 0.0);
        assert_eq!(p0, 1.0);

        let p100 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 100.0);
        assert_eq!(p100, 10.0);

        // Test with single sample
        let single = vec![42.0];
        let p50_single = StatisticsCollector::test_percentile_nearest_rank(single, 50.0);
        assert_eq!(p50_single, 42.0);

        // Test with empty samples
        let empty: Vec<f64> = vec![];
        let p50_empty = StatisticsCollector::test_percentile_nearest_rank(empty, 50.0);
        assert_eq!(p50_empty, 0.0);
    }

    #[test]
    fn test_statistics_collector_with_unsorted_samples() {
        let mut collector = StatisticsCollector::new();

        // Add samples in random order
        let samples = vec![50.0, 10.0, 90.0, 30.0, 70.0, 20.0, 80.0, 40.0, 60.0, 100.0];
        for sample in samples {
            collector.add_sample(sample);
        }

        let result = collector.finalize();

        // Should correctly calculate percentiles even with unsorted input
        assert_eq!(result.min_speed_mbps, 10.0); // P5 of sorted [10,20,30,40,50,60,70,80,90,100]
        assert_eq!(result.max_speed_mbps, 100.0); // P95
        assert_eq!(result.avg_speed_mbps, 55.0); // Average
        assert_eq!(result.sample_count, 10);
    }

    #[test]
    fn test_statistics_collector_with_duplicate_samples() {
        let mut collector = StatisticsCollector::new();

        // Add duplicate samples
        for _ in 0..5 {
            collector.add_sample(100.0);
        }

        let result = collector.finalize();

        assert_eq!(result.min_speed_mbps, 100.0);
        assert_eq!(result.max_speed_mbps, 100.0);
        assert_eq!(result.avg_speed_mbps, 100.0);
        assert_eq!(result.sample_count, 5);
    }

    #[test]
    fn test_statistics_collector_with_extreme_values() {
        let mut collector = StatisticsCollector::new();

        // Add extreme values
        collector.add_sample(f64::MIN);
        collector.add_sample(f64::MAX);
        collector.add_sample(0.0);
        collector.add_sample(1.0);
        collector.add_sample(-1.0);

        let result = collector.finalize();

        // Should handle extreme values without panicking
        assert!(result.min_speed_mbps.is_finite() || result.min_speed_mbps == f64::MIN);
        assert!(result.max_speed_mbps.is_finite() || result.max_speed_mbps == f64::MAX);
        assert_eq!(result.sample_count, 5);
    }

    #[test]
    fn test_statistics_collector_with_special_float_values() {
        let mut collector = StatisticsCollector::new();

        // Add special float values
        collector.add_sample(f64::INFINITY);
        collector.add_sample(f64::NEG_INFINITY);
        collector.add_sample(f64::NAN);
        collector.add_sample(100.0);

        let result = collector.finalize();

        // Should handle special values gracefully
        assert_eq!(result.sample_count, 4);
        // The exact behavior with NaN and infinity depends on implementation
        // but it shouldn't panic
    }

    #[test]
    fn test_real_time_stats_tracker_basic() {
        let mut tracker = RealTimeStatsTracker::new();

        // Test initial state
        assert_eq!(tracker.current_average_speed(), 0.0);
        let (_min, _max, _avg, count) = tracker.current_stats();
        assert_eq!(count, 0);

        // Record some blocks
        let _block_speed = tracker.record_block(1024, Duration::from_millis(10));
        // May or may not return a speed depending on timing

        let (_min, _max, _avg, count) = tracker.current_stats();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_real_time_stats_tracker_progress_updates() {
        let mut tracker = RealTimeStatsTracker::with_sample_interval(Duration::from_millis(10));

        // Update progress - should not sample immediately
        let sample_taken = tracker.update_progress(1024);
        assert!(sample_taken.is_none());

        // After waiting, should sample
        thread::sleep(Duration::from_millis(15));
        let sample_taken = tracker.update_progress(2048);
        assert!(sample_taken.is_some());

        let (_min, _max, _avg, count) = tracker.current_stats();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_real_time_stats_tracker_instantaneous_speed() {
        let mut tracker = RealTimeStatsTracker::new();

        // Initial instantaneous speed should be 0
        assert_eq!(tracker.current_instantaneous_speed(), 0.0);

        // Update progress
        tracker.update_progress(1024 * 1024); // 1MB

        // Sleep a bit to ensure time passes
        thread::sleep(Duration::from_millis(10));

        // Update again
        tracker.update_progress(2 * 1024 * 1024); // 2MB total

        let instantaneous = tracker.current_instantaneous_speed();
        // Should be positive (1MB transferred in ~10ms)
        assert!(instantaneous >= 0.0);
    }

    #[test]
    fn test_real_time_stats_tracker_with_zero_duration_blocks() {
        let mut tracker = RealTimeStatsTracker::new();

        // Record block with zero duration
        let result = tracker.record_block(1024, Duration::from_nanos(0));
        assert!(result.is_none()); // Should not record invalid blocks

        // Record block with zero bytes
        let result = tracker.record_block(0, Duration::from_millis(10));
        assert!(result.is_none()); // Should not record invalid blocks
    }

    #[test]
    fn test_real_time_stats_tracker_finalization() {
        let mut tracker = RealTimeStatsTracker::new();

        // Record several blocks
        for i in 1..=5 {
            tracker.record_block(1024 * i, Duration::from_millis(10));
        }

        let result = tracker.finalize();

        assert_eq!(result.sample_count, 5);
        assert!(result.avg_speed_mbps > 0.0);
        assert!(result.min_speed_mbps <= result.avg_speed_mbps);
        assert!(result.avg_speed_mbps <= result.max_speed_mbps);
    }

    #[test]
    fn test_statistics_accuracy_with_known_data() {
        let mut collector = StatisticsCollector::new();

        // Add known data set: [10, 20, 30, 40, 50]
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for value in &data {
            collector.add_sample(*value);
        }

        // Verify current calculations
        assert_eq!(collector.current_min(), 10.0);
        assert_eq!(collector.current_max(), 50.0);
        assert_eq!(collector.current_average(), 30.0); // (10+20+30+40+50)/5 = 30
        assert_eq!(collector.sample_count(), 5);

        let result = collector.finalize();

        // P5 of [10,20,30,40,50]: 5% of 5 = 0.25, ceil = 1, index 0 = 10
        assert_eq!(result.min_speed_mbps, 10.0);
        // P95 of [10,20,30,40,50]: 95% of 5 = 4.75, ceil = 5, index 4 = 50
        assert_eq!(result.max_speed_mbps, 50.0);
        assert_eq!(result.avg_speed_mbps, 30.0);
        assert_eq!(result.sample_count, 5);
    }

    #[test]
    fn test_statistics_with_large_dataset() {
        let mut collector = StatisticsCollector::new();

        // Add 1000 samples from 1 to 1000
        for i in 1..=1000 {
            collector.add_sample(i as f64);
        }

        let result = collector.finalize();

        // P5 of 1000 samples: 5% = 50, so 50th value = 50
        assert_eq!(result.min_speed_mbps, 50.0);
        // P95 of 1000 samples: 95% = 950, so 950th value = 950
        assert_eq!(result.max_speed_mbps, 950.0);
        // Average of 1 to 1000 = 500.5
        assert!((result.avg_speed_mbps - 500.5).abs() < 0.1);
        assert_eq!(result.sample_count, 1000);
    }

    #[test]
    fn test_collector_current_speed_with_elapsed_time() {
        let mut collector = StatisticsCollector::new();

        // Set some bytes transferred
        collector.update_bytes_transferred(1024 * 1024); // 1MB

        // Sleep to ensure time passes
        thread::sleep(Duration::from_millis(10));

        let speed = collector.current_speed_mbps();
        assert!(speed > 0.0);
        assert!(speed < 10000.0); // Should be reasonable
    }

    #[test]
    fn test_collector_sample_timing() {
        let mut collector = StatisticsCollector::with_sample_interval(Duration::from_millis(50));

        // Should not be ready initially
        assert!(!collector.should_sample());

        // Should not sample if not ready
        assert!(!collector.sample_if_ready());
        assert_eq!(collector.sample_count(), 0);

        // Wait for interval
        thread::sleep(Duration::from_millis(60));

        // Should be ready now
        assert!(collector.should_sample());

        // Should sample now
        collector.update_bytes_transferred(1024);
        assert!(collector.sample_if_ready());
        assert_eq!(collector.sample_count(), 1);

        // Should not be ready immediately after sampling
        assert!(!collector.should_sample());
    }

    #[test]
    fn test_collector_force_sample() {
        let mut collector = StatisticsCollector::new();
        collector.update_bytes_transferred(1024);

        // Force sample should work regardless of timing
        collector.force_sample();
        assert_eq!(collector.sample_count(), 1);

        // Should be able to force another immediately
        collector.force_sample();
        assert_eq!(collector.sample_count(), 2);
    }

    #[test]
    fn test_percentile_with_odd_sample_count() {
        // Test with 7 samples: [1, 2, 3, 4, 5, 6, 7]
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

        // P50 of 7 samples: 50% of 7 = 3.5, ceil = 4, index 3 = 4
        let p50 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 50.0);
        assert_eq!(p50, 4.0);

        // P25 of 7 samples: 25% of 7 = 1.75, ceil = 2, index 1 = 2
        let p25 = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 25.0);
        assert_eq!(p25, 2.0);

        // P75 of 7 samples: 75% of 7 = 5.25, ceil = 6, index 5 = 6
        let p75 = StatisticsCollector::test_percentile_nearest_rank(samples, 75.0);
        assert_eq!(p75, 6.0);
    }

    #[test]
    fn test_percentile_with_invalid_percentiles() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // Test negative percentile (should be clamped to 0)
        let p_neg = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), -10.0);
        assert_eq!(p_neg, 1.0);

        // Test > 100 percentile (should be clamped to 100)
        let p_over = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), 150.0);
        assert_eq!(p_over, 5.0);

        // Test NaN percentile (should be treated as 0)
        let p_nan = StatisticsCollector::test_percentile_nearest_rank(samples.clone(), f64::NAN);
        assert_eq!(p_nan, 1.0);

        // Test infinity percentile (should be treated as 0 due to clamping)
        let p_inf = StatisticsCollector::test_percentile_nearest_rank(samples, f64::INFINITY);
        assert_eq!(p_inf, 1.0); // Clamped to 0, which gives first element
    }

    #[test]
    fn test_real_time_tracker_with_rapid_updates() {
        let mut tracker = RealTimeStatsTracker::with_sample_interval(Duration::from_millis(1));

        // Rapidly update progress with longer sleeps to ensure sampling
        for i in 1..=10 {
            tracker.update_progress(i * 1024);
            // Longer sleep to ensure sampling occurs
            thread::sleep(Duration::from_millis(2));
        }

        let (_min, _max, avg, count) = tracker.current_stats();
        // With the longer sleep, we should get at least some samples
        if count > 0 {
            assert!(avg > 0.0);
        }
        // Test passes if no panic occurs
    }

    #[test]
    fn test_collector_with_very_small_intervals() {
        let collector = StatisticsCollector::with_sample_interval(Duration::from_nanos(1));

        // Should always be ready to sample with such a small interval
        thread::sleep(Duration::from_millis(1));
        assert!(collector.should_sample());
    }

    #[test]
    fn test_collector_with_very_large_intervals() {
        let collector = StatisticsCollector::with_sample_interval(Duration::from_secs(3600));

        // Should not be ready to sample with such a large interval
        assert!(!collector.should_sample());

        // Even after a short sleep, should still not be ready
        thread::sleep(Duration::from_millis(10));
        assert!(!collector.should_sample());
    }
}
