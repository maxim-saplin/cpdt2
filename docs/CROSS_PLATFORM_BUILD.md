# Cross-Platform Build Guide

This document describes how to build `disk-speed-test` for multiple platforms and architectures.

## Supported Platforms

### Desktop Platforms
- **Windows**: x86_64 (MSVC and GNU), aarch64 (ARM64)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Linux**: x86_64, aarch64 (GNU and musl libc)

### Mobile Platforms
- **Android**: aarch64, armv7, i686, x86_64
- **iOS**: aarch64 (device), x86_64 (simulator)

## Quick Start

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **Cross-compilation tools**: Run the setup script
   ```bash
   ./scripts/setup-cross-compilation.sh --all
   ```

### Build All Platforms

```bash
# Build for all supported platforms
./scripts/build-cross-platform.sh --all

# Build desktop platforms only (default)
./scripts/build-cross-platform.sh --desktop-only

# Build mobile platforms only
./scripts/build-cross-platform.sh --mobile-only
```

### Using Make

```bash
# Build all platforms
make build-cross-platform

# Build specific platform groups
make build-windows
make build-macos
make build-linux
make build-mobile
```

## Manual Cross-Compilation

### Install Targets

```bash
# Desktop targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# Mobile targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
```

### Install Cross

```bash
cargo install cross
```

### Build for Specific Targets

```bash
# Windows
cross build --target x86_64-pc-windows-gnu --release
cargo build --target x86_64-pc-windows-msvc --release

# macOS
cargo build --target x86_64-apple-darwin --release
cargo build --target aarch64-apple-darwin --release

# Linux
cargo build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-unknown-linux-musl --release

# Android
cross build --target aarch64-linux-android --release

# iOS
cargo build --target aarch64-apple-ios --release
```

## Platform-Specific Setup

### Windows

#### MSVC Toolchain
- Install Visual Studio Build Tools or Visual Studio Community
- Ensure Windows SDK is installed

#### GNU Toolchain
- Install MinGW-w64
- On Ubuntu/Debian: `sudo apt-get install gcc-mingw-w64`

### macOS

#### Prerequisites
- Xcode Command Line Tools: `xcode-select --install`
- For iOS builds: Full Xcode installation

#### Cross-compilation from Linux
```bash
# Install osxcross (advanced users)
# See: https://github.com/tpoechtrager/osxcross
```

### Linux

#### Dependencies
```bash
# Ubuntu/Debian
sudo apt-get install build-essential gcc-aarch64-linux-gnu

# RHEL/CentOS
sudo yum groupinstall "Development Tools"

# Arch Linux
sudo pacman -S base-devel aarch64-linux-gnu-gcc
```

#### musl libc
For static binaries with musl libc:
```bash
cross build --target x86_64-unknown-linux-musl --release
```

### Android

#### Prerequisites
1. Install Android NDK
2. Set environment variables:
   ```bash
   export ANDROID_NDK_HOME=/path/to/android-ndk
   export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
   ```

#### Build
```bash
cross build --target aarch64-linux-android --release
cross build --target armv7-linux-androideabi --release
```

### iOS

#### Prerequisites
- macOS with Xcode installed
- iOS SDK

#### Build
```bash
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release  # Simulator
```

## Build Configuration

### Cargo.toml Features

The project uses conditional compilation for platform-specific code:

```toml
[features]
windows-platform = []
macos-platform = []
linux-platform = []
android-platform = []
ios-platform = []
```

### Build Script

The `build.rs` script automatically:
- Detects target platform and architecture
- Sets appropriate compilation flags
- Configures platform-specific linking
- Embeds build metadata

### Cross.toml

Configuration for the `cross` tool:
- Docker images for each target
- Environment variables
- Linker configuration

## Release Process

### Automated Release

```bash
# Prepare release
./scripts/release.sh prepare 1.2.3

# Build all platforms
./scripts/release.sh build

# Publish release
./scripts/release.sh publish

# Full release process
./scripts/release.sh full 1.2.3
```

### Manual Release

```bash
# Update version
cargo set-version 1.2.3

# Build all platforms
./scripts/build-cross-platform.sh --all

# Create archives and checksums
# (handled automatically by build script)

# Tag and push
git tag v1.2.3
git push origin v1.2.3
```

## CI/CD Integration

### GitHub Actions

The project includes automated workflows:

- **Cross-compilation**: Builds for all targets
- **CI**: Tests on Windows, macOS, and Linux
- **Release**: Automated release creation and publishing