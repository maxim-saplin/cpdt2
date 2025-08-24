#!/bin/bash

# Setup script for cross-compilation dependencies
# Installs required tools and targets for building disk-speed-test on all platforms

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

check_rust_installation() {
    log_info "Checking Rust installation..."
    
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install Rust first:"
        log_info "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    local rust_version=$(rustc --version)
    log_success "Rust found: $rust_version"
}

install_cross() {
    log_info "Installing cross..."
    
    if command -v cross &> /dev/null; then
        local cross_version=$(cross --version)
        log_info "cross already installed: $cross_version"
    else
        cargo install cross
        log_success "cross installed successfully"
    fi
}

install_targets() {
    local targets=("$@")
    
    log_info "Installing Rust targets..."
    
    for target in "${targets[@]}"; do
        log_info "Installing target: $target"
        if rustup target add "$target"; then
            log_success "Target $target installed"
        else
            log_warning "Failed to install target: $target"
        fi
    done
}

install_additional_tools() {
    log_info "Installing additional development tools..."
    
    # Tools for development and CI
    local tools=(
        "cargo-llvm-cov"
        "cargo-audit"
        "cargo-nextest"
        "cargo-watch"
    )
    
    for tool in "${tools[@]}"; do
        if cargo install --list | grep -q "^$tool "; then
            log_info "$tool already installed"
        else
            log_info "Installing $tool..."
            if cargo install "$tool"; then
                log_success "$tool installed"
            else
                log_warning "Failed to install $tool"
            fi
        fi
    done
}

setup_platform_specific() {
    local os=$(uname -s)
    
    case $os in
        "Darwin")
            setup_macos
            ;;
        "Linux")
            setup_linux
            ;;
        "MINGW"*|"MSYS"*|"CYGWIN"*)
            setup_windows
            ;;
        *)
            log_warning "Unknown platform: $os"
            ;;
    esac
}

setup_macos() {
    log_info "Setting up macOS-specific dependencies..."
    
    # Check if Xcode command line tools are installed
    if ! xcode-select -p &> /dev/null; then
        log_warning "Xcode command line tools not found. Installing..."
        xcode-select --install
    else
        log_success "Xcode command line tools found"
    fi
    
    # Install additional targets for iOS development
    log_info "Installing iOS targets..."
    rustup target add aarch64-apple-ios
    rustup target add x86_64-apple-ios
    rustup target add aarch64-apple-ios-sim
}

setup_linux() {
    log_info "Setting up Linux-specific dependencies..."
    
    # Detect package manager and install dependencies
    if command -v apt-get &> /dev/null; then
        setup_debian_ubuntu
    elif command -v yum &> /dev/null; then
        setup_rhel_centos
    elif command -v pacman &> /dev/null; then
        setup_arch
    else
        log_warning "Unknown package manager. Please install cross-compilation tools manually."
    fi
}

setup_debian_ubuntu() {
    log_info "Installing dependencies for Debian/Ubuntu..."
    
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        gcc-mingw-w64 \
        gcc-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
        pkg-config \
        libssl-dev
}

setup_rhel_centos() {
    log_info "Installing dependencies for RHEL/CentOS..."
    
    sudo yum groupinstall -y "Development Tools"
    sudo yum install -y \
        gcc \
        gcc-c++ \
        mingw64-gcc \
        mingw64-gcc-c++ \
        openssl-devel
}

setup_arch() {
    log_info "Installing dependencies for Arch Linux..."
    
    sudo pacman -S --needed \
        base-devel \
        mingw-w64-gcc \
        aarch64-linux-gnu-gcc
}

setup_windows() {
    log_info "Setting up Windows-specific dependencies..."
    
    # Check if we're in a proper development environment
    if command -v gcc &> /dev/null; then
        log_success "GCC found in PATH"
    else
        log_warning "GCC not found. Please install MinGW-w64 or Visual Studio Build Tools"
    fi
}

verify_installation() {
    log_info "Verifying installation..."
    
    # Check cross
    if command -v cross &> /dev/null; then
        log_success "cross: $(cross --version)"
    else
        log_error "cross not found"
        return 1
    fi
    
    # Check some key targets
    local key_targets=(
        "x86_64-unknown-linux-gnu"
        "x86_64-pc-windows-gnu"
        "x86_64-apple-darwin"
    )
    
    for target in "${key_targets[@]}"; do
        if rustup target list --installed | grep -q "$target"; then
            log_success "Target $target: installed"
        else
            log_warning "Target $target: not installed"
        fi
    done
}

show_help() {
    cat << EOF
Cross-compilation setup script for disk-speed-test

Usage: $0 [options]

Options:
  --desktop-only    Install only desktop targets (default)
  --mobile-only     Install only mobile targets
  --all            Install all targets
  --tools-only     Install only additional tools
  --help           Show this help message

Examples:
  $0                    # Install desktop targets and tools
  $0 --all             # Install all targets and tools
  $0 --mobile-only     # Install only mobile targets
  $0 --tools-only      # Install only additional development tools
EOF
}

main() {
    log_info "Setting up cross-compilation environment for disk-speed-test"
    
    # Parse command line arguments
    INSTALL_DESKTOP=true
    INSTALL_MOBILE=false
    INSTALL_TOOLS=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --desktop-only)
                INSTALL_MOBILE=false
                shift
                ;;
            --mobile-only)
                INSTALL_DESKTOP=false
                INSTALL_MOBILE=true
                shift
                ;;
            --all)
                INSTALL_DESKTOP=true
                INSTALL_MOBILE=true
                shift
                ;;
            --tools-only)
                INSTALL_DESKTOP=false
                INSTALL_MOBILE=false
                INSTALL_TOOLS=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Run setup steps
    check_rust_installation
    
    if [[ $INSTALL_TOOLS == true ]]; then
        install_cross
        install_additional_tools
    fi
    
    if [[ $INSTALL_DESKTOP == true ]]; then
        install_targets "${DESKTOP_TARGETS[@]}"
    fi
    
    if [[ $INSTALL_MOBILE == true ]]; then
        install_targets "${MOBILE_TARGETS[@]}"
    fi
    
    setup_platform_specific
    verify_installation
    
    log_success "Cross-compilation setup completed!"
    log_info "You can now build for multiple platforms using:"
    log_info "  ./scripts/build-cross-platform.sh --all"
    log_info "  make build-cross-platform"
}

# Run main function with all arguments
main "$@"