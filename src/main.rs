#![feature(globs)]
#![feature(asm)]
#![allow(non_snake_case)]
extern crate libc;

mod game;

#[cfg(target_os="windows")]
mod ffi;
mod win32;
#[cfg(target_os="linux")]
mod linux;

