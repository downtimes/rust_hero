#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub use libc::{c_char, c_int, c_void, c_long, size_t, ssize_t, off_t};
pub use std::ffi::CString;

use std::ptr;
use std::ffi::CStr;


// Random value just to know how big our C-buffer needs to be. Enough
// in for our own usage.
pub const MAX_PATH: usize = 256;
pub const RTLD_LAZY: c_int = 1;

#[repr(C)]
pub struct timespec {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

pub fn dl_error() -> Option<CString> {
    unsafe {
        let val = dlerror();
        if val == ptr::null() {
            None
        } else {
            Some(CString::new(CStr::from_ptr(val).to_bytes()).unwrap())
        }
    }
}

extern "C" {
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
    pub fn sendfile(out_fd: c_int, in_fd: c_int, offset: *mut off_t, count: size_t) -> ssize_t;
}
