# Makefile for disk-speed-test project
# Provides convenient commands for development and CI

.PHONY: help test test-unit test-integration test-all coverage clean format lint audit install-tools ci-setup

# Default target
help:
	@echo "Available targets:"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-all       - Run comprehensive test suite"
	@echo "  coverage       - Generate code coverage report"
 
	@echo "  format         - Format code"
	@echo "  lint           - Run clippy linting"
	@echo "  audit          - Run security audit"
	@echo "  clean          - Clean build artifacts"
	@echo "  install-tools  - Install required development tools"
	@echo "  ci-setup       - Set up CI environment"

# Test targets
test: test-unit test-integration

test-unit:
	@echo "Running unit tests..."
	cargo test --lib --bins

test-integration:
	@echo "Running integration tests..."
	cargo test --test '*'

test-all:
	@echo "Running comprehensive test suite..."
	./scripts/test-runner.sh

# Coverage target
coverage:
	@echo "Generating code coverage..."
	cargo llvm-cov --all-features --workspace --html --open

coverage-ci:
	@echo "Generating code coverage for CI..."
	cargo llvm-cov --all-features --workspace --lcov --output-path coverage.lcov

# Benchmark targets removed

# Code quality targets
format:
	@echo "Formatting code..."
	cargo fmt --all

format-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

audit:
	@echo "Running security audit..."
	cargo audit

# Build targets
build:
	@echo "Building project..."
	cargo build

build-release:
	@echo "Building release..."
	cargo build --release

# Cross-compilation targets
build-windows:
	@echo "Cross-compiling for Windows..."
	cross build --target x86_64-pc-windows-gnu --release
	cross build --target x86_64-pc-windows-msvc --release

build-macos:
	@echo "Cross-compiling for macOS..."
	cargo build --target x86_64-apple-darwin --release
	cargo build --target aarch64-apple-darwin --release

build-linux:
	@echo "Cross-compiling for Linux..."
	cargo build --target x86_64-unknown-linux-gnu --release
	cargo build --target aarch64-unknown-linux-gnu --release
	cross build --target x86_64-unknown-linux-musl --release
	cross build --target aarch64-unknown-linux-musl --release

build-mobile:
	@echo "Cross-compiling for mobile platforms..."
	cross build --target aarch64-linux-android --release
	cross build --target armv7-linux-androideabi --release
	cargo build --target aarch64-apple-ios --release

build-all-platforms: build-windows build-macos build-linux

build-cross-platform:
	@echo "Building all platforms using cross-platform script..."
	./scripts/build-cross-platform.sh --all

# Utility targets
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	rm -rf coverage.lcov
	rm -rf coverage-html/
	rm -f test-report.md

install-tools:
	@echo "Installing development tools..."
	cargo install cargo-llvm-cov
	cargo install cargo-audit
	cargo install cargo-nextest
	cargo install cross

# CI targets
ci-setup: install-tools
	@echo "Setting up CI environment..."
	rustup component add rustfmt clippy llvm-tools-preview

ci-test:
	@echo "Running CI test suite..."
	make format-check
	make lint
	make test-all
	make coverage-ci

# Documentation targets
doc:
	@echo "Building documentation..."
	cargo doc --all-features --no-deps

doc-open:
	@echo "Building and opening documentation..."
	cargo doc --all-features --no-deps --open

# Development targets
dev-setup: install-tools
	@echo "Setting up development environment..."
	rustup component add rustfmt clippy llvm-tools-preview
	@echo "Development environment ready!"

watch-test:
	@echo "Watching for changes and running tests..."
	cargo watch -x "test --lib --bins"

watch-check:
	@echo "Watching for changes and running checks..."
	cargo watch -x check -x "clippy --all-targets --all-features"

# Release targets
release-check:
	@echo "Running release checks..."
	make format-check
	make lint
	make test-all
	make audit

release-build: release-check
	@echo "Building release binaries..."
	./scripts/build-cross-platform.sh --all

release-prepare:
	@echo "Preparing release..."
	./scripts/release.sh prepare $(VERSION)

release-publish:
	@echo "Publishing release..."
	./scripts/release.sh publish

release-full:
	@echo "Running full release process..."
	./scripts/release.sh full $(VERSION)

# Platform-specific test targets
test-windows:
	@echo "Running Windows-specific tests..."
	cargo test --test '*' --features windows-tests

test-macos:
	@echo "Running macOS-specific tests..."
	cargo test --test '*' --features macos-tests

test-linux:
	@echo "Running Linux-specific tests..."
	cargo test --test '*' --features linux-tests

# Performance targets
perf-profile:
	@echo "Running performance profiling..."
	cargo build --release
	perf record --call-graph=dwarf target/release/disk-speed-test benchmark /tmp
	perf report

flamegraph:
	@echo "Generating flamegraph..."
	cargo flamegraph --bin disk-speed-test -- benchmark /tmp

# Docker targets (if using containers)
docker-build:
	@echo "Building Docker image..."
	docker build -t disk-speed-test .

docker-test:
	@echo "Running tests in Docker..."
	docker run --rm -v $(PWD):/workspace disk-speed-test make test-all

# Variables for customization
RUST_LOG ?= info
CARGO_TARGET_DIR ?= target
TEST_THREADS ?= 1

# Export environment variables
export RUST_LOG
export CARGO_TARGET_DIR
export RUST_TEST_THREADS=$(TEST_THREADS)