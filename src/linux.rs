use libc::{readlink, mode_t, size_t, S_IRUSR, S_IWUSR};
use libc::{open, close, mmap, munmap, MAP_PRIVATE, MAP_FAILED, MAP_ANON};
use libc::{O_RDONLY, O_WRONLY, O_CREAT, PROT_READ, PROT_WRITE, fstat, stat};
use std::default::Default;
use std::ptr;
use std::mem;
use std::path::PathBuf;
use std::raw::Slice;
use std::ffi::CString;

use ffi::sdl::*;
use ffi::linux;
use common::util;
use common::{GetSoundSamplesT, UpdateAndRenderT, Input, SoundBuffer, Button};
use common::{VideoBuffer, GameMemory, ThreadContext};

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
        __unused: [0; 3],
    }
}

const S_IRGRP: mode_t = 32;
const S_IROTH: mode_t = 4;

#[allow(unused_imports)]
#[allow(dead_code)]
mod debug {

    use libc::{c_void, open, close, mmap, munmap, O_RDONLY, O_CREAT};
    use libc::{MAP_ANON, MAP_PRIVATE, MAP_FAILED, stat, write, read, fstat};
    use libc::{PROT_READ, PROT_WRITE, S_IRUSR, S_IWUSR};
    use libc::{O_WRONLY, mode_t, size_t};
    use std::ptr;
    use std::default::Default;
    use std::ffi::CString;

    use common::{ThreadContext, ReadFileResult};
    use common::util;


    pub fn platform_read_entire_file(context: &ThreadContext,
                                     filename: &str) -> Result<ReadFileResult, ()> {

        let mut result: Result<ReadFileResult, ()> = Err(());
        let name = CString::new(filename).unwrap();
        let handle =
            unsafe { open(name.as_ptr(), O_RDONLY, 0) };

        if handle != -1 {
            let mut file_stat: stat = super::get_stat_struct();
            if unsafe { fstat(handle, &mut file_stat) != -1 } {
                let size = util::safe_truncate_u64(file_stat.st_size as u64);
                let memory: *mut c_void =
                    unsafe { mmap(ptr::null_mut(),
                                  size as size_t, PROT_WRITE,
                                  MAP_PRIVATE | MAP_ANON, -1, 0)  };

                if !memory.is_null() {
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
                                                    .offset(bytes_read as isize) };
                    }

                    if bytes_to_read == 0 {
                        result = Ok(ReadFileResult {
                                        size: size,
                                        contents: memory,
                                      });
                    } else {
                        println!("Reading the file contents failed! ({})", filename);
                        platform_free_file_memory(context, memory, size);
                    }
                } else {
                    println!("Not enough memory could be optained!");
                }
            } else {
                println!("Fstat for the File was not successfull! ({})", filename);
            }
            unsafe { close(handle); }
        } else { 
            println!("The File could not be opened! ({})", filename);
        }

        result
    }

    pub fn platform_free_file_memory(_context: &ThreadContext,
                                     memory: *mut c_void,
                                     size: u32) {
        if !memory.is_null() {
            unsafe { munmap(memory, size as u64); }
        }
    }

    pub fn platform_write_entire_file(_context: &ThreadContext,
                                      filename: &str, size: u32,
                                      memory: *mut c_void) -> bool {
        let mut result = false;
        let name = CString::new(filename).unwrap();
        let handle =
            unsafe { open(name.as_ptr(), O_WRONLY | O_CREAT,
                          S_IRUSR | S_IWUSR | super::S_IRGRP | super::S_IROTH) };

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
                                            .offset(bytes_written as isize) };
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

const SAMPLES_PER_SECOND: i32 = 48000;
const BYTES_PER_SAMPLE: i32 = 4;

struct BackBuffer {
    pixels: *mut c_void,
    size: size_t,
    width: i32,
    height: i32,
    texture: *mut SDL_Texture,
    texture_width: i32,
}

struct SdlAudioRingBuffer {
    size: c_int,
    write_cursor: c_int,
    play_cursor: c_int,
    data: *mut u8,
}

struct SdlSoundOutput {
    running_sample_idx: i32,
    secondary_buffer_size: i32,
    latency_sample_count: i32,
}

struct Game {
    handle: *mut c_void,
    get_sound_samples: GetSoundSamplesT, 
    update_and_render: UpdateAndRenderT,
    write_time: linux::timespec,
}


//Stub functons if none of the game Code could be loaded! 
extern fn get_sound_samples_stub(_: &ThreadContext, _: &mut GameMemory, _: &mut SoundBuffer) { }
extern fn update_and_render_stub(_: &ThreadContext, _: &mut GameMemory, _: &Input, _: &mut VideoBuffer) { } 


fn get_last_write_time(file_path: &CString) -> linux::timespec {
    let mut file_stat = get_stat_struct();
    if unsafe { stat(file_path.as_ptr(), &mut file_stat) != -1 } {
        linux::timespec {
            tv_sec: file_stat.st_mtime,
            tv_nsec: file_stat.st_mtime_nsec,
        }
    } else {
        linux::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        }
    }
}

fn load_game_functions(game_so_name: &CString, temp_so_name: &CString) -> Game {

    let in_fd = unsafe { open(game_so_name.as_ptr(), O_RDONLY, 0) };
    let out_fd = unsafe { open(temp_so_name.as_ptr(), O_WRONLY | O_CREAT,
                          S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH) };

    let mut result = Game {
        handle: ptr::null_mut(),
        get_sound_samples: get_sound_samples_stub,
        update_and_render: update_and_render_stub,
        write_time: linux::timespec { tv_sec: 0 , tv_nsec: 0 },
    };
    
    let mut file_stat: stat = get_stat_struct();
    if unsafe { fstat(in_fd, &mut file_stat) != -1 } {
        let size = file_stat.st_size as u64;
        unsafe { 
            linux::sendfile(out_fd, in_fd, ptr::null_mut(), size); 
            result.write_time = get_last_write_time(temp_so_name);
            result.handle = linux::dlopen(temp_so_name.as_ptr(), linux::RTLD_LAZY);

            if !result.handle.is_null() {
                let get_sound_samples_name = CString::new("get_sound_samples").unwrap();
                let update_and_render_name = CString::new("update_and_render").unwrap();

                let get_sound_samples = linux::dlsym(result.handle, 
                                                     get_sound_samples_name.as_ptr());
                let update_and_render = linux::dlsym(result.handle, 
                                                     update_and_render_name.as_ptr());
                if !get_sound_samples.is_null() && !update_and_render.is_null() {
                    result.get_sound_samples = mem::transmute(get_sound_samples);
                    result.update_and_render = mem::transmute(update_and_render);
                }
            }
        }
        
    }

    unsafe { 
        close(in_fd);
        close(out_fd);
    }

    result
}

fn unload_game_functions(game: &mut Game) {
    if !game.handle.is_null() {
        unsafe { linux::dlclose(game.handle); }
        game.handle = ptr::null_mut();
    }
    game.get_sound_samples = get_sound_samples_stub;
    game.update_and_render = update_and_render_stub;
}

extern "C" fn audio_callback(user_data: *mut c_void, audio_data: *mut u8,
                             length: c_int) {
    let ring_buffer: &mut SdlAudioRingBuffer = 
        unsafe { mem::transmute(user_data as *mut SdlAudioRingBuffer) };

    let mut region_size = length;
    let mut region2_size = 0;

    if ring_buffer.play_cursor + length > ring_buffer.size {
        region_size = ring_buffer.size - ring_buffer.play_cursor;
        region2_size = length - region_size;
   }

    copy_contents(audio_data,
                  unsafe { ring_buffer.data
                            .offset(ring_buffer.play_cursor as isize) },
                  region_size);
    copy_contents(unsafe { audio_data.offset(region_size as isize) },
                  ring_buffer.data, region2_size);

    fn copy_contents(out: *mut u8, src: *mut u8, size: c_int) {
        let mut out = out;
        let mut src = src;
        for _ in 0..size {
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


#[allow(dead_code)]
fn process_game_controller_button(old_state: &Button,
                                  new_state: &mut Button, value: bool) {
    new_state.ended_down = value;
    if new_state.ended_down == old_state.ended_down {
        new_state.half_transitions += 1;
    }
}

#[allow(dead_code)]
fn process_game_controller_axis(value: i16, dead_zone: i16) -> f32 {
    let mut result = 0.0f32;

    if value < -dead_zone {
        result = (value + dead_zone) as f32 / (32768.0f32 - dead_zone as f32)
    } else if value > dead_zone {
        result = (value - dead_zone) as f32 / (32767.0f32 - dead_zone as f32)
    }

    result
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
    if !buffer.pixels.is_null() {
        unsafe { munmap(buffer.pixels, buffer.size); }
        buffer.pixels = ptr::null_mut();
    }
    if !buffer.texture.is_null() {
        unsafe { SDL_DestroyTexture(buffer.texture); }
        buffer.texture = ptr::null_mut();
    }

    buffer.texture =
        unsafe { SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888,
                                       SDL_TEXTUREACCESS_STREAMING,
                                       width as c_int, height as c_int) };
    buffer.texture_width = width;
    buffer.width = width;
    buffer.height = height;
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
                let alt_key_down = 
                    if (keyboard_event.keysym._mod & KMOD_ALT) != 0 {
                        true
                    } else {
                        false 
                    };
                match code {
                    SDLK_F4 => if alt_key_down { keep_running = false; },
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

fn open_controllers(controllers: &mut [*mut SDL_GameController; MAX_CONTROLLERS as usize]) -> i32 {
    let mut controller_num = 0;
    for controller_idx in 0..unsafe { SDL_NumJoysticks() } {
        if controller_num >= MAX_CONTROLLERS {
            break;
        }
        if unsafe { SDL_IsGameController(controller_idx) }
        == SDL_bool::SDL_TRUE {
            controllers[controller_num as usize] =
                unsafe { SDL_GameControllerOpen(controller_num) };
            controller_num += 1;
        }

    }
    controller_num
}

fn close_controllers(controllers: [*mut SDL_GameController; MAX_CONTROLLERS as usize], controller_num: i32) {
    for controller_idx in 0..controller_num {
        let controller = controllers[controller_idx as usize];
        if !controller.is_null() {
            unsafe { SDL_GameControllerClose(controller); }
        }
    }
}

fn get_exe_path() -> PathBuf {

    let name = CString::new("/proc/self/exe").unwrap();
    let mut buffer: [i8; linux::MAX_PATH] = [0; linux::MAX_PATH];
    let name_length = unsafe { 
        readlink(name.as_ptr(), buffer.as_mut_ptr(),
                           linux::MAX_PATH as size_t)
    };
    let result = unsafe { String::from_raw_parts(buffer.as_ptr() as *mut u8, 
                                                 name_length as usize,
                                                 (name_length + 10) as usize) };

    PathBuf::from(result.clone())
}

fn compare_file_time(time1: &linux::timespec, time2: &linux::timespec) -> i8 {
    if (time1.tv_sec == time2.tv_sec) && (time1.tv_nsec == time2.tv_nsec) {
        0
    } else if (time1.tv_sec > time2.tv_sec) || 
              ((time1.tv_sec == time2.tv_sec) && (time1.tv_nsec > time2.tv_nsec)) {
        -1
    } else {
        1
    }
}

fn get_seconds_elapsed(old_counter: u64, new_counter: u64, frequency: u64) -> f32 {
    (new_counter - old_counter) as f32 / frequency as f32
}

fn get_window_refresh_rate(window: *mut SDL_Window) -> c_int {
    let mut mode: SDL_DisplayMode = Default::default();
    let index = unsafe { SDL_GetWindowDisplayIndex(window) };

    if unsafe { SDL_GetDesktopDisplayMode(index, &mut mode) != 0} {
        60
    } else if mode.refresh_rate == 0 {
        60
    } else {
        mode.refresh_rate
    }
}

#[main]
fn main() {
    if unsafe { SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER 
                        | SDL_INIT_AUDIO) != 0 } {
        panic!("SDL initialisation failed!");
    }


    let window_title = CString::new("Rust Hero").unwrap();
    let window: *mut SDL_Window =
        unsafe { SDL_CreateWindow(window_title.as_ptr(),
                                  SDL_WINDOWPOS_UNDEFINED,
                                  SDL_WINDOWPOS_UNDEFINED,
                                  WINDOW_WIDTH, WINDOW_HEIGHT,
                                  SDL_WINDOW_RESIZABLE) };
    if !window.is_null() {
        let renderer: *mut SDL_Renderer =
            unsafe { SDL_CreateRenderer(window, -1, 0) };

        if renderer.is_null() {
            panic!("Couldn't create a render context for the window!");
        }

        let mut buffer = BackBuffer {
                           pixels: ptr::null_mut(),
                           size: 0,
                           width: 0,
                           height: 0,
                           texture: ptr::null_mut(),
                           texture_width: 0,
                        };
        resize_texture(renderer, &mut buffer, WINDOW_WIDTH, WINDOW_HEIGHT);

        let monitor_refresh_rate = get_window_refresh_rate(window); 
        let game_refresh_rate = monitor_refresh_rate / 2;
        let target_seconds_per_frame = 1.0 / game_refresh_rate as f32;

        let mut controllers =
            [ptr::null_mut::<SDL_GameController>(); MAX_CONTROLLERS as usize];

        let controller_num = open_controllers(&mut controllers);

        let sound_output = SdlSoundOutput {
            running_sample_idx: 0,
            secondary_buffer_size: SAMPLES_PER_SECOND * BYTES_PER_SAMPLE,
            latency_sample_count: SAMPLES_PER_SECOND / 15,
        };
        let mut ring_buffer = SdlAudioRingBuffer {
            size: 0,
            write_cursor: 0,
            play_cursor: 0,
            data: ptr::null_mut(),
        };
        init_audio(SAMPLES_PER_SECOND, sound_output.secondary_buffer_size,
                   &mut ring_buffer);

        unsafe { SDL_PauseAudio(0); }
        let mut sound_samples: &mut [i16] = unsafe { 
            //Allocation implicitly freed at the end of the execution
            let data = mmap(ptr::null_mut(),
                            (SAMPLES_PER_SECOND * BYTES_PER_SAMPLE) as u64, 
                            PROT_READ | PROT_WRITE,
                            MAP_PRIVATE | MAP_ANON, -1, 0);
            if data.is_null() {
                panic!("Couldn't allocate the resources for the Sound-Buffer!");
            }

            mem::transmute(
                Slice { data: data as *const i16,
                len: ((SAMPLES_PER_SECOND * BYTES_PER_SAMPLE) / 2) as usize})
        };

        let base_address = if cfg!(ndebug) { 0 } else { util::tera_bytes(2) };
        let permanent_store_size = util::mega_bytes(64);
        let transient_store_size = util::giga_bytes(1);
        let total_size = permanent_store_size + transient_store_size;

        let memory = unsafe { mmap(base_address as *mut c_void,
                                  total_size as u64, PROT_READ | PROT_WRITE,
                                  MAP_PRIVATE | MAP_ANON, -1, 0)  };
            
        if memory.is_null() { panic!("Memory for the Game could not be obtained!"); }

        let mut game_memory: GameMemory = 
            GameMemory {
                initialized: false,
                permanent: unsafe { 
                    mem::transmute( Slice { 
                        data: memory as *const u8, 
                        len: permanent_store_size
                    } 
                    ) 
                },
                transient: unsafe { 
                    mem::transmute( Slice { 
                        data: (memory as *const u8)
                            .offset(permanent_store_size as isize), 
                        len: transient_store_size
                    }
                    ) 
                },
                platform_read_entire_file: debug::platform_read_entire_file,
                platform_write_entire_file: debug::platform_write_entire_file,
                platform_free_file_memory: debug::platform_free_file_memory,
            };

        let frequency = unsafe { SDL_GetPerformanceFrequency() };
        
        let mut exe_dirname = get_exe_path();
        exe_dirname.pop();

        let mut game_so_path = exe_dirname.clone();
        game_so_path.push("libgame.so");

        let mut temp_so_path = exe_dirname.clone();
        temp_so_path.push("libgame_temp.so");

        let game_so_string = CString::new(game_so_path.to_str().unwrap()).unwrap();
        let temp_so_string = CString::new(temp_so_path.to_str().unwrap()).unwrap();
        
        let mut game = load_game_functions(&game_so_string, &temp_so_string);

        let thread_context = ThreadContext;

        let mut new_input: &mut Input = &mut Default::default();
        let mut old_input: &mut Input = &mut Default::default();

        let mut last_counter = unsafe { SDL_GetPerformanceCounter() };
        let mut running = true;
        while running {
            
            let new_write_time = get_last_write_time(&game_so_string);
            if compare_file_time(&game.write_time, &new_write_time) != 0 {
                unload_game_functions(&mut game);
                game = load_game_functions(&game_so_string, &temp_so_string);
            }

            let mut event: SDL_Event = Default::default();
            while unsafe { SDL_PollEvent(&mut event) } != 0 {
                running = handle_event(&event, &mut buffer);
            }

            for controller_idx in 0..controller_num as usize {
                if unsafe { SDL_GameControllerGetAttached(
                                                controllers[controller_idx]) }
                            == SDL_bool::SDL_TRUE {

                    let controller = controllers[controller_idx];
                    let _up = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP) == 1
                    };

                    let _down = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN) == 1
                    };
                    let _left = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT) == 1
                    };
                    let _right = unsafe {
                        SDL_GameControllerGetButton(
                            controller,
                            SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT) == 1
                    };


                    let _stick_x = unsafe {
                        SDL_GameControllerGetAxis(
                            controller,
                            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX) == 1
                    };
                    let _stick_y = unsafe {
                        SDL_GameControllerGetAxis(
                            controller,
                            SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY) == 1
                    };
                } else {
                    //The controller was plugged out so remove him from the
                    //controllers list
                    //If it is plugged in again query the SDL_ControllerEvent
                    //with the corresponding eventID
                }
            }

            let mut video_buf = VideoBuffer {
                memory: unsafe { mem::transmute(
                                Slice { data: buffer.pixels as *const u32, 
                                        len: (buffer.size/BYTES_PER_PIXEL as u64) as usize}
                                    )},
                width: buffer.width as usize,
                height: buffer.height as usize,
                pitch: (buffer.width*BYTES_PER_PIXEL as i32) as usize,
            };

            (game.update_and_render)(&thread_context,
                                     &mut game_memory,
                                     new_input,
                                     &mut video_buf);

            unsafe { SDL_LockAudio(); }
            let byte_to_lock = (sound_output.running_sample_idx * BYTES_PER_SAMPLE)
                                % sound_output.secondary_buffer_size;
            let target_cursor = (ring_buffer.play_cursor + 
                                  (sound_output.latency_sample_count *
                                   BYTES_PER_SAMPLE)) 
                                 % sound_output.secondary_buffer_size;
            let bytes_to_write = 
                if byte_to_lock > target_cursor {
                    sound_output.secondary_buffer_size - byte_to_lock + target_cursor
                } else {
                    target_cursor - byte_to_lock
                };
            unsafe { SDL_UnlockAudio(); }
            let mut sound_buffer = SoundBuffer {
                samples: &mut sound_samples[..bytes_to_write as usize/
                                                    mem::size_of::<i16>()],
                samples_per_second: SAMPLES_PER_SECOND as u32,
            };


            (game.get_sound_samples)(&thread_context,
                                     &mut game_memory,
                                     &mut sound_buffer);


            let time_elapsed = get_seconds_elapsed(last_counter,
                                   unsafe { SDL_GetPerformanceCounter()},
                                   frequency);
            if time_elapsed < target_seconds_per_frame {
                let sleep_time = ((target_seconds_per_frame - time_elapsed) * 1000.0) - 1.0;
                if sleep_time as u32 > 0 {
                    unsafe { SDL_Delay(sleep_time as u32); }
                }

                //busy loop for the rest of the last second
                while get_seconds_elapsed(last_counter,
                                          unsafe { SDL_GetPerformanceCounter() },
                                          frequency) < target_seconds_per_frame {
                }
            }

            last_counter = unsafe { SDL_GetPerformanceCounter() };


            update_window(renderer, &mut buffer);
            mem::swap(new_input, old_input);
        }

        close_controllers(controllers, controller_num);

    } else {
        //TODO: Window creation failed horribly just log it
    }

    unsafe { SDL_Quit(); }
}
