use std::ptr;
use std::path::PathBuf;
use std::slice;
use std::mem;
use std::i16;
use std::ffi::CString;
use std::iter::FromIterator;

use common::util;
use common::{Input, GameMemory, SoundBuffer, ControllerInput, Button, VideoBuffer};
use common::{ThreadContext, GetSoundSamplesT, UpdateAndRenderT};
use ffi::*;

#[cfg(not(ndebug))]
pub mod debug {
    use std::ptr;
    use std::ffi::CString;

    use ffi::*;
    use common::{ReadFileResult, ThreadContext};
    use common::util;

    #[derive(Copy, Clone)]
    pub struct SoundTimeMarker {
        pub flip_play_cursor: DWORD,
        pub flip_write_cursor: DWORD,

        pub output_play_cursor: DWORD,
        pub output_write_cursor: DWORD,

        pub output_location: DWORD,
        pub output_byte_count: DWORD,

        pub expected_frame_byte: DWORD,
    }

    impl Default for SoundTimeMarker {
        fn default() -> SoundTimeMarker {
            SoundTimeMarker {
                flip_play_cursor: 0,
                flip_write_cursor: 0,

                output_play_cursor: 0,
                output_write_cursor: 0,

                output_location: 0,
                output_byte_count: 0,

                expected_frame_byte: 0,
            }
        }
    }


    // TODO: make generic over the thing we want to load?
    // or just return a byteslice?
    pub fn platform_read_entire_file(context: &ThreadContext,
                                     filename: &str)
                                     -> Result<ReadFileResult, ()> {
        debug_assert!(filename.len() <= MAX_PATH);

        let mut result: Result<ReadFileResult, ()> = Err(());
        let name = CString::new(filename).unwrap();
        let handle = unsafe {
            CreateFileA(name.as_ptr(),
                        GENERIC_READ,
                        FILE_SHARE_READ,
                        ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL,
                        ptr::null_mut())
        };

        if handle != INVALID_HANDLE_VALUE {
            let mut file_size: i64 = 0;
            if unsafe { GetFileSizeEx(handle, &mut file_size) } != 0 {
                let memory: *mut c_void = unsafe {
                    VirtualAlloc(ptr::null_mut(),
                                 file_size as SIZE_T,
                                 MEM_RESERVE | MEM_COMMIT,
                                 PAGE_READWRITE)
                };

                if !memory.is_null() {
                    let size = util::safe_truncate_u64(file_size as u64);
                    let mut bytes_read = 0;
                    if (unsafe {
                        ReadFile(handle, memory, size, &mut bytes_read, ptr::null_mut())
                    } != 0) && (bytes_read == size) {

                        result = Ok(ReadFileResult {
                            size: size,
                            contents: memory as *mut u8,
                        });
                    } else {
                        platform_free_file_memory(context, memory as *mut u8, size);
                    }
                }
            }
            unsafe {
                CloseHandle(handle);
            }
        }

        result
    }

    pub fn platform_free_file_memory(_context: &ThreadContext, memory: *mut u8, _size: u32) {
        if !memory.is_null() {
            unsafe {
                VirtualFree(memory as *mut c_void, 0, MEM_RELEASE);
            }
        }
    }

    pub fn platform_write_entire_file(_context: &ThreadContext,
                                      filename: &str,
                                      size: DWORD,
                                      memory: *mut u8)
                                      -> bool {
        debug_assert!(filename.len() <= MAX_PATH);

        let mut result = false;
        let name = CString::new(filename).unwrap();
        let handle = unsafe {
            CreateFileA(name.as_ptr(),
                        GENERIC_WRITE,
                        0,
                        ptr::null_mut(),
                        CREATE_ALWAYS,
                        FILE_ATTRIBUTE_NORMAL,
                        ptr::null_mut())
        };

        if handle != INVALID_HANDLE_VALUE {
            let mut bytes_written = 0;
            if unsafe {
                WriteFile(handle,
                          memory as *mut c_void,
                          size,
                          &mut bytes_written,
                          ptr::null_mut())
            } != 0 {

                result = bytes_written == size;
            }
            unsafe {
                CloseHandle(handle);
            }
        }
        result
    }
}

// Graphics System constants
const BYTES_PER_PIXEL: c_int = 4;
const DEFAULT_MONITOR_REFRESH_RATE: usize = 60;

// Sound System constants
const CHANNELS: WORD = 2;
const BITS_PER_CHANNEL: WORD = 16;
const BYTES_PER_SAMPLE: DWORD = 4;
const SOUND_BYTES_PER_SECOND: DWORD = 48000 * BYTES_PER_SAMPLE;
// TODO: see how low we can go with this value reasonably

struct Game {
    handle: HMODULE,
    get_sound_samples: GetSoundSamplesT,
    update_and_render: UpdateAndRenderT,
    write_time: FILETIME,
}

// impl Game {
//    fn is_valid(&self) -> bool {
//        self.handle.is_not_null()
//    }
// }

struct SoundOutput {
    byte_index: DWORD,
    safety_bytes: DWORD,
    sound_buffer: *mut IDirectSoundBuffer,
}


impl SoundOutput {
    fn get_buffer_size(&self) -> DWORD {
        let mut dsbcaps: DSBCAPS = Default::default();
        dsbcaps.dwSize = mem::size_of::<DSBCAPS>() as DWORD;

        unsafe {
            ((*(*self.sound_buffer).lpVtbl).GetCaps)(self.sound_buffer, &mut dsbcaps);
        }

        dsbcaps.dwBufferBytes
    }
}

#[derive(PartialEq, Eq)]
enum ReplayState {
    Recording,
    Replaying,
    Nothing,
}

struct Replay {
    input_path: CString,
    input_file_handle: HANDLE,
    game_address: *mut c_void,
    memory: *mut c_void,
    memory_size: usize,
    state: ReplayState,
}


impl Replay {
    fn is_recording(&self) -> bool {
        self.state == ReplayState::Recording
    }

    fn is_replaying(&self) -> bool {
        self.state == ReplayState::Replaying
    }

    fn stop_recording(&mut self) {
        unsafe {
            CloseHandle(self.input_file_handle);
        }
        self.state = ReplayState::Nothing;
    }

    fn start_replay(&mut self) {
        self.input_file_handle = unsafe {
            CreateFileA(self.input_path.as_ptr(),
                        GENERIC_READ,
                        FILE_SHARE_READ,
                        ptr::null_mut(),
                        OPEN_EXISTING,
                        0,
                        ptr::null_mut())
        };
        unsafe {
            RtlCopyMemory(self.game_address,
                          self.memory as *const c_void,
                          self.memory_size as SIZE_T);
        }
        self.state = ReplayState::Replaying;
    }

    fn stop_replay(&mut self) {
        unsafe {
            CloseHandle(self.input_file_handle);
        }
        self.state = ReplayState::Nothing;
    }

    fn start_recording(&mut self) {
        self.input_file_handle = unsafe {
            CreateFileA(self.input_path.as_ptr(),
                        GENERIC_WRITE,
                        0,
                        ptr::null_mut(),
                        CREATE_ALWAYS,
                        0,
                        ptr::null_mut())
        };
        unsafe {
            RtlCopyMemory(self.memory,
                          self.game_address as *const c_void,
                          self.memory_size as SIZE_T);
        }
        self.state = ReplayState::Recording;
    }
}

struct Backbuffer {
    info: BITMAPINFO,
    memory: *mut c_void,
    height: c_int,
    width: c_int,
    pitch: c_int,
    size: c_int,
}

struct Window {
    handle: HWND,
    running: bool,
    pause: bool,
    timer_fine_resolution: bool,
    backbuffer: Backbuffer,
    debug_cursor: HCURSOR,
    win_pos: WINDOWPLACEMENT,
}

impl Window {
    fn toggle_fullscreen(&mut self) {
        unsafe {
            let style = GetWindowLongA(self.handle, GWL_STYLE);
            if (style & WS_OVERLAPPEDWINDOW) != 0 {
                let mut monitor_info = Default::default();
                let win_place = GetWindowPlacement(self.handle, &mut self.win_pos);
                let mon_inf = GetMonitorInfoA(MonitorFromWindow(self.handle,
                                                                MONITOR_DEFAULTTOPRIMARY),
                                              &mut monitor_info);
                if win_place != 0 && mon_inf != 0 {
                    SetWindowLongA(self.handle, GWL_STYLE, style & (!WS_OVERLAPPEDWINDOW));
                    SetWindowPos(self.handle,
                                 HWND_TOP,
                                 monitor_info.rcMonitor.left,
                                 monitor_info.rcMonitor.top,
                                 monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                                 monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
                                 SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
                }
            } else {
                SetWindowLongA(self.handle, GWL_STYLE, style | WS_OVERLAPPEDWINDOW);
                SetWindowPlacement(self.handle, &mut self.win_pos);
                SetWindowPos(self.handle,
                             ptr::null_mut(),
                             0,
                             0,
                             0,
                             0,
                             SWP_NOOWNERZORDER | SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE |
                             SWP_NOZORDER);
            }
        }
    }

    fn process_messages(&mut self, message: c_uint, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

        let mut res: LRESULT = 0;

        match message {
            WM_DESTROY => {
                self.running = false;
                unsafe {
                    PostQuitMessage(0);
                }
            }

            WM_CLOSE => self.running = false,

            WM_SETCURSOR => unsafe {
                if cfg!(ndebug) {
                    SetCursor(ptr::null_mut());
                } else {
                    SetCursor(self.debug_cursor);
                }
            },

            WM_SYSKEYDOWN |
            WM_SYSKEYUP |
            WM_KEYDOWN |
            WM_KEYUP => {
                debug_assert!(false,
                              "There sould be no key-messages inthe windows message callback!")
            }

            WM_PAINT => {
                let mut paint = Default::default();

                let context = unsafe { BeginPaint(self.handle, &mut paint) };
                if context.is_null() {
                    panic!("BeginPaint failed!");
                }

                let (width, height) = get_client_dimensions(self.handle).unwrap();
                unsafe {
                    blit_buffer_to_window(context, &self.backbuffer, width, height);
                    EndPaint(self.handle, &paint);
                }
            }

            _ => unsafe {
                res = DefWindowProcA(self.handle, message, wparam, lparam);
            },
        }

        res
    }
}

extern "system" fn process_messages(handle: HWND,
                                    message: c_uint,
                                    wparam: WPARAM,
                                    lparam: LPARAM)
                                    -> LRESULT {
    let mut res: LRESULT = 0;

    match message {
        // When creating the window we pass the window struct containing all our information
        // and registering it with windows
        WM_CREATE => unsafe {
            let ptr_to_window = (*(lparam as *const CREATESTRUCT)).lpCreateParams as LONG_PTR;
            SetWindowLongPtrA(handle, GWLP_USERDATA, ptr_to_window);
        },

        // For all the other messages we know we have a window struct registered
        // with windows. So we get it and dispatch to its message handleing
        _ => {
            unsafe {
                let window = GetWindowLongPtrA(handle, GWLP_USERDATA) as *mut Window;
                if !window.is_null() {
                    res = (*window).process_messages(message, wparam, lparam);
                    // During construction when there is still no struct registered we need to
                    // handle all the cases with the default behavior
                } else {
                    res = DefWindowProcA(handle, message, wparam, lparam);
                }
            }
        }
    }

    res
}

fn fill_sound_output(sound_output: &mut SoundOutput,
                     byte_to_lock: DWORD,
                     bytes_to_write: DWORD,
                     source: &SoundBuffer) {

    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0;
    let mut region2_size: DWORD = 0;

    let lock = unsafe {
        ((*(*sound_output.sound_buffer).lpVtbl).Lock)(sound_output.sound_buffer,
                                                      byte_to_lock,
                                                      bytes_to_write,
                                                      &mut region1,
                                                      &mut region1_size,
                                                      &mut region2,
                                                      &mut region2_size,
                                                      0 as DWORD)
    };

    if SUCCEEDED(lock) {

        debug_assert!(bytes_to_write == region1_size + region2_size);

        let (samples_1, samples_2) = source.samples
                                           .split_at(region1_size as usize / mem::size_of::<i16>());
        fill_region(region1, region1_size, sound_output, samples_1);
        fill_region(region2, region2_size, sound_output, samples_2);

        fn fill_region(region: *mut c_void,
                       region_size: DWORD,
                       sound_output: &mut SoundOutput,
                       source: &[i16]) {

            let out: &mut [i16] = unsafe {
                slice::from_raw_parts_mut(region as *mut i16,
                                          region_size as usize / mem::size_of::<i16>())
            };
            debug_assert!((region_size % BYTES_PER_SAMPLE) == 0);
            debug_assert!(out.len() == source.len());

            for (output, input) in out.iter_mut().zip(source.iter()) {
                *output = *input;
                sound_output.byte_index += 2;
            }
        }

        unsafe {
            ((*(*sound_output.sound_buffer).lpVtbl).Unlock)(sound_output.sound_buffer,
                                                            region1,
                                                            region1_size,
                                                            region2,
                                                            region2_size);
        }
    }
}

fn clear_sound_output(sound_output: &mut SoundOutput) {

    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0;
    let mut region2_size: DWORD = 0;

    let lock = unsafe {
        ((*(*sound_output.sound_buffer).lpVtbl).Lock)(sound_output.sound_buffer,
                                                      0,
                                                      sound_output.get_buffer_size(),
                                                      &mut region1,
                                                      &mut region1_size,
                                                      &mut region2,
                                                      &mut region2_size,
                                                      0 as DWORD)
    };

    if SUCCEEDED(lock) {


        fill_region(region1, region1_size);
        fill_region(region2, region2_size);

        fn fill_region(region: *mut c_void, region_size: DWORD) {
            let out: &mut [i32] = unsafe {
                slice::from_raw_parts_mut(region as *mut i32,
                                          (region_size / BYTES_PER_SAMPLE) as usize)
            };
            for sample in out.iter_mut() {
                *sample = 0;
            }
        }

        unsafe {
            ((*(*sound_output.sound_buffer).lpVtbl).Unlock)(sound_output.sound_buffer,
                                                            region1,
                                                            region1_size,
                                                            region2,
                                                            region2_size);
        }
    }
}

fn dsound_init(window: HWND,
               buffer_size_bytes: DWORD,
               samples_per_second: DWORD)
               -> Result<*mut IDirectSoundBuffer, ()> {
    let library_name = CString::new("dsound.dll").unwrap();
    let library = unsafe { LoadLibraryA(library_name.as_ptr()) };

    if !library.is_null() {
        let create_name = CString::new("DirectSoundCreate").unwrap();
        let ds_create = unsafe { GetProcAddress(library, create_name.as_ptr()) };
        if ds_create.is_null() {
            return Err(());
        }

        // We have DirectSound capabilities
        let DirectSoundCreate: DirectSoundCreate_t = unsafe { mem::transmute(ds_create) };
        let mut direct_sound: *mut IDirectSound = ptr::null_mut();
        if SUCCEEDED(DirectSoundCreate(ptr::null(), &mut direct_sound, ptr::null_mut())) {
            // Creating the primary buffer and setting our format
            let buffer_desc: DSBUFFERDESC = DSBUFFERDESC {
                dwSize: mem::size_of::<DSBUFFERDESC>() as DWORD,
                dwFlags: DSBCAPS_PRIMARYBUFFER,
                dwBufferBytes: 0 as DWORD,
                dwReserved: 0 as DWORD,
                lpwfxFormat: ptr::null_mut(),
                guid: Default::default(),
            };
            let mut primary_buffer: *mut IDirectSoundBuffer = ptr::null_mut();
            // Holy shit: it's the syntax from hell!
            unsafe {
                ((*(*direct_sound).lpVtbl).SetCooperativeLevel)(direct_sound,
                                                                window,
                                                                DSSCL_PRIORITY);
                ((*(*direct_sound).lpVtbl).CreateSoundBuffer)(direct_sound,
                                                              &buffer_desc,
                                                              &mut primary_buffer,
                                                              ptr::null_mut());
            }

            let block_align = (CHANNELS * BITS_PER_CHANNEL) / 8;
            let mut wave_format: WAVEFORMATEX = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM,
                nChannels: CHANNELS,
                nSamplesPerSec: samples_per_second,
                nAvgBytesPerSec: samples_per_second * (block_align as DWORD),
                nBlockAlign: block_align as WORD,
                wBitsPerSample: BITS_PER_CHANNEL,
                cbSize: 0 as WORD,
            };
            unsafe {
                ((*(*primary_buffer).lpVtbl).SetFormat)(primary_buffer, &wave_format);
            }

            // Creating our secondary buffer
            let buffer_desc_secondary: DSBUFFERDESC = DSBUFFERDESC {
                dwSize: mem::size_of::<DSBUFFERDESC>() as DWORD,
                dwFlags: DSBCAPS_GETCURRENTPOSITION2,
                dwBufferBytes: buffer_size_bytes,
                dwReserved: 0,
                lpwfxFormat: &mut wave_format,
                guid: Default::default(),
            };
            let mut secondary_buffer: *mut IDirectSoundBuffer = ptr::null_mut();
            unsafe {
                ((*(*direct_sound).lpVtbl).CreateSoundBuffer)(direct_sound,
                                                              &buffer_desc_secondary,
                                                              &mut secondary_buffer,
                                                              ptr::null_mut());
            }

            return Ok(secondary_buffer);
        }
    }

    Err(())
}

// Stub functions if none of the XInput libraries could be loaded!
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD {
    ERROR_DEVICE_NOT_CONNECTED
}
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD {
    ERROR_DEVICE_NOT_CONNECTED
}


// Stub functons if none of the game Code could be loaded!
extern "C" fn get_sound_samples_stub(_: &ThreadContext, _: &mut GameMemory, _: &mut SoundBuffer) {}
extern "C" fn update_and_render_stub(_: &ThreadContext,
                                     _: &mut GameMemory,
                                     _: &Input,
                                     _: &mut VideoBuffer) {
}


fn get_last_write_time(file_name: &CString) -> Result<FILETIME, ()> {
    let mut file_info: WIN32_FILE_ATTRIBUTE_DATA = Default::default();
    let mut res: Result<FILETIME, ()> = Err(());
    unsafe {
        if GetFileAttributesExA(file_name.as_ptr(),
                                GET_FILEEX_INFO_LEVELS::GET_FILE_EX_INFO_STANDARD,
                                (&mut file_info) as *mut _ as *mut c_void) != 0 {
            res = Ok(file_info.ftLastWriteTime);
        }
    }
    res
}

fn load_game_functions(game_dll_name: &CString, temp_dll_name: &CString) -> Game {

    unsafe {
        CopyFileA(game_dll_name.as_ptr(), temp_dll_name.as_ptr(), FALSE);
    }

    let filetime = get_last_write_time(game_dll_name).unwrap_or(FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    });

    let mut result = Game {
        handle: ptr::null_mut(),
        get_sound_samples: get_sound_samples_stub,
        update_and_render: update_and_render_stub,
        write_time: filetime,
    };

    result.handle = unsafe { LoadLibraryA(temp_dll_name.as_ptr()) };

    if !result.handle.is_null() {
        let get_sound_samples_name = CString::new("get_sound_samples").unwrap();
        let update_and_render_name = CString::new("update_and_render").unwrap();

        let get_sound_samples = unsafe {
            GetProcAddress(result.handle, get_sound_samples_name.as_ptr())
        };
        let update_and_render = unsafe {
            GetProcAddress(result.handle, update_and_render_name.as_ptr())
        };

        if !get_sound_samples.is_null() && !update_and_render.is_null() {
            unsafe {
                result.get_sound_samples = mem::transmute(get_sound_samples);
                result.update_and_render = mem::transmute(update_and_render);
            }
        }
    }
    result
}


fn unload_game_functions(game: &mut Game) {
    if !game.handle.is_null() {
        unsafe {
            FreeLibrary(game.handle);
        }
        game.handle = ptr::null_mut();
    }
    game.get_sound_samples = get_sound_samples_stub;
    game.update_and_render = update_and_render_stub;
}

fn load_xinput_functions() -> (XInputGetState_t, XInputSetState_t) {

    let xlib_first_name = CString::new("xinput1_4.dll").unwrap();
    let xlib_second_name = CString::new("xinput1_3.dll").unwrap();
    let xlib_third_name = CString::new("xinput9_1_0.dll").unwrap();

    let mut module = unsafe { LoadLibraryA(xlib_first_name.as_ptr()) };

    if module.is_null() {
        module = unsafe { LoadLibraryA(xlib_second_name.as_ptr()) };
    }
    if module.is_null() {
        module = unsafe { LoadLibraryA(xlib_third_name.as_ptr()) };
    }

    if !module.is_null() {
        let get_state_name = CString::new("XInputGetState").unwrap();
        let set_state_name = CString::new("XInputSetState").unwrap();

        let xinput_get_state = unsafe { GetProcAddress(module, get_state_name.as_ptr()) };
        let xinput_set_state = unsafe { GetProcAddress(module, set_state_name.as_ptr()) };

        unsafe {
            (mem::transmute(xinput_get_state),
             mem::transmute(xinput_set_state))
        }
    } else {
        (xinput_get_state_stub, xinput_set_state_stub)
    }
}

fn process_xinput(XInputGetState: XInputGetState_t,
                  new_controllers: &mut [ControllerInput],
                  old_controllers: &[ControllerInput]) {

    debug_assert!(XUSER_MAX_COUNT >= new_controllers.len() as u32);

    for (index, controller) in new_controllers.iter_mut().enumerate() {
        let mut state = Default::default();

        let res: u32 = XInputGetState(index as u32, &mut state);
        match res {
            // Case the Controller is connected and we got data
            ERROR_SUCCESS => {
                let old_controller = &old_controllers[index];
                controller.is_connected = true;

                let gamepad = &state.Gamepad;

                fn process_xinput_stick(value: SHORT, dead_zone: SHORT) -> f32 {
                    if value < -dead_zone {
                        -((value + dead_zone) as f32) / (i16::MIN + dead_zone) as f32
                    } else if value > dead_zone {
                        (value - dead_zone) as f32 / (i16::MAX - dead_zone) as f32
                    } else {
                        0.0
                    }
                }

                let mut xvalue = process_xinput_stick(gamepad.sThumbLX,
                                                      XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);
                let mut yvalue = process_xinput_stick(gamepad.sThumbLY,
                                                      XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);

                let dpad_up = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_UP) != 0;
                let dpad_down = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN) != 0;
                let dpad_left = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT) != 0;
                let dpad_right = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT) != 0;

                // If the DPAD was not used default to analog input with
                // the controllers
                if !(dpad_up || dpad_down || dpad_left || dpad_right) {
                    controller.average_x = Some(xvalue);
                    controller.average_y = Some(yvalue);
                } else {
                    controller.average_x = None;
                    controller.average_y = None;
                    if dpad_up {
                        yvalue = 1.0;
                    } else if dpad_down {
                        yvalue = -1.0;
                    }
                    if dpad_left {
                        xvalue = -1.0;
                    } else if dpad_right {
                        xvalue = 1.0;
                    }
                }

                let threshhold = 0.5;
                let up_fake = if yvalue > threshhold {
                    1
                } else {
                    0
                };
                let down_fake = if yvalue < -threshhold {
                    1
                } else {
                    0
                };
                let left_fake = if xvalue < -threshhold {
                    1
                } else {
                    0
                };
                let right_fake = if xvalue > threshhold {
                    1
                } else {
                    0
                };
                process_xinput_button(up_fake, &old_controller.move_up, &mut controller.move_up, 1);
                process_xinput_button(down_fake,
                                      &old_controller.move_down,
                                      &mut controller.move_down,
                                      1);
                process_xinput_button(left_fake,
                                      &old_controller.move_left,
                                      &mut controller.move_left,
                                      1);
                process_xinput_button(right_fake,
                                      &old_controller.move_right,
                                      &mut controller.move_right,
                                      1);

                process_xinput_button(gamepad.wButtons,
                                      &old_controller.left_shoulder,
                                      &mut controller.left_shoulder,
                                      XINPUT_GAMEPAD_LEFT_SHOULDER);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.right_shoulder,
                                      &mut controller.right_shoulder,
                                      XINPUT_GAMEPAD_RIGHT_SHOULDER);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.action_up,
                                      &mut controller.action_up,
                                      XINPUT_GAMEPAD_Y);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.action_down,
                                      &mut controller.action_down,
                                      XINPUT_GAMEPAD_A);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.action_left,
                                      &mut controller.action_left,
                                      XINPUT_GAMEPAD_X);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.action_right,
                                      &mut controller.action_right,
                                      XINPUT_GAMEPAD_B);

                process_xinput_button(gamepad.wButtons,
                                      &old_controller.start,
                                      &mut controller.start,
                                      XINPUT_GAMEPAD_START);
                process_xinput_button(gamepad.wButtons,
                                      &old_controller.back,
                                      &mut controller.back,
                                      XINPUT_GAMEPAD_BACK);
            }

            // Case the Controller is not connected
            ERROR_DEVICE_NOT_CONNECTED => controller.is_connected = false,

            // Some arbitrary Error was found
            _ => (),
        }
    }
}

fn get_client_dimensions(window: HWND) -> Result<(c_int, c_int), &'static str> {
    let mut client_rect = Default::default();
    let res = unsafe { GetClientRect(window, &mut client_rect) };
    match res {
        0 => Err("Client Rect not optainable"),
        _ => {
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;
            Ok((width, height))
        }
    }
}


fn resize_dib_section(buffer: &mut Backbuffer, width: c_int, height: c_int) {
    if !buffer.memory.is_null() {
        unsafe {
            if VirtualFree(buffer.memory, 0 as SIZE_T, MEM_RELEASE) == 0 {
                panic!("VirtualFree ran sizeo an error");
            }
        }
    }

    // Height is negative to denote a top to bottom Bitmap for StretchDIBits
    buffer.info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as DWORD,
            biWidth: width as LONG,
            biHeight: -height as LONG,
            biPlanes: 1 as WORD,
            biBitCount: (BYTES_PER_PIXEL * 8) as WORD,
            biCompression: BI_RGB,
            biSizeImage: 0 as DWORD,
            biXPelsPerMeter: 0 as LONG,
            biYPelsPerMeter: 0 as LONG,
            biClrUsed: 0 as DWORD,
            biClrImportant: 0 as DWORD,
        },
        bmiColors: 0 as *mut RGBQUAD,
    };
    buffer.width = width;
    buffer.height = height;
    buffer.pitch = width * BYTES_PER_PIXEL;
    buffer.size = buffer.width * buffer.height * BYTES_PER_PIXEL;

    unsafe {
        // The last created buffer is implicitly destroyed at end of execution
        buffer.memory = VirtualAlloc(ptr::null_mut(),
                                     buffer.size as SIZE_T,
                                     MEM_RESERVE | MEM_COMMIT,
                                     PAGE_READWRITE);
        if buffer.memory.is_null() {
            panic!("No memory could be allocated by VirtualAlloc");
        }
    }
}

fn blit_buffer_to_window(context: HDC,
                         buffer: &Backbuffer,
                         client_width: c_int,
                         client_height: c_int) {

    let offset_x = 10;
    let offset_y = 10;

    unsafe {
        PatBlt(context, 0, 0, client_width, offset_y, BLACKNESS);
        PatBlt(context,
               0,
               offset_y + buffer.height,
               client_width,
               client_height,
               BLACKNESS);
        PatBlt(context, 0, 0, offset_x, client_height, BLACKNESS);
        PatBlt(context,
               offset_x + buffer.width,
               0,
               client_width,
               client_height,
               BLACKNESS);

        StretchDIBits(context,
                      offset_x,
                      offset_y,
                      buffer.width,
                      buffer.height,
                      0,
                      0,
                      buffer.width,
                      buffer.height,
                      buffer.memory as *const c_void,
                      &buffer.info,
                      DIB_RGB_COLORS,
                      SRCCOPY)
    };
}

fn process_xinput_button(xinput_button_state: WORD,
                         old_state: &Button,
                         new_state: &mut Button,
                         button_bit: WORD) {

    new_state.ended_down = (xinput_button_state & button_bit) == button_bit;
    new_state.half_transitions = if old_state.ended_down != new_state.ended_down {
        1
    } else {
        0
    };
}

fn process_pending_messages(window: &mut Window,
                            keyboard_controller: &mut ControllerInput,
                            replay: &mut Replay) {
    let mut msg = Default::default();
    // Process the Message Queue
    while unsafe {
        PeekMessageA(&mut msg,
                     0 as HWND,
                     0 as c_uint,
                     0 as c_uint,
                     PM_REMOVE as c_uint)
    } != 0 {
        match msg.message {
            WM_QUIT => window.running = false,

            WM_SYSKEYDOWN |
            WM_SYSKEYUP |
            WM_KEYDOWN |
            WM_KEYUP => {
                let vk_code = msg.wparam as u8;
                let was_down = (msg.lparam & (1 << 30)) != 0;
                let is_down = (msg.lparam & (1 << 31)) == 0;
                let alt_key_down = (msg.lparam & (1 << 29)) != 0;

                if was_down != is_down {
                    match vk_code {
                        VK_UP => {
                            process_keyboard_message(&mut keyboard_controller.action_up, is_down)
                        }
                        VK_DOWN => {
                            process_keyboard_message(&mut keyboard_controller.action_down, is_down)
                        }
                        VK_LEFT => {
                            process_keyboard_message(&mut keyboard_controller.action_left, is_down)
                        }
                        VK_RIGHT => {
                            process_keyboard_message(&mut keyboard_controller.action_right, is_down)
                        }
                        b'W' => process_keyboard_message(&mut keyboard_controller.move_up, is_down),
                        b'A' => {
                            process_keyboard_message(&mut keyboard_controller.move_left, is_down)
                        }
                        b'S' => {
                            process_keyboard_message(&mut keyboard_controller.move_down, is_down)
                        }
                        b'D' => {
                            process_keyboard_message(&mut keyboard_controller.move_right, is_down)
                        }
                        b'E' => {
                            process_keyboard_message(&mut keyboard_controller.right_shoulder,
                                                     is_down)
                        }
                        b'Q' => {
                            process_keyboard_message(&mut keyboard_controller.left_shoulder,
                                                     is_down)
                        }
                        b'P' => {
                            if is_down {
                                window.pause = !window.pause;
                            }
                        }
                        VK_ESCAPE => {
                            process_keyboard_message(&mut keyboard_controller.back, is_down)
                        }
                        VK_SPACE => {
                            process_keyboard_message(&mut keyboard_controller.start, is_down)
                        }
                        VK_F4 => {
                            if alt_key_down {
                                window.running = false;
                            }
                        }

                        VK_RETURN => {
                            if alt_key_down && is_down {
                                window.toggle_fullscreen();
                            }
                        }

                        b'L' => {
                            if is_down {
                                if replay.is_recording() {
                                    replay.stop_recording();
                                    replay.start_replay();
                                } else if replay.is_replaying() {
                                    replay.stop_replay();
                                } else {
                                    replay.start_recording()
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }

            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            },
        }
    }
}


fn process_keyboard_message(button: &mut Button, is_down: bool) {
    // TODO: with the replay mechanic combined with this code the key can
    // remain stuck if one transition happens during the recording and the
    // other one happens during playback and gets disrecarded therefore!
    if is_down != button.ended_down {
        button.ended_down = is_down;
        button.half_transitions += 1;
    }
}

// TODO: clean the input handleing for the Mouse up before shipping
fn process_mouse_input(window: &Window, input: &mut Input) {
    let mut cursor_posize = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut cursor_posize);
        ScreenToClient(window.handle, &mut cursor_posize);
    }
    input.mouse_x = cursor_posize.x as i32;
    input.mouse_y = cursor_posize.y as i32;

    process_keyboard_message(&mut input.mouse_l,
                             unsafe { (GetKeyState(VK_LBUTTON as i32) & (1 << 15)) != 0 });
    process_keyboard_message(&mut input.mouse_r,
                             unsafe { (GetKeyState(VK_RBUTTON as i32) & (1 << 15)) != 0 });
    process_keyboard_message(&mut input.mouse_m,
                             unsafe { (GetKeyState(VK_MBUTTON as i32) & (1 << 15)) != 0 });
    process_keyboard_message(&mut input.mouse_x1,
                             unsafe { (GetKeyState(VK_XBUTTON1 as i32) & (1 << 15)) != 0 });
    process_keyboard_message(&mut input.mouse_x2,
                             unsafe { (GetKeyState(VK_XBUTTON2 as i32) & (1 << 15)) != 0 });
}

fn get_wall_clock() -> i64 {
    let mut res: i64 = 0;
    unsafe {
        QueryPerformanceCounter(&mut res);
    }
    res
}

fn get_seconds_elapsed(start: i64, end: i64, frequency: i64) -> f32 {
    debug_assert!(start < end);
    (end - start) as f32 / frequency as f32
}


fn get_exe_path() -> PathBuf {
    let mut buffer: [i8; MAX_PATH] = [0; MAX_PATH];
    unsafe {
        // TODO: remove all the occurances of MAX_PATH because on NTFS paths
        // can actually be longer than this constant!
        GetModuleFileNameA(ptr::null_mut(), buffer.as_mut_ptr(), MAX_PATH as u32);
    }
    // TODO: I'm pretty sure this doesn't work with anythin except ascii
    // characters for the path at the moment!
    let string = String::from_iter(buffer.iter().map(|&x| x as u8 as char));
    PathBuf::from(string)
}

fn initialize_replay(exe_dirname: &PathBuf,
                     file_size: usize,
                     game_address: *mut c_void)
                     -> Result<Replay, ()> {
    let mut result: Result<Replay, ()> = Err(());

    let mut mmap_path = exe_dirname.clone();
    mmap_path.push("mmap.rhm");

    let mut input_path = exe_dirname.clone();
    input_path.push("input.rhi");

    let mmap_name = CString::new(mmap_path.to_str().unwrap()).unwrap();
    let input_name = CString::new(input_path.to_str().unwrap()).unwrap();
    let file_handle = unsafe {
        CreateFileA(mmap_name.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    ptr::null_mut(),
                    CREATE_ALWAYS,
                    0,
                    ptr::null_mut())
    };

    if file_handle != INVALID_HANDLE_VALUE {
        let file_size_hi = (file_size >> 32) as DWORD;
        let file_size_lo = (file_size & 0xFFFFFFFF) as DWORD;
        let mapping_handle = unsafe {
            CreateFileMappingA(file_handle,
                               ptr::null_mut(),
                               PAGE_READWRITE,
                               file_size_hi,
                               file_size_lo,
                               ptr::null())
        };

        if !mapping_handle.is_null() {
            let address = unsafe { MapViewOfFile(mapping_handle, FILE_MAP_WRITE, 0, 0, 0) };

            if !address.is_null() {
                result = Ok(Replay {
                    input_path: input_name,
                    input_file_handle: ptr::null_mut(),
                    game_address: game_address,
                    memory: address,
                    memory_size: file_size,
                    state: ReplayState::Nothing,
                });
            } else {
                // TODO: Diagnostic that no replay is possible
            }
        } else {
            // TODO: Diagnostic that no replay is possible
        }
    } else {
        // TODO: Diagnostic that no replay is possible
    }
    result
}

fn log_input(replay: &Replay, input: &mut Input) {
    unsafe {
        let mut ignored: DWORD = 0;
        WriteFile(replay.input_file_handle,
                  input as *mut _ as *mut c_void,
                  mem::size_of_val(input) as DWORD,
                  &mut ignored,
                  ptr::null_mut());
    }
}

fn override_input(replay: &mut Replay, input: &mut Input) {
    unsafe {
        if !replay.input_file_handle.is_null() {
            let mut bytes_read: DWORD = 0;
            ReadFile(replay.input_file_handle,
                     input as *mut _ as *mut c_void,
                     mem::size_of_val(input) as DWORD,
                     &mut bytes_read,
                     ptr::null_mut());
            if bytes_read == 0 {
                // use recursion here to loop
                replay.stop_replay();
                replay.start_replay();
                override_input(replay, input);
            }
        }
    }
}

// TODO: Looped live code editing is currently busted. Needs a fix
// recording starts but playback crashes (infinite loop?)
pub fn winmain() {
    let (XInputGetState, _) = load_xinput_functions();

    let module_handle = unsafe { GetModuleHandleA(ptr::null()) };
    let mut exe_dirname = get_exe_path();
    exe_dirname.pop();


    if module_handle.is_null() {
        panic!("Handle to our executable could not be obtained!");
    }

    let mut window = Window {
        handle: ptr::null_mut(),
        running: false,
        pause: false,
        timer_fine_resolution: false,
        backbuffer: Backbuffer {
            info: Default::default(),
            memory: ptr::null_mut(),
            height: 0,
            width: 0,
            pitch: 0,
            size: 0,
        },

        debug_cursor: unsafe { LoadCursorA(ptr::null_mut(), IDC_ARROW) },
        win_pos: Default::default(),
    };

    resize_dib_section(&mut window.backbuffer, 960, 540);

    let class_str = CString::new("HandmadeHeroWindowClass");
    let window_class = WNDCLASS {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: process_messages,
        cbClsExtra: 0 as c_int,
        cbWndExtra: 0 as c_int,
        hInstance: module_handle,
        hIcon: 0 as HICON,
        hCursor: 0 as HCURSOR,
        hbrBackground: 0 as HBRUSH,
        lpszMenuName: 0 as LPCTSTR,
        lpszClassName: class_str.unwrap().as_ptr(),
    };

    unsafe {
        RegisterClassA(&window_class);
    }

    let window_title = CString::new("Rust Hero").unwrap();


    window.handle = unsafe {
        CreateWindowExA(0 as DWORD,
                        window_class.lpszClassName,
                        window_title.as_ptr(),
                        WS_OVERLAPPEDWINDOW as DWORD | WS_VISIBLE as DWORD,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        0 as HWND,
                        0 as HWND,
                        module_handle,
                        (&mut window) as *mut _ as *mut c_void)
    };

    if window.handle.is_null() {
        panic!("Window could not be created!");
    }

    let monitor_refresh_rate: usize = unsafe {
        let dc = GetDC(window.handle);
        let refresh_rate = GetDeviceCaps(dc, VREFRESH);
        if refresh_rate > 1 {
            refresh_rate as usize
        } else {
            DEFAULT_MONITOR_REFRESH_RATE
        }
    };

    let game_refresh_rate = monitor_refresh_rate / 2;
    let target_seconds_per_frame = 1.0 / game_refresh_rate as f32;


    let mut sound_output = SoundOutput {
        byte_index: 0,
        safety_bytes: (SOUND_BYTES_PER_SECOND / game_refresh_rate as u32) / 2,
        // TODO: Handle all the cases when there was no DirectSound and unwrap fails!
        sound_buffer: dsound_init(window.handle,
                                  SOUND_BYTES_PER_SECOND,
                                  SOUND_BYTES_PER_SECOND / BYTES_PER_SAMPLE)
                          .unwrap(),
    };

    unsafe {
        clear_sound_output(&mut sound_output);
        ((*(*sound_output.sound_buffer).lpVtbl).Play)(sound_output.sound_buffer,
                                                      0,
                                                      0,
                                                      DSBPLAY_LOOPING);
    }

    //    loop {
    //        let mut play_cursor: DWORD = 0;
    //        let mut write_cursor: DWORD = 0;
    //        if unsafe { ((*(*sound_output.sound_buffer).lpVtbl).GetCurrentPosition)
    //                     (sound_output.sound_buffer,
    //                      &mut play_cursor,
    //                      &mut write_cursor) == DS_OK } {
    //            prsizeln!("PC:{} WC:{}", play_cursor, write_cursor);
    //        }
    //    }


    let mut sound_samples: &mut [i16] = unsafe {
        // Allocation implicitly freed at the end of the execution
        let data = VirtualAlloc(ptr::null_mut(),
                                sound_output.get_buffer_size() as SIZE_T,
                                MEM_RESERVE | MEM_COMMIT,
                                PAGE_READWRITE);
        if data.is_null() {
            panic!("Couldn't allocate the resources for the Sound-Buffer!");
        }


        slice::from_raw_parts_mut(data as *mut i16,
                                  sound_output.get_buffer_size() as usize / mem::size_of::<i16>())
    };

    let base_address = if cfg!(ndebug) {
        0
    } else {
        util::tera_bytes(2)
    };
    let permanent_store_size = util::mega_bytes(64);
    let transient_store_size = util::giga_bytes(1);
    let total_size = permanent_store_size + transient_store_size;
    let memory = unsafe {
        VirtualAlloc(base_address as LPVOID,
                     total_size as SIZE_T,
                     MEM_RESERVE | MEM_COMMIT,
                     PAGE_READWRITE)
    };

    if memory.is_null() {
        panic!("Memory for the Game could not be obtained!");
    }

    let mut game_memory: GameMemory = GameMemory {
        initialized: false,
        permanent: unsafe { slice::from_raw_parts_mut(memory as *mut u8, permanent_store_size) },
        transient: unsafe {
            slice::from_raw_parts_mut((memory as *mut u8).offset(permanent_store_size as isize),
                                      permanent_store_size)
        },
        platform_read_entire_file: debug::platform_read_entire_file,
        platform_write_entire_file: debug::platform_write_entire_file,
        platform_free_file_memory: debug::platform_free_file_memory,
    };

    let mut replay = initialize_replay(&exe_dirname, total_size, memory)
                         .ok()
                         .expect("Error with replay");

    window.timer_fine_resolution = unsafe { timeBeginPeriod(1) == TIMERR_NOERROR };

    let mut game_dll_path = exe_dirname.clone();
    game_dll_path.push("game.dll");

    let mut temp_dll_path = exe_dirname.clone();
    temp_dll_path.push("game_temp.dll");

    let game_dll_string = CString::new(game_dll_path.to_str().unwrap()).unwrap();
    let temp_dll_string = CString::new(temp_dll_path.to_str().unwrap()).unwrap();

    let thread_context = ThreadContext;

    let mut sound_is_valid = false;
    let mut last_time_marker_index: usize = 0;
    let mut last_time_markers: [debug::SoundTimeMarker; 15] = [Default::default(); 15];

    let mut new_input: &mut Input = &mut Default::default();
    let mut old_input: &mut Input = &mut Default::default();

    let mut counter_frequency: i64 = 0;
    unsafe {
        QueryPerformanceFrequency(&mut counter_frequency);
    }

    let mut last_cycles = intrinsics::__rdtsc();

    let mut last_counter: i64 = get_wall_clock();
    let mut flip_wall_clock: i64 = 0;

    let mut game = load_game_functions(&game_dll_string, &temp_dll_string);

    window.running = true;
    while window.running {

        new_input.delta_t = target_seconds_per_frame;

        let new_write_time = get_last_write_time(&game_dll_string).unwrap_or(FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        });
        if unsafe { CompareFileTime(&game.write_time, &new_write_time) } != 0 {

            unload_game_functions(&mut game);
            game = load_game_functions(&game_dll_string, &temp_dll_string);
        }

        // Keep the old button state but zero out the halftransitoncount to not mess up
        new_input.controllers[0] = old_input.controllers[0];
        new_input.controllers[0].is_connected = true;
        new_input.controllers[0].zero_half_transitions();
        process_pending_messages(&mut window, &mut new_input.controllers[0], &mut replay);

        process_mouse_input(&window, new_input);

        if !window.pause {
            process_xinput(XInputGetState,
                           &mut new_input.controllers[1..],
                           &old_input.controllers[1..]);

            let mut video_buf = VideoBuffer {
                memory: unsafe {
                    slice::from_raw_parts_mut(window.backbuffer.memory as *mut u32,
                                              (window.backbuffer.size / BYTES_PER_PIXEL) as usize)
                },
                width: window.backbuffer.width as usize,
                height: window.backbuffer.height as usize,
                pitch: (window.backbuffer.pitch / BYTES_PER_PIXEL) as usize,
            };

            if replay.is_recording() {
                log_input(&replay, new_input);
            }

            if replay.is_replaying() {
                override_input(&mut replay, new_input);
            }

            (game.update_and_render)(&thread_context, &mut game_memory, new_input, &mut video_buf);

            let from_begin_to_audio = get_seconds_elapsed(flip_wall_clock,
                                                          get_wall_clock(),
                                                          counter_frequency);
            let mut play_cursor: DWORD = 0;
            let mut write_cursor: DWORD = 0;

            if unsafe {
                ((*(*sound_output.sound_buffer)
                       .lpVtbl)
                     .GetCurrentPosition)(sound_output.sound_buffer,
                                          &mut play_cursor,
                                          &mut write_cursor) == DS_OK
            } {

                if !sound_is_valid {
                    sound_output.byte_index = write_cursor;
                    sound_is_valid = true;
                }

                let sound_buffer_size = sound_output.get_buffer_size();
                let byte_to_lock = sound_output.byte_index % sound_buffer_size;

                let safe_write_cursor = if write_cursor < play_cursor {
                    write_cursor + sound_output.safety_bytes + sound_buffer_size
                } else {
                    write_cursor + sound_output.safety_bytes
                };
                debug_assert!(safe_write_cursor >= play_cursor);

                let expected_sound_bytes_per_frame = SOUND_BYTES_PER_SECOND /
                                                     game_refresh_rate as u32;
                let seconds_to_flip = if target_seconds_per_frame > from_begin_to_audio {
                    target_seconds_per_frame - from_begin_to_audio
                } else {
                    0.0f32
                };
                let expected_bytes_to_flip =
                    (expected_sound_bytes_per_frame as f32 *
                     (seconds_to_flip / target_seconds_per_frame)) as DWORD;
                let expected_frame_boundary_byte = play_cursor + expected_bytes_to_flip;
                let audio_card_not_latent = safe_write_cursor < expected_frame_boundary_byte;

                // The division with BYTES_PER_SAMPLE as well as the multiplication afterwards
                // are for the fact that we need a multiple of 4 to be sample aligned
                // because we calculated the expected_bytes_to_flip exactly it may
                // not be 4byte aligned so we do the alignment here!
                let target_cursor = if audio_card_not_latent {
                    (((expected_frame_boundary_byte + expected_sound_bytes_per_frame) /
                      BYTES_PER_SAMPLE) * BYTES_PER_SAMPLE) % sound_buffer_size
                } else {
                    (((write_cursor + expected_sound_bytes_per_frame +
                       sound_output.safety_bytes) / BYTES_PER_SAMPLE) *
                     BYTES_PER_SAMPLE) % sound_buffer_size
                };

                let bytes_to_write = if byte_to_lock > target_cursor {
                    sound_buffer_size - byte_to_lock + target_cursor
                } else {
                    target_cursor - byte_to_lock
                };

                let mut sound_buf = SoundBuffer {
                    samples: &mut sound_samples[..bytes_to_write as usize / mem::size_of::<i16>()],
                    samples_per_second: SOUND_BYTES_PER_SECOND / BYTES_PER_SAMPLE,
                };

                (game.get_sound_samples)(&thread_context, &mut game_memory, &mut sound_buf);

                fill_sound_output(&mut sound_output, byte_to_lock, bytes_to_write, &sound_buf);

            } else {
                sound_is_valid = false;
            }

            let mut seconds_elapsed_for_work = get_seconds_elapsed(last_counter,
                                                                   get_wall_clock(),
                                                                   counter_frequency);
            if seconds_elapsed_for_work < target_seconds_per_frame {
                while seconds_elapsed_for_work < target_seconds_per_frame {
                    if window.timer_fine_resolution {
                        let sleep_ms = (1000.0 *
                                        (target_seconds_per_frame - seconds_elapsed_for_work) -
                                        1.0f32) as DWORD;
                        if sleep_ms > 0 {
                            unsafe {
                                Sleep(sleep_ms);
                            }
                        }
                    }
                    seconds_elapsed_for_work = get_seconds_elapsed(last_counter,
                                                                   get_wall_clock(),
                                                                   counter_frequency);
                }
            } else {
                // TODO: missed frame time we have to put out a log
            }

            let end_counter = get_wall_clock();

            let (width, height) = get_client_dimensions(window.handle).unwrap();

            {
                let context = unsafe { GetDC(window.handle) };
                if context.is_null() {
                    panic!("DC for the Window not available!");
                }
                blit_buffer_to_window(context, &window.backbuffer, width, height);
                unsafe {
                    ReleaseDC(window.handle, context);
                }
            }

            flip_wall_clock = get_wall_clock();

            if cfg!(not(ndebug)) {
                let mut play_cursor: DWORD = 0;
                let mut write_cursor: DWORD = 0;
                if unsafe {
                    ((*(*sound_output.sound_buffer)
                           .lpVtbl)
                         .GetCurrentPosition)(sound_output.sound_buffer,
                                              &mut play_cursor,
                                              &mut write_cursor) == DS_OK
                } {
                    last_time_markers[last_time_marker_index].flip_play_cursor = play_cursor;
                    last_time_markers[last_time_marker_index].flip_write_cursor = write_cursor;

                    last_time_marker_index += 1;
                    if last_time_marker_index >= last_time_markers.len() {
                        last_time_marker_index = 0;
                    }
                }
            }

            let ms_per_frame = 1000.0 *
                               get_seconds_elapsed(last_counter, end_counter, counter_frequency);
            let fps: f32 = 1000.0 / ms_per_frame;
            let display_cicles = intrinsics::__rdtsc();
            let mc_per_second = (display_cicles - last_cycles) as f32 / (1000.0 * 1000.0);

            println!("{:.2}ms/f, {:.2}f/s, {:.2}mc/s",
                     ms_per_frame,
                     fps,
                     mc_per_second);

            mem::swap(new_input, old_input);

            last_counter = end_counter;
            last_cycles = intrinsics::__rdtsc();
        }
    }
}
