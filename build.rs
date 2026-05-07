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

    // Check if we can link against pre-built Tether libraries (required)
    let use_prebuilt = tether_build.join("lib/libtether_gcode.a").exists() 
        && tether_build.join("lib/libtether_common.a").exists();

    if use_prebuilt {
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
    } else {
        // Tether must be built first - provide clear error message
        panic!("Tether libraries not found. Please build Tether first: cd ../Tether && mkdir -p build && cd build && cmake .. && make");
    }

    println!("cargo:rustc-link-lib=stdc++");
}
