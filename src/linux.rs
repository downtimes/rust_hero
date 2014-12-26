use libc::{size_t};
use libc::{mmap, munmap, MAP_PRIVATE, MAP_FAILED, MAP_ANON};
use libc::{PROT_READ, PROT_WRITE};
use std::default::Default;
use std::ptr;

use ffi::sdl::*;

mod debug {

    use libc::{c_void, open, close, mmap, munmap, O_RDONLY, O_CREAT};
    use libc::{MAP_ANON, MAP_PRIVATE, MAP_FAILED, stat, write, read, fstat};
    use libc::{PROT_READ, PROT_WRITE, S_IRUSR, S_IWUSR};
    use libc::{O_WRONLY, mode_t};
    use std::ptr;
    use std::default::Default;

    use common::{ThreadContext, ReadFileResult};
    use common::util;

    const S_IRGRP: mode_t = 32;
    const S_IROTH: mode_t = 4;

    fn get_stat_struct() -> stat {
        stat {
            st_dev: 0,
            st_ino: 0,
            st_nlink: 0,
            st_mode: 0,
            st_uid: 0,
            st_gid: 0,
            __pad0: 0,
            st_rdev: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused: [0, ..3],
        }
    }

    pub fn platform_read_entire_file(context: &ThreadContext,
                                     filename: &str) -> Result<ReadFileResult, ()> {

        let mut result: Result<ReadFileResult, ()> = Err(());
        let name = filename.to_c_str();
        let handle =
            unsafe { open(name.as_ptr(), O_RDONLY, 0) };

        if handle != -1 {
            let mut file_stat: stat = get_stat_struct();
            if unsafe { fstat(handle, &mut file_stat) != -1 } {
                let size = util::safe_truncate_u64(file_stat.st_size as u64);
                let memory: *mut c_void =
                    unsafe { mmap(ptr::null_mut(),
                                  size as u64, PROT_READ,
                                  MAP_PRIVATE | MAP_ANON, -1, 0)  };

                if memory.is_not_null() {
                    let mut bytes_to_read = size;
                    let mut next_write_byte: *mut u8 = memory as *mut u8;

                    while bytes_to_read > 0 {
                        let bytes_read = unsafe { read(handle,
                                                       next_write_byte as *mut c_void,
                                                       bytes_to_read as u64) };
                        if bytes_read == -1 {
                            break;
                        }
                        bytes_to_read -= bytes_read as u32;
                        next_write_byte = unsafe { next_write_byte
                                                    .offset(bytes_read as int) };
                    }

                    if bytes_to_read == 0 {
                        result = Ok(ReadFileResult {
                                        size: size,
                                        contents: memory,
                                      });
                    } else {
                        platform_free_file_memory(context, memory, size);
                    }
                }
            }
            unsafe { close(handle); }
        }

        result
    }

    pub fn platform_free_file_memory(_context: &ThreadContext,
                                     memory: *mut c_void,
                                     size: u32) {
        if memory.is_not_null() {
            unsafe { munmap(memory, size as u64); }
        }
    }

    pub fn platform_write_entire_file(_context: &ThreadContext,
                                      filename: &str, size: u32,
                                      memory: *mut c_void) -> bool {
        let mut result = false;
        let name = filename.to_c_str();
        let handle =
            unsafe { open(name.as_ptr(), O_WRONLY | O_CREAT,
                          S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH) };

        if handle != -1 {
            let mut bytes_to_write = size;
            let mut byte_to_write: *mut u8 = memory as *mut u8;

            while bytes_to_write > 0 {
                let bytes_written = unsafe { write(handle,
                                                   byte_to_write as *const c_void,
                                                   bytes_to_write as u64) };
                if bytes_written == -1 {
                    break;
                }
                bytes_to_write -= bytes_written as u32;
                byte_to_write = unsafe { byte_to_write
                                            .offset(bytes_written as int) };
            }

            if bytes_to_write == 0 {
                result = true;
            }

            unsafe { close(handle); }
        }
        result
    }

}

const MAX_CONTROLLERS: c_int = 4;

const BYTES_PER_PIXEL: u32 = 4;
const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

struct BackBuffer {
    pixels: *mut c_void,
    size: size_t,
    texture: *mut SDL_Texture,
    texture_width: i32,
}

struct SdlAudioRingBuffer {
    size: c_int,
    write_cursor: c_int,
    play_cursor: c_int,
    data: *mut u8,
}

extern "C" fn audio_callback(user_data: *mut c_void, audio_data: *mut u8,
                             length: c_int) {
    let mut ring_buffer = unsafe { *(user_data as *mut SdlAudioRingBuffer) };

    let mut region_size = length;
    let mut region2_size = 0;

    if ring_buffer.play_cursor + length > ring_buffer.size {
        region_size = ring_buffer.size - ring_buffer.play_cursor;
        region2_size = length - region_size;
    }

    copy_contents(audio_data,
                  unsafe { ring_buffer.data
                            .offset(ring_buffer.play_cursor as int) },
                  region_size);
    copy_contents(unsafe { audio_data.offset(region_size as int) },
                  ring_buffer.data, region2_size);

    fn copy_contents(out: *mut u8, src: *mut u8, size: c_int) {
        let mut out = out;
        let mut src = src;
        for _ in range(0, size) {
            unsafe {
                *out = *src;
                out = out.offset(1);
                src = src.offset(1);
            }
        }
    }

    ring_buffer.play_cursor = (ring_buffer.play_cursor + length) % ring_buffer.size;
    ring_buffer.write_cursor = (ring_buffer.write_cursor + length) % ring_buffer.size;
}

fn init_audio(samples_per_second: i32, buffer_size: i32,
              ring_buffer: &mut SdlAudioRingBuffer) {
    let mut audio_settings = SDL_AudioSpec {
        freq: samples_per_second,
        format: AUDIO_S16LSB,
        channels: 2,
        silence: 0,
        samples: 512,
        padding: 0,
        size: 0,
        callback: audio_callback,
        userdata: ring_buffer as *mut _ as *mut c_void,
    };

    ring_buffer.size = buffer_size;
    ring_buffer.data = unsafe { mmap(ptr::null_mut(), buffer_size as u64,
                                     PROT_READ | PROT_WRITE,
                                     MAP_PRIVATE | MAP_ANON, -1, 0) as *mut u8};
    ring_buffer.play_cursor = 0;
    ring_buffer.write_cursor = 0;

    if ring_buffer.data.is_null() {
        panic!("Audio buffer could not be optained!");
    }

    unsafe { SDL_OpenAudio(&mut audio_settings, ptr::null_mut()); }

    if audio_settings.format != AUDIO_S16LSB {
        panic!("Audio buffer format can not be used!");
    }
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
