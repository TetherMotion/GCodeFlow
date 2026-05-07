use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/gcode_ffi.cpp");
    println!("cargo:rerun-if-changed=src/gcode_ffi.hpp");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../Tether/build/lib/libtether_gcode.a");
    println!("cargo:rerun-if-changed=../Tether/build/lib/libtether_common.a");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = PathBuf::from(&manifest_dir).parent().unwrap().to_path_buf();

    // Include path should be Tether/include so that #include "tether/gcode/..." works
    let tether_include = project_root.join("Tether/include");
    let tether_build = project_root.join("Tether/build");
    let ffi_src = PathBuf::from(&manifest_dir).join("src/gcode_ffi.cpp");

    // Check if we can link against pre-built Tether libraries
    let use_prebuilt = tether_build.join("lib/libtether_gcode.a").exists()
        && tether_build.join("lib/libtether_common.a").exists();

    // Check if FFI is explicitly disabled via environment variable
    let ffi_disabled = env::var("GCODEFLOW_DISABLE_FFI").is_ok();

    if use_prebuilt && !ffi_disabled {
        // Link against pre-built component libraries
        println!("cargo:rustc-link-search=native={}", tether_build.join("lib").display());
        println!("cargo:rustc-link-lib=static=tether_gcode");
        println!("cargo:rustc-link-lib=static=tether_common");

        // Tell cxx to generate the Rust/C++ glue from the bridge definition
        cxx_build::bridge("src/gcode.rs")
            .file(&ffi_src)
            .include(&tether_include)
            .include(&PathBuf::from(&manifest_dir).join("src"))
            .flag_if_supported("-std=c++17")
            .flag_if_supported("-Wall")
            .flag_if_supported("-O2")
            .compile("gcode_cxx");

        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-cfg=tether_ffi");
    } else {
        // FFI is not available - skip C++ compilation
        if ffi_disabled {
            println!("cargo:warning=FFI explicitly disabled via GCODEFLOW_DISABLE_FFI");
        } else {
            println!("cargo:warning=Tether libraries not found. FFI disabled. Build will have limited functionality.");
            println!("cargo:warning=To enable FFI, build Tether first: cd ../Tether && mkdir -p build && cd build && cmake .. && make");
        }
        println!("cargo:rustc-cfg(tether_ffi_disabled)");
    }
}
