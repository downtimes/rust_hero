use std::env;

fn main() {
    match env::var("TARGET") {
        Ok(value) => {
            if value.contains("windows") {
                println!("cargo:rustc-link-lib=static=gdi32");
                println!("cargo:rustc-link-lib=static=winmm");
            } else if value.contains("linux") {
                println!("cargo:rustc-link-lib=dylib=SDL2");
            }
        },
        Err(_) => {},
    }
}
