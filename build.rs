use std::os;

fn main() {
    let target = os::getenv("TARGET").unwrap();
    if target.contains("windows") {
        println!("cargo:rustc-flags=-l gdi32");
    }
    
}
