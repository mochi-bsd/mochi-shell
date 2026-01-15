use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    // Link against system libraries
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=EGL");
            println!("cargo:rustc-link-lib=GLESv2");
            println!("cargo:rustc-link-lib=gbm");
            println!("cargo:rustc-link-lib=drm");
            println!("cargo:rustc-link-lib=input");
            println!("cargo:rustc-link-lib=udev");
        }
        "freebsd" => {
            println!("cargo:rustc-link-lib=EGL");
            println!("cargo:rustc-link-lib=GLESv2");
            println!("cargo:rustc-link-lib=gbm");
            println!("cargo:rustc-link-lib=drm");
            println!("cargo:rustc-link-lib=input");
            println!("cargo:rustc-link-search=native=/usr/local/lib");
        }
        _ => {}
    }
}
