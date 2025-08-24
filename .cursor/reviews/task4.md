### Task 4 Review — Statistics collection engine

**Verdict: Implemented (meets intent) with a cadence note**

#### What’s complete
- **Statistics data model** (`src/core/stats.rs`):
  - `TestResult` includes `min_speed_mbps`, `max_speed_mbps`, `avg_speed_mbps`, `test_duration`, and `sample_count`. Serde derives present. Defaults provided.
- **Collection engine** (`StatisticsCollector`):
  - Manages an in-memory sample list and timing (`start_time`, `last_sample_time`).
  - Provides `add_sample`, `current_min`, `current_max`, `current_average`, `elapsed`, and `finalize()` to compute `TestResult`.
  - Includes `calculate_speed_mbps(bytes, duration)` to convert counters to throughput — matches “real-time speed calculation” requirement.
- **Unit tests**:
  - Validate min/max/avg correctness with known samples.
  - Validate MB/s conversion for simple cases.
  - Handle empty-collector edge case (returns zeroed `TestResult`).

#### Cadence (100ms sampling) note
- The implementation exposes primitives for sampling but does not internally enforce a 100ms cadence (e.g., via a timer or throttle). The `last_sample_time` field is tracked but not used to gate sampling frequency. It is reasonable to implement the 100ms scheduling in the test runners (Tasks 9–13) using this collector; however, if the acceptance criterion expects the collector to enforce cadence, that part is not present yet.

#### Alignment to requirements (Task 4 references)
- **Req 8.1–8.3**: Data model supports min/max/avg in MB/s; presentation formatting (bold avg) is a CLI concern for a later task.
- **Req 8.4**: Collector can ingest frequent samples; tests verify correctness and edge cases.
- **Req 8.5**: Units are MB/s throughout (`calculate_speed_mbps`).

#### Recommendations (non-blocking)
- Either (a) document that sampling cadence is controlled by the caller (recommended), or (b) add an optional helper (e.g., `try_add_sample_throttled(100ms)`) using `last_sample_time`.
- Consider numeric stability safeguards if very long runs accumulate many samples (e.g., incremental average), though f64 is likely sufficient for intended durations.

Overall, the statistics engine fulfills Task 4’s goals; only the explicit 100ms cadence enforcement is deferred to callers.


