use std::env;

fn main() {
    // Get the target triple
    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TARGET");
    
    // Set platform-specific configuration flags
    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-cfg=platform_windows");
            configure_windows_build(&target);
        }
        "macos" => {
            println!("cargo:rustc-cfg=platform_macos");
            configure_macos_build(&target);
        }
        "linux" => {
            println!("cargo:rustc-cfg=platform_linux");
            configure_linux_build(&target);
        }
        "android" => {
            println!("cargo:rustc-cfg=platform_android");
            println!("cargo:rustc-cfg=platform_mobile");
            configure_android_build(&target);
        }
        "ios" => {
            println!("cargo:rustc-cfg=platform_ios");
            println!("cargo:rustc-cfg=platform_mobile");
            configure_ios_build(&target);
        }
        _ => {
            println!("cargo:warning=Unsupported target OS: {}", target_os);
        }
    }
    
    // Set architecture-specific flags
    match target_arch.as_str() {
        "x86_64" => println!("cargo:rustc-cfg=arch_x86_64"),
        "aarch64" => println!("cargo:rustc-cfg=arch_aarch64"),
        "arm" => println!("cargo:rustc-cfg=arch_arm"),
        "x86" => println!("cargo:rustc-cfg=arch_x86"),
        _ => println!("cargo:warning=Unsupported target architecture: {}", target_arch),
    }
    
    // Set desktop vs mobile flags
    match target_os.as_str() {
        "windows" | "macos" | "linux" => {
            println!("cargo:rustc-cfg=platform_desktop");
        }
        "android" | "ios" => {
            println!("cargo:rustc-cfg=platform_mobile");
        }
        _ => {}
    }
    
    // Print build information
    println!("cargo:rustc-env=BUILD_TARGET={}", target);
    println!("cargo:rustc-env=BUILD_TARGET_OS={}", target_os);
    println!("cargo:rustc-env=BUILD_TARGET_ARCH={}", target_arch);
    
    // Set version information
    if let Ok(git_hash) = env::var("GITHUB_SHA") {
        println!("cargo:rustc-env=GIT_HASH={}", &git_hash[..8]);
    } else if let Ok(output) = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=GIT_HASH={}", git_hash);
        }
    }
    
    if let Ok(build_time) = env::var("BUILD_TIME") {
        println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    } else {
        let build_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    }
}

fn configure_windows_build(target: &str) {
    println!("cargo:rustc-link-lib=kernel32");
    println!("cargo:rustc-link-lib=user32");
    
    if target.contains("msvc") {
        println!("cargo:rustc-cfg=windows_msvc");
        // Link statically for MSVC builds
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    } else if target.contains("gnu") {
        println!("cargo:rustc-cfg=windows_gnu");
        // MinGW-specific configuration
        println!("cargo:rustc-link-arg=-static-libgcc");
        println!("cargo:rustc-link-arg=-static-libstdc++");
    }
}

fn configure_macos_build(target: &str) {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=IOKit");
    
    if target.contains("aarch64") {
        println!("cargo:rustc-cfg=macos_arm64");
    } else if target.contains("x86_64") {
        println!("cargo:rustc-cfg=macos_x86_64");
    }
}

fn configure_linux_build(target: &str) {
    if target.contains("musl") {
        println!("cargo:rustc-cfg=linux_musl");
        // Static linking for musl builds
        println!("cargo:rustc-link-arg=-static");
    } else {
        println!("cargo:rustc-cfg=linux_gnu");
    }
    
    if target.contains("aarch64") {
        println!("cargo:rustc-cfg=linux_arm64");
    } else if target.contains("x86_64") {
        println!("cargo:rustc-cfg=linux_x86_64");
    }
}

fn configure_android_build(target: &str) {
    println!("cargo:rustc-link-lib=log");
    println!("cargo:rustc-link-lib=android");
    
    if target.contains("aarch64") {
        println!("cargo:rustc-cfg=android_arm64");
    } else if target.contains("armv7") {
        println!("cargo:rustc-cfg=android_arm");
    } else if target.contains("x86_64") {
        println!("cargo:rustc-cfg=android_x86_64");
    } else if target.contains("i686") {
        println!("cargo:rustc-cfg=android_x86");
    }
}

fn configure_ios_build(target: &str) {
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=UIKit");
    
    if target.contains("aarch64") {
        if target.contains("sim") {
            println!("cargo:rustc-cfg=ios_simulator");
        } else {
            println!("cargo:rustc-cfg=ios_device");
        }
        println!("cargo:rustc-cfg=ios_arm64");
    } else if target.contains("x86_64") {
        println!("cargo:rustc-cfg=ios_simulator");
        println!("cargo:rustc-cfg=ios_x86_64");
    }
}