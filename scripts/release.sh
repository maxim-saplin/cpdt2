#!/bin/bash

# Release automation script for disk-speed-test
# Handles versioning, building, and publishing releases

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="disk-speed-test"
CARGO_TOML="Cargo.toml"
CHANGELOG="CHANGELOG.md"
README="README.md"

# Functions
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

show_help() {
    cat << EOF
Release automation script for $PROJECT_NAME

Usage: $0 <command> [options]

Commands:
  prepare <version>    Prepare a new release (update version, changelog)
  build               Build release binaries for all platforms
  publish             Publish release to GitHub/crates.io
  full <version>      Run complete release process (prepare + build + publish)

Options:
  --dry-run          Show what would be done without making changes
  --skip-tests       Skip running tests before release
  --skip-build       Skip building binaries (for prepare/publish only)
  --help             Show this help message

Examples:
  $0 prepare 1.2.0           # Prepare version 1.2.0
  $0 build                   # Build release binaries
  $0 full 1.2.0 --dry-run    # Show what full release would do
  $0 publish                 # Publish current version

Version formats:
  - Semantic versioning: 1.2.3, 2.0.0-beta.1, 1.0.0-rc.1
  - Must follow semver specification
EOF
}

get_current_version() {
    grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
}

validate_version() {
    local version=$1
    
    # Basic semver validation
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_info "Version must follow semantic versioning (e.g., 1.2.3, 2.0.0-beta.1)"
        return 1
    fi
    
    return 0
}

check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working directory is not clean. Please commit or stash changes."
        git status --short
        return 1
    fi
    
    # Check if we're on main/master branch
    local current_branch=$(git branch --show-current)
    if [[ $current_branch != "main" && $current_branch != "master" ]]; then
        log_warning "Not on main/master branch (current: $current_branch)"
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Release cancelled"
            return 1
        fi
    fi
    
    return 0
}

update_version() {
    local new_version=$1
    local current_version=$(get_current_version)
    
    log_info "Updating version from $current_version to $new_version"
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would update version in $CARGO_TOML"
        return 0
    fi
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    rm -f "$CARGO_TOML.bak"
    
    # Update Cargo.lock
    cargo check --quiet
    
    log_success "Version updated to $new_version"
}

update_changelog() {
    local version=$1
    local date=$(date '+%Y-%m-%d')
    
    log_info "Updating changelog for version $version"
    
    if [[ ! -f $CHANGELOG ]]; then
        log_info "Creating new changelog"
        if [[ $DRY_RUN == false ]]; then
            cat > "$CHANGELOG" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [$version] - $date

### Added
- Initial release of $PROJECT_NAME
- Cross-platform disk speed testing utility
- Support for Windows, macOS, and Linux
- Sequential and random read/write tests
- Memory copy performance testing
- Real-time progress reporting
- Comprehensive statistics (P5, P95, average speeds)

EOF
        fi
    else
        if [[ $DRY_RUN == false ]]; then
            # Add new version entry after [Unreleased]
            sed -i.bak "/## \[Unreleased\]/a\\
\\
## [$version] - $date\\
\\
### Added\\
- Add your changes here\\
\\
### Changed\\
- Add your changes here\\
\\
### Fixed\\
- Add your changes here\\
" "$CHANGELOG"
            rm -f "$CHANGELOG.bak"
        fi
    fi
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would update changelog with version $version"
    else
        log_success "Changelog updated"
        log_warning "Please edit $CHANGELOG to add release notes for version $version"
    fi
}

run_tests() {
    if [[ $SKIP_TESTS == true ]]; then
        log_warning "Skipping tests"
        return 0
    fi
    
    log_info "Running test suite..."
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would run: make ci-test"
        return 0
    fi
    
    if ! make ci-test; then
        log_error "Tests failed. Please fix issues before releasing."
        return 1
    fi
    
    log_success "All tests passed"
}

build_release() {
    if [[ $SKIP_BUILD == true ]]; then
        log_warning "Skipping build"
        return 0
    fi
    
    log_info "Building release binaries..."
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would run: ./scripts/build-cross-platform.sh --all"
        return 0
    fi
    
    if ! ./scripts/build-cross-platform.sh --all; then
        log_error "Build failed"
        return 1
    fi
    
    log_success "Release binaries built successfully"
}

create_git_tag() {
    local version=$1
    local tag="v$version"
    
    log_info "Creating git tag: $tag"
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would create tag: $tag"
        return 0
    fi
    
    # Commit version changes
    git add "$CARGO_TOML" Cargo.lock "$CHANGELOG" 2>/dev/null || true
    git commit -m "Release version $version"
    
    # Create annotated tag
    git tag -a "$tag" -m "Release version $version"
    
    log_success "Git tag created: $tag"
}

publish_crates_io() {
    local version=$1
    
    log_info "Publishing to crates.io..."
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would run: cargo publish"
        return 0
    fi
    
    # Dry run first
    if ! cargo publish --dry-run; then
        log_error "Cargo publish dry run failed"
        return 1
    fi
    
    # Actual publish
    read -p "Publish to crates.io? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if cargo publish; then
            log_success "Published to crates.io"
        else
            log_error "Failed to publish to crates.io"
            return 1
        fi
    else
        log_info "Skipped crates.io publishing"
    fi
}

publish_github() {
    local version=$1
    local tag="v$version"
    
    log_info "Publishing to GitHub..."
    
    if [[ $DRY_RUN == true ]]; then
        log_info "[DRY RUN] Would push tag and create GitHub release"
        return 0
    fi
    
    # Push tag
    git push origin "$tag"
    git push origin HEAD
    
    # Create GitHub release (requires gh CLI)
    if command -v gh &> /dev/null; then
        local release_notes="target/release-builds/RELEASE_NOTES.md"
        if [[ -f $release_notes ]]; then
            gh release create "$tag" \
                --title "Release $version" \
                --notes-file "$release_notes" \
                target/release-builds/*.tar.gz \
                target/release-builds/*.zip \
                target/release-builds/checksums.*
        else
            gh release create "$tag" \
                --title "Release $version" \
                --generate-notes \
                target/release-builds/*.tar.gz \
                target/release-builds/*.zip \
                target/release-builds/checksums.*
        fi
        log_success "GitHub release created"
    else
        log_warning "GitHub CLI not found. Please create release manually at:"
        log_info "https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases/new?tag=$tag"
    fi
}

prepare_release() {
    local version=$1
    
    log_info "Preparing release $version"
    
    validate_version "$version"
    check_git_status
    update_version "$version"
    update_changelog "$version"
    run_tests
    
    if [[ $DRY_RUN == false ]]; then
        log_success "Release $version prepared successfully"
        log_info "Next steps:"
        log_info "1. Review and edit $CHANGELOG"
        log_info "2. Run: $0 build"
        log_info "3. Run: $0 publish"
    fi
}

publish_release() {
    local version=$(get_current_version)
    
    log_info "Publishing release $version"
    
    check_git_status
    run_tests
    create_git_tag "$version"
    publish_crates_io "$version"
    publish_github "$version"
    
    if [[ $DRY_RUN == false ]]; then
        log_success "Release $version published successfully!"
    fi
}

full_release() {
    local version=$1
    
    log_info "Running full release process for version $version"
    
    prepare_release "$version"
    
    if [[ $DRY_RUN == false ]]; then
        read -p "Continue with build and publish? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Release process stopped after prepare phase"
            return 0
        fi
    fi
    
    build_release
    
    if [[ $DRY_RUN == false ]]; then
        read -p "Continue with publish? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Release process stopped after build phase"
            return 0
        fi
    fi
    
    publish_release
}

main() {
    # Default options
    DRY_RUN=false
    SKIP_TESTS=false
    SKIP_BUILD=false
    
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            prepare|build|publish|full)
                COMMAND=$1
                shift
                break
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Check if command was provided
    if [[ -z ${COMMAND:-} ]]; then
        log_error "No command specified"
        show_help
        exit 1
    fi
    
    # Execute command
    case $COMMAND in
        prepare)
            if [[ $# -lt 1 ]]; then
                log_error "Version required for prepare command"
                exit 1
            fi
            prepare_release "$1"
            ;;
        build)
            build_release
            ;;
        publish)
            publish_release
            ;;
        full)
            if [[ $# -lt 1 ]]; then
                log_error "Version required for full command"
                exit 1
            fi
            full_release "$1"
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"