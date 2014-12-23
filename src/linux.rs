use ffi::sdl::*;
use libc::{size_t};
use libc::{mmap, munmap, MAP_PRIVATE, MAP_FAILED, MAP_ANON};
use libc::{PROT_READ, PROT_WRITE};
use std::default::Default;
use std::ptr;

const MAX_CONTROLLERS: c_int = 4;

const BYTES_PER_PIXEL: u32 = 4;
const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

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
    let mut keep_running = true;

    let event_type = event._type();

    match event_type {
        SDL_QUIT => keep_running = false,
        SDL_WINDOWEVENT => {
            let window_event = event.window_event();
            let renderer =
                get_renderer_from_window_id(window_event.windowID);
            match window_event.event {
                SDL_WINDOWEVENT_SIZE_CHANGED => {
                },

                SDL_WINDOWEVENT_EXPOSED => {
                    update_window(renderer, buffer);
                },
                _ => (),
            }
        },

        SDL_KEYDOWN
        | SDL_KEYUP => {
            let keyboard_event = event.keyboard_event();
            let code = keyboard_event.keysym.sym;
            let is_down = keyboard_event.state == SDL_PRESSED;
            let was_down =
                if (keyboard_event.state != SDL_PRESSED)
                    || (keyboard_event.repeat != 0) {
                    true
                } else {
                    false
                };

            if was_down != is_down {
                match code {
                    SDLK_w => println!("w"),
                    _ => (),
                }
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

    keep_running
}

#[main]
fn main() {
    if unsafe { SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER) != 0 } {
        panic!("SDL initialisation failed!");
    }

    let window_title = "Rust Hero".to_c_str();
    let window: *mut SDL_Window =
        unsafe { SDL_CreateWindow(window_title.as_ptr(),
                                  SDL_WINDOWPOS_UNDEFINED,
                                  SDL_WINDOWPOS_UNDEFINED,
                                  WINDOW_WIDTH, WINDOW_HEIGHT,
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
        resize_texture(renderer, &mut buffer, WINDOW_WIDTH, WINDOW_HEIGHT);

        let mut controllers =
            [ptr::null_mut::<SDL_GameController>(),
                                            ..MAX_CONTROLLERS as uint];

        let mut controller_num = 0;
        for controller_idx in range(0, unsafe { SDL_NumJoysticks() }) {
            if controller_num >= MAX_CONTROLLERS {
               break;
            }
            if unsafe { SDL_IsGameController(controller_idx) }
                        == SDL_bool::SDL_TRUE {
                controllers[controller_num as uint] =
                    unsafe { SDL_GameControllerOpen(controller_num) };
                controller_num += 1;
            }

        }

        let mut running = true;
        while running {
            let mut event: SDL_Event = Default::default();
            while unsafe { SDL_PollEvent(&mut event) } != 0 {
                running = handle_event(&event, &mut buffer);
            }

            for controller_idx in range(0, controller_num as uint) {
                if unsafe { SDL_GameControllerGetAttached(
                                                controllers[controller_idx]) }
                            == SDL_bool::SDL_TRUE {

                    let controller = controllers[controller_idx];
                    let _up = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_CONTROLLER_BUTTON_DPAD_UP) == 1
                    };

                    let _down = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_CONTROLLER_BUTTON_DPAD_DOWN) == 1
                    };
                    let _left = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_CONTROLLER_BUTTON_DPAD_LEFT) == 1
                    };
                    let _right = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_CONTROLLER_BUTTON_DPAD_RIGHT) == 1
                    };


                    let _stick_x = unsafe {
                        SDL_GameControllerGetAxis(
                            controller,
                            SDL_CONTROLLER_AXIS_LEFTX) == 1
                    };
                    let _stick_y = unsafe {
                        SDL_GameControllerGetAxis(
                            controller,
                            SDL_CONTROLLER_AXIS_LEFTY) == 1
                    };
                } else {
                    //The controller was plugged out so remove him from the
                    //controllers list
                    //If it is plugged in again query the SDL_ControllerEvent
                    //with the corresponding eventID
                }
            }


            update_window(renderer, &mut buffer);
        }

        for controller_idx in range(0, controller_num) {
            unsafe {
                SDL_GameControllerClose(controllers[controller_idx as uint]);
            }
        }
    } else {
        //TODO: Window creation failed horribly just log it
    }

    unsafe { SDL_Quit(); }
}
