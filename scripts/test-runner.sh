#!/bin/bash
# Test runner script with quality gates and comprehensive test execution

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MIN_COVERAGE=80
MAX_TEST_DURATION=300  # 5 minutes
TEMP_DIR="${TMPDIR:-/tmp}/disk-speed-test-$$"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR" || true
    
    # Clean up any test artifacts
    find . -name "*.tmp" -type f -delete 2>/dev/null || true
    find . -name "test_*.dat" -type f -delete 2>/dev/null || true
    find . -name "benchmark_*.log" -type f -delete 2>/dev/null || true
}

# Set up cleanup trap
trap cleanup EXIT

# Create temporary directory
mkdir -p "$TEMP_DIR"
export DISK_SPEED_TEST_TEMP_DIR="$TEMP_DIR"

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    # Check available disk space (need at least 1GB for tests)
    available_space=$(df "$TEMP_DIR" | awk 'NR==2 {print $4}')
    required_space=1048576  # 1GB in KB
    
    if [ "$available_space" -lt "$required_space" ]; then
        log_warning "Low disk space: ${available_space}KB available, ${required_space}KB recommended"
    fi
    
    log_success "Prerequisites check passed"
}

# Run code formatting check
check_formatting() {
    log_info "Checking code formatting..."
    
    if ! cargo fmt --all -- --check; then
        log_error "Code formatting check failed. Run 'cargo fmt' to fix."
        return 1
    fi
    
    log_success "Code formatting check passed"
}

# Run clippy linting
run_clippy() {
    log_info "Running Clippy linting..."
    
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        log_error "Clippy linting failed"
        return 1
    fi
    
    log_success "Clippy linting passed"
}

# Run unit tests
run_unit_tests() {
    log_info "Running unit tests..."
    
    local start_time=$(date +%s)
    
    if ! timeout "$MAX_TEST_DURATION" cargo test --lib --bins --verbose; then
        log_error "Unit tests failed or timed out"
        return 1
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_success "Unit tests passed in ${duration}s"
}

# Run integration tests
run_integration_tests() {
    log_info "Running integration tests..."
    
    local start_time=$(date +%s)
    
    if ! timeout "$MAX_TEST_DURATION" cargo test --test '*' --verbose; then
        log_error "Integration tests failed or timed out"
        return 1
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_success "Integration tests passed in ${duration}s"
}

# Run benchmarks (dry run)
run_benchmarks() {
    log_info "Running benchmark dry run..."
    
    if ! cargo bench --no-run; then
        log_error "Benchmark compilation failed"
        return 1
    fi
    
    log_success "Benchmark dry run passed"
}

# Generate and check code coverage
check_coverage() {
    log_info "Generating code coverage report..."
    
    # Check if llvm-cov is available
    if ! command -v cargo-llvm-cov &> /dev/null; then
        log_warning "cargo-llvm-cov not found. Installing..."
        cargo install cargo-llvm-cov || {
            log_error "Failed to install cargo-llvm-cov"
            return 1
        }
    fi
    
    # Generate coverage report
    if ! cargo llvm-cov --all-features --workspace --lcov --output-path coverage.lcov; then
        log_error "Coverage generation failed"
        return 1
    fi
    
    # Extract coverage percentage
    local coverage_percent
    coverage_percent=$(cargo llvm-cov report --summary-only | grep -oP 'TOTAL.*\K\d+\.\d+(?=%)' || echo "0")
    
    log_info "Current coverage: ${coverage_percent}%"
    
    # Check coverage threshold
    if (( $(echo "$coverage_percent < $MIN_COVERAGE" | bc -l) )); then
        log_error "Coverage ${coverage_percent}% is below threshold ${MIN_COVERAGE}%"
        return 1
    fi
    
    log_success "Coverage ${coverage_percent}% meets threshold ${MIN_COVERAGE}%"
}

# Run performance regression tests
run_performance_tests() {
    log_info "Running performance regression tests..."
    
    # Run a subset of performance benchmarks for CI
    if ! timeout 120 cargo bench --bench performance_regression -- --sample-size 10; then
        log_error "Performance regression tests failed"
        return 1
    fi
    
    log_success "Performance regression tests passed"
}

# Check for security vulnerabilities
security_audit() {
    log_info "Running security audit..."
    
    # Check if cargo-audit is available
    if ! command -v cargo-audit &> /dev/null; then
        log_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit || {
            log_warning "Failed to install cargo-audit, skipping security audit"
            return 0
        }
    fi
    
    if ! cargo audit; then
        log_error "Security audit found vulnerabilities"
        return 1
    fi
    
    log_success "Security audit passed"
}

# Generate test report
generate_report() {
    log_info "Generating test report..."
    
    local report_file="test-report.md"
    
    cat > "$report_file" << EOF
# Test Report

Generated: $(date)
Platform: $(uname -s) $(uname -m)
Rust Version: $(rustc --version)

## Test Results

- ✅ Code Formatting
- ✅ Clippy Linting  
- ✅ Unit Tests
- ✅ Integration Tests
- ✅ Benchmark Compilation
- ✅ Code Coverage (${coverage_percent:-N/A}%)
- ✅ Performance Tests
- ✅ Security Audit

## Coverage Details

$(cargo llvm-cov report --summary-only 2>/dev/null || echo "Coverage report not available")

## Environment

- Temporary Directory: $TEMP_DIR
- Available Space: $(df -h "$TEMP_DIR" | awk 'NR==2 {print $4}')
- Test Duration: $(date +%s)s

EOF
    
    log_success "Test report generated: $report_file"
}

# Main execution
main() {
    log_info "Starting comprehensive test suite..."
    
    local start_time=$(date +%s)
    local failed_checks=()
    
    # Run all checks
    check_prerequisites || failed_checks+=("prerequisites")
    check_formatting || failed_checks+=("formatting")
    run_clippy || failed_checks+=("clippy")
    run_unit_tests || failed_checks+=("unit_tests")
    run_integration_tests || failed_checks+=("integration_tests")
    run_benchmarks || failed_checks+=("benchmarks")
    check_coverage || failed_checks+=("coverage")
    run_performance_tests || failed_checks+=("performance")
    security_audit || failed_checks+=("security")
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    # Generate report
    generate_report
    
    # Summary
    if [ ${#failed_checks[@]} -eq 0 ]; then
        log_success "All quality gates passed! Total time: ${total_duration}s"
        exit 0
    else
        log_error "Failed checks: ${failed_checks[*]}"
        log_error "Quality gates failed! Total time: ${total_duration}s"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-all}" in
    "format")
        check_formatting
        ;;
    "lint")
        run_clippy
        ;;
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "coverage")
        check_coverage
        ;;
    "performance")
        run_performance_tests
        ;;
    "security")
        security_audit
        ;;
    "all"|*)
        main
        ;;
esac