use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    // Compile the C GPU context and rendering passes
    let mut build = cc::Build::new();
    build
        .file("src/core/gpu/gpucontext.c")
        .file("src/core/gpu/passes.c")
        .include("src/core/gpu")
        .warnings(false);
    
    // Platform-specific configuration
    match target_os.as_str() {
        "linux" => {
            build.define("__linux__", None);
            println!("cargo:rustc-link-lib=EGL");
            println!("cargo:rustc-link-lib=GLESv2");
        }
        "windows" => {
            build.define("_WIN32", None);
            println!("cargo:rustc-link-lib=opengl32");
            println!("cargo:rustc-link-lib=gdi32");
        }
        "macos" => {
            build.define("__APPLE__", None);
            println!("cargo:rustc-link-lib=framework=OpenGL");
        }
        _ => {}
    }
    
    build.compile("gpucontext");
    
    println!("cargo:rerun-if-changed=src/core/gpu/gpucontext.c");
    println!("cargo:rerun-if-changed=src/core/gpu/gpucontext.h");
    println!("cargo:rerun-if-changed=src/core/gpu/passes.c");
    println!("cargo:rerun-if-changed=src/core/gpu/passes.h");
}
