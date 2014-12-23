use ffi::sdl::*;
use std::default::Default;

static mut is_white: bool = false;


fn handle_event(event: &SDL_Event) -> bool {

    let mut result = false;

    let event_type = unsafe { *event._type() };

    match event_type {
        SDL_QUIT => result = true,
        SDL_WINDOWEVENT => {
            let window_event = unsafe { *event.window_event() };
            match window_event.event {
                SDL_WINDOWEVENT_RESIZED
                | SDL_WINDOWEVENT_EXPOSED => {
                    let window = 
                        unsafe { SDL_GetWindowFromID(window_event.windowID) };
                    let renderer = unsafe { SDL_GetRenderer(window) };
                    unsafe {
                        if is_white  {
                            SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
                            is_white = false;
                        } else {
                            SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
                            is_white = true;
                        }
                        SDL_RenderClear(renderer);
                        SDL_RenderPresent(renderer);
                    }
                },
                _ => (),
            }
        },
        _ => (),
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
                                  640, 480, SDL_WINDOW_RESIZABLE) };
    if window.is_not_null() {
        let renderer: *mut SDL_Renderer = 
            unsafe { SDL_CreateRenderer(window, -1, 0) };

        loop {
            let mut event: SDL_Event = Default::default();
            unsafe { SDL_WaitEvent(&mut event); }
            if handle_event(&event) {
                break;
            }
        }
    } else {
        //TODO: Window creation failed horribly just log it
    }

    unsafe { SDL_Quit(); }
}
