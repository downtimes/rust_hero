use ffi::sdl::*;
use libc::{size_t, off_t};
use libc::{mmap, munmap, MAP_PRIVATE, MAP_FAILED, MAP_ANON};
use libc::{PROT_READ, PROT_WRITE};
use std::default::Default;
use std::ptr;

static mut is_white: bool = false;

const BYTES_PER_PIXEL: u32 = 4;
const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

pub struct BackBuffer {
    pixels: *mut c_void,
    size: size_t,
    texture: *mut SDL_Texture,
    texture_width: i32,
}

fn resize_texture(renderer: *mut SDL_Renderer, buffer: &mut BackBuffer,
                  width: i32, height: i32) {
    if buffer.pixels.is_not_null() {
        unsafe { munmap(buffer.pixels, buffer.size); }
        buffer.pixels = ptr::null_mut();
    }
    if buffer.texture.is_not_null() {
        unsafe { SDL_DestroyTexture(buffer.texture); }
        buffer.texture = ptr::null_mut();
    }

    buffer.texture =
        unsafe { SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888,
                                       SDL_TEXTUREACCESS_STREAMING,
                                       width as c_int, height as c_int) };
    buffer.texture_width = width;
    let size = (width * height * BYTES_PER_PIXEL as i32) as size_t;
    buffer.pixels = unsafe { mmap(ptr::null_mut(), size, PROT_READ | PROT_WRITE,
                                  MAP_PRIVATE | MAP_ANON, -1, 0) };
    if buffer.pixels == MAP_FAILED {
        panic!("The memory for the backbuffer could not be optained!");
    }
    buffer.size = size;
}

fn update_window(renderer: *mut SDL_Renderer,
                 buffer: &mut BackBuffer) {
    unsafe {
        SDL_UpdateTexture(buffer.texture, ptr::null(), buffer.pixels as *const _,
                          buffer.texture_width * BYTES_PER_PIXEL as i32);
        SDL_RenderCopy(renderer, buffer.texture, ptr::null(), ptr::null());
        SDL_RenderPresent(renderer);
    }
}

fn handle_event(event: &SDL_Event, buffer: &mut BackBuffer) -> bool {
    let mut result = false;

    let event_type = event._type();

    match event_type {
        SDL_QUIT => result = true,
        SDL_WINDOWEVENT => {
            let window_event = event.window_event();
            let renderer =
                get_renderer_from_window_id(window_event.windowID);
            match window_event.event {
                SDL_WINDOWEVENT_SIZE_CHANGED => {
                    resize_texture(renderer, buffer, window_event.data1,
                                   window_event.data2);
                },

                SDL_WINDOWEVENT_EXPOSED => {
                    update_window(renderer, buffer);
                },
                _ => (),
            }
        },
        _ => (),
    }
    fn get_renderer_from_window_id(window_id: u32) -> *mut SDL_Renderer {
        unsafe {
            let window = SDL_GetWindowFromID(window_id);
            SDL_GetRenderer(window)
        }
    }

    result
}

#[main]
fn main() {
    if unsafe { SDL_Init(SDL_INIT_VIDEO) != 0 } {
        panic!("SDL initialisation failed!");
    }

    let window_title = "Rust Hero".to_c_str();
    let window: *mut SDL_Window =
        unsafe { SDL_CreateWindow(window_title.as_ptr(),
                                  SDL_WINDOWPOS_UNDEFINED,
                                  SDL_WINDOWPOS_UNDEFINED,
                                  SCREEN_WIDTH, SCREEN_HEIGHT,
                                  SDL_WINDOW_RESIZABLE) };
    if window.is_not_null() {
        let renderer: *mut SDL_Renderer =
            unsafe { SDL_CreateRenderer(window, -1, 0) };

        let mut buffer = BackBuffer {
                           pixels: ptr::null_mut(),
                           size: 0,
                           texture: ptr::null_mut(),
                           texture_width: 0,
                        };
        
        loop {
            let mut event: SDL_Event = Default::default();
            unsafe { SDL_WaitEvent(&mut event); }
            if handle_event(&event, &mut buffer) {
                break;
            }
        }
    } else {
        //TODO: Window creation failed horribly just log it
    }

    unsafe { SDL_Quit(); }
}
