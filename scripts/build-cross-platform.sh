#!/bin/bash

# Cross-platform build script for disk-speed-test
# Builds binaries for all supported platforms

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Host detection
HOST_OS=$(uname -s)

# Configuration
PROJECT_NAME="disk-speed-test"
VERSION=${VERSION:-$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')}
BUILD_DIR="target/release-builds"
DIST_DIR="dist"

# Supported targets
DESKTOP_TARGETS=(
    "x86_64-pc-windows-gnu"
    "x86_64-pc-windows-msvc"
    "aarch64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

MOBILE_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "i686-linux-android"
    "x86_64-linux-android"
    "aarch64-apple-ios"
    "x86_64-apple-ios"
)

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

check_dependencies() {
    log_info "Checking build dependencies..."
    
    # Check if cross is installed
    if ! command -v cross &> /dev/null; then
        log_warning "cross not found, installing..."
        cargo install cross
    fi
    
    # Check if jq is installed (for version extraction)
    if ! command -v jq &> /dev/null; then
        log_warning "jq not found, please install jq for version extraction"
        VERSION="0.1.0"
    fi
    
    log_success "Dependencies checked"
}

setup_build_environment() {
    log_info "Setting up build environment..."
    
    # Create build directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DIST_DIR"
    
    # Set build time
    export BUILD_TIME=$(date -u '+%Y-%m-%d %H:%M:%S UTC')
    
    # Set git hash if available
    if git rev-parse --git-dir > /dev/null 2>&1; then
        export GIT_HASH=$(git rev-parse --short HEAD)
    fi
    
    log_success "Build environment ready"
}

build_target() {
    local target=$1
    local target_type=$2
    
    log_info "Building for target: $target ($target_type)"
    
    # Determine binary extension
    local binary_ext=""
    if [[ $target == *"windows"* ]]; then
        binary_ext=".exe"
    fi
    
    # Decide build tool based on host and target compatibility
    local build_tool="cross"
    case "$HOST_OS" in
        Darwin)
            if [[ $target == *"apple-darwin"* ]]; then
                build_tool="cargo"
            else
                log_warning "Skipping unsupported target on macOS host: $target"
                return 0
            fi
            ;;
        Linux)
            if [[ $target == *"apple-darwin"* ]]; then
                log_warning "Skipping Apple targets on Linux host: $target"
                return 0
            fi
            if [[ $target == *"pc-windows-msvc"* ]]; then
                log_warning "Skipping Windows MSVC target on Linux host: $target"
                return 0
            fi
            build_tool="cross"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            if [[ $target == *"windows"* ]]; then
                build_tool="cargo"
            else
                log_warning "Skipping non-Windows target on Windows host: $target"
                return 0
            fi
            ;;
        *)
            log_warning "Unknown host OS ($HOST_OS). Attempting cross for $target"
            build_tool="cross"
            ;;
    esac

    # Build the target
    if { [[ "$build_tool" == "cargo" ]] && cargo build --target "$target" --release; } || \
       { [[ "$build_tool" == "cross" ]] && cross build --target "$target" --release; }; then
        # Copy binary to build directory
        local binary_name="${PROJECT_NAME}${binary_ext}"
        local target_binary="target/$target/release/$binary_name"
        local output_binary="$BUILD_DIR/${PROJECT_NAME}-${VERSION}-${target}${binary_ext}"
        
        if [[ -f "$target_binary" ]]; then
            cp "$target_binary" "$output_binary"
            
            # Create compressed archive
            create_archive "$target" "$output_binary"
            
            log_success "Built $target successfully"
            return 0
        else
            log_error "Binary not found: $target_binary"
            return 1
        fi
    else
        log_error "Failed to build $target"
        return 1
    fi
}

create_archive() {
    local target=$1
    local binary_path=$2
    local binary_name=$(basename "$binary_path")
    
    # Create archive directory
    local archive_dir="$BUILD_DIR/${PROJECT_NAME}-${VERSION}-${target}"
    mkdir -p "$archive_dir"
    
    # Copy binary and additional files
    cp "$binary_path" "$archive_dir/"
    cp README.md "$archive_dir/" 2>/dev/null || true
    cp LICENSE* "$archive_dir/" 2>/dev/null || true
    
    # Create version info file
    cat > "$archive_dir/VERSION.txt" << EOF
${PROJECT_NAME} ${VERSION}
Target: ${target}
Build Time: ${BUILD_TIME:-Unknown}
Git Hash: ${GIT_HASH:-Unknown}
EOF
    
    # Create archive based on platform
    cd "$BUILD_DIR"
    if [[ $target == *"windows"* ]]; then
        # Create ZIP for Windows
        if command -v zip &> /dev/null; then
            zip -r "${PROJECT_NAME}-${VERSION}-${target}.zip" "${PROJECT_NAME}-${VERSION}-${target}/"
        else
            tar -czf "${PROJECT_NAME}-${VERSION}-${target}.tar.gz" "${PROJECT_NAME}-${VERSION}-${target}/"
        fi
    else
        # Create tar.gz for Unix-like systems
        tar -czf "${PROJECT_NAME}-${VERSION}-${target}.tar.gz" "${PROJECT_NAME}-${VERSION}-${target}/"
    fi
    cd - > /dev/null
    
    # Clean up temporary directory
    rm -rf "$archive_dir"
}

build_desktop_targets() {
    log_info "Building desktop targets..."
    
    local success_count=0
    local total_count=${#DESKTOP_TARGETS[@]}
    
    for target in "${DESKTOP_TARGETS[@]}"; do
        if build_target "$target" "desktop"; then
            ((success_count++))
        fi
    done
    
    log_info "Desktop builds completed: $success_count/$total_count successful"
}

build_mobile_targets() {
    log_info "Building mobile targets..."
    
    local success_count=0
    local total_count=${#MOBILE_TARGETS[@]}
    
    for target in "${MOBILE_TARGETS[@]}"; do
        if build_target "$target" "mobile"; then
            ((success_count++))
        fi
    done
    
    log_info "Mobile builds completed: $success_count/$total_count successful"
}

create_checksums() {
    log_info "Creating checksums..."
    
    cd "$BUILD_DIR"
    
    # Create SHA256 checksums
    if command -v sha256sum &> /dev/null; then
        sha256sum *.tar.gz *.zip 2>/dev/null > checksums.sha256 || true
    elif command -v shasum &> /dev/null; then
        shasum -a 256 *.tar.gz *.zip 2>/dev/null > checksums.sha256 || true
    fi
    
    # Create MD5 checksums
    if command -v md5sum &> /dev/null; then
        md5sum *.tar.gz *.zip 2>/dev/null > checksums.md5 || true
    elif command -v md5 &> /dev/null; then
        md5 *.tar.gz *.zip 2>/dev/null > checksums.md5 || true
    fi
    
    cd - > /dev/null
    
    log_success "Checksums created"
}

generate_release_notes() {
    log_info "Generating release notes..."
    
    cat > "$BUILD_DIR/RELEASE_NOTES.md" << EOF
# ${PROJECT_NAME} ${VERSION}

## Build Information
- **Version**: ${VERSION}
- **Build Time**: ${BUILD_TIME:-Unknown}
- **Git Hash**: ${GIT_HASH:-Unknown}

## Supported Platforms

### Desktop Platforms
EOF
    
    for target in "${DESKTOP_TARGETS[@]}"; do
        echo "- $target" >> "$BUILD_DIR/RELEASE_NOTES.md"
    done
    
    cat >> "$BUILD_DIR/RELEASE_NOTES.md" << EOF

### Mobile Platforms
EOF
    
    for target in "${MOBILE_TARGETS[@]}"; do
        echo "- $target" >> "$BUILD_DIR/RELEASE_NOTES.md"
    done
    
    cat >> "$BUILD_DIR/RELEASE_NOTES.md" << EOF

## Installation

### Desktop
1. Download the appropriate archive for your platform
2. Extract the archive
3. Run the \`${PROJECT_NAME}\` binary

### Mobile
Mobile builds require integration with platform-specific applications.

## Verification

Verify the integrity of downloaded files using the provided checksums:

\`\`\`bash
# SHA256
sha256sum -c checksums.sha256

# MD5
md5sum -c checksums.md5
\`\`\`
EOF
    
    log_success "Release notes generated"
}

cleanup() {
    log_info "Cleaning up..."
    
    # Remove individual binaries, keep only archives
    find "$BUILD_DIR" -name "${PROJECT_NAME}-*" -type f ! -name "*.tar.gz" ! -name "*.zip" ! -name "*.md" ! -name "*.sha256" ! -name "*.md5" -delete
    
    log_success "Cleanup completed"
}

main() {
    log_info "Starting cross-platform build for $PROJECT_NAME v$VERSION"
    
    check_dependencies
    setup_build_environment
    
    # Parse command line arguments
    BUILD_DESKTOP=true
    BUILD_MOBILE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --desktop-only)
                BUILD_MOBILE=false
                shift
                ;;
            --mobile-only)
                BUILD_DESKTOP=false
                BUILD_MOBILE=true
                shift
                ;;
            --all)
                BUILD_DESKTOP=true
                BUILD_MOBILE=true
                shift
                ;;
            --help)
                echo "Usage: $0 [--desktop-only|--mobile-only|--all] [--help]"
                echo "  --desktop-only  Build only desktop targets (default)"
                echo "  --mobile-only   Build only mobile targets"
                echo "  --all          Build all targets"
                echo "  --help         Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Build targets
    if [[ $BUILD_DESKTOP == true ]]; then
        build_desktop_targets
    fi
    
    if [[ $BUILD_MOBILE == true ]]; then
        build_mobile_targets
    fi
    
    # Post-build tasks
    create_checksums
    generate_release_notes
    cleanup
    
    log_success "Cross-platform build completed!"
    log_info "Build artifacts available in: $BUILD_DIR"
    
    # List created files
    echo
    log_info "Created files:"
    ls -la "$BUILD_DIR"
}

# Run main function with all arguments
main "$@"