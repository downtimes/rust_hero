#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub use libc::{c_int, c_char};
use std::default::Default;

pub const SDL_INIT_TIMER: u32 = 0x00000001;
pub const SDL_INIT_AUDIO: u32 = 0x00000010;
pub const SDL_INIT_VIDEO: u32 = 0x00000020;
pub const SDL_INIT_GAMECONTROLLER: u32 = 0x00002000;
pub const SDL_INIT_EVENTS: u32 = 0x000004000;

pub const SDL_WINDOWPOS_UNDEFINED: c_int = 0x1FFF0000;

pub const SDL_WINDOWEVENT_EXPOSED: u8 = 4;
pub const SDL_WINDOWEVENT_RESIZED: u8 = 6;

pub const SDL_WINDOW_RESIZABLE: u32 = 0x00000020;

pub const SDL_QUIT: u32 = 256;
pub const SDL_WINDOWEVENT: u32 = 512;

pub struct SDL_Window;

pub struct SDL_Renderer;

#[repr(C)]
pub struct SDL_WindowEvent {
    pub _type: u32,
    pub timestamp: u32,
    pub windowID: u32,
    pub event: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub data1: i32,
    pub data2: i32,
}

#[repr(C)]
pub struct SDL_Quit {
    pub _type: u32,
    pub timestamp: u32,
}

#[repr(C)]
pub struct SDL_Event {
    pub data: [u8, ..56],
}

impl Default for SDL_Event {
    fn default() -> SDL_Event {
        SDL_Event {
            data: [0, ..56],
        }
    }
}

impl SDL_Event {
    pub fn _type(&self) -> *const u32 {
        self.data.as_ptr() as *const _
    }

    pub fn window_event(&self) -> *const SDL_WindowEvent {
        self.data.as_ptr() as *const _
    }
}

extern "C" {
    pub fn SDL_Init(flags: u32) -> c_int;
    pub fn SDL_Quit();
    pub fn SDL_CreateWindow(title: *const c_char, x: c_int,
                            y: c_int, w: c_int, h: c_int,
                            flags: u32) -> *mut SDL_Window;
    pub fn SDL_WaitEvent(event: *mut SDL_Event) -> c_int;
    pub fn SDL_CreateRenderer(window: *mut SDL_Window,
                              index: c_int, flags: u32) -> *mut SDL_Renderer;
    pub fn SDL_SetRenderDrawColor(renderer: *mut SDL_Renderer,
                                 r: u8, g: u8, b: u8, a: u8)
                                    -> c_int;
    pub fn SDL_GetWindowFromID(id: u32) -> *mut SDL_Window;
    pub fn SDL_GetRenderer(window: *mut SDL_Window)
                                -> *mut SDL_Renderer;
    pub fn SDL_RenderClear(renderer: *mut SDL_Renderer) -> c_int;
    pub fn SDL_RenderPresent(renderer: *mut SDL_Renderer);
}
