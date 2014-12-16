#![feature(globs)]
#![feature(asm)]
#![allow(non_snake_case)]
extern crate libc;

mod common;

#[cfg(target_os="windows")]
mod ffi;
#[cfg(target_os="windows")]
mod win32;

#[cfg(target_os="linux")]
mod linux;

