use std::env;

fn main() {
    match env::var("TARGET") {
        Ok(value) => {
            if value.contains("windows") {
                println!("cargo:rustc-link-lib=dylib=gdi32");
                println!("cargo:rustc-link-lib=dylib=winmm");
                println!("cargo:rustc-link-lib=dylib=user32");
            } else if value.contains("linux") {
                println!("cargo:rustc-link-lib=dylib=SDL2");
            }
        },
        Err(_) => {},
    }
}
