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

- **CI**: Tests on Windows, macOS, and Linux
- **Cross-compilation**: Builds for all targets
- **Release**: Automated release creation and publishing

### Workflow Files

- `.github/workflows/ci.yml`: Continuous integration
- `.github/workflows/release.yml`: Release automation
- `.github/workflows/coverage.yml`: Code coverage

## Troubleshooting

### Common Issues

#### Linker Errors
```bash
# Install appropriate linker
sudo apt-get install gcc-aarch64-linux-gnu

# Or use cross
cross build --target aarch64-unknown-linux-gnu --release
```

#### Missing Dependencies
```bash
# Run setup script
./scripts/setup-cross-compilation.sh --all
```

#### Docker Issues with Cross
```bash
# Update cross
cargo install cross --force

# Clear Docker cache
docker system prune -a
```

### Platform-Specific Issues

#### Windows
- Ensure Visual Studio Build Tools are installed
- For GNU targets, install MinGW-w64

#### macOS
- Install Xcode Command Line Tools
- For iOS, install full Xcode

#### Linux
- Install build-essential and cross-compilation toolchains
- For musl targets, use cross tool

#### Android
- Set ANDROID_NDK_HOME environment variable
- Ensure NDK version compatibility

#### iOS
- Build only on macOS
- Requires Xcode and iOS SDK

## Performance Considerations

### Binary Size

- Release builds use LTO and strip symbols
- musl targets produce smaller static binaries
- Consider using `strip` for further size reduction

### Optimization

```bash
# Maximum optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Size optimization
RUSTFLAGS="-C opt-level=s" cargo build --release
```

### Static vs Dynamic Linking

- Windows MSVC: Static linking by default
- Linux musl: Static linking
- Linux GNU: Dynamic linking (smaller binaries)
- macOS: Dynamic linking with system libraries

## Testing Cross-Platform Builds

### Local Testing

```bash
# Test specific target
cross test --target x86_64-unknown-linux-musl

# Test all targets (CI simulation)
./scripts/test-cross-platform.sh
```

### CI Testing

All targets are tested in CI:
- Unit tests on native platforms
- Cross-compilation verification
- Integration tests where possible

## Distribution

### Archive Formats

- **Windows**: ZIP archives
- **Unix-like**: tar.gz archives
- **Checksums**: SHA256 and MD5

### Naming Convention

```
disk-speed-test-{version}-{target}.{ext}
```

Examples:
- `disk-speed-test-1.2.3-x86_64-pc-windows-msvc.zip`
- `disk-speed-test-1.2.3-x86_64-unknown-linux-gnu.tar.gz`
- `disk-speed-test-1.2.3-aarch64-apple-darwin.tar.gz`

### Verification

```bash
# Verify checksums
sha256sum -c checksums.sha256
md5sum -c checksums.md5
```

## Contributing

When adding platform-specific code:

1. Use conditional compilation attributes
2. Update build configuration if needed
3. Test on target platform or CI
4. Update documentation

Example:
```rust
#[cfg(platform_windows)]
fn windows_specific_function() {
    // Windows implementation
}

#[cfg(platform_linux)]
fn linux_specific_function() {
    // Linux implementation
}
```

## Resources

- [Rust Cross-compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Cross Tool Documentation](https://github.com/cross-rs/cross)
- [Platform Support Tier List](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Android NDK Guide](https://developer.android.com/ndk/guides)
- [iOS Development with Rust](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-ios.html)