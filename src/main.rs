#![feature(asm)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
extern crate libc;

mod common;

#[cfg(target_os="windows")]
mod ffi;
#[cfg(target_os="windows")]
mod win32;

#[cfg(target_os="windows")]
fn main() {
    win32::winmain();
}

#[cfg(target_os="linux")]
mod ffi {
    pub mod sdl;
    pub mod linux;
}

#[cfg(target_os="linux")]
mod linux;

#[cfg(target_os="linux")]
fn main() {
    linux::linuxmain();
}

