use std::ptr;
use std::raw;
use std::mem; 
use std::i16;
use ffi::*;
use game;

#[cfg(not(ndebug))]
pub mod debug {
    use ffi::*;
    use std::ptr;
    use super::util;

    pub struct ReadFileResult {
        pub size: u32,
        pub contents: *mut c_void,
    }

    //TODO: make generic over the thing we want to load?
    //or just return a byteslice?
    pub fn platform_read_entire_file(filename: &str) -> Result<ReadFileResult, ()> {
        debug_assert!(filename.len() <= MAX_PATH);

        let mut result: Result<ReadFileResult, ()> = Err(());
        let name = filename.to_c_str();
        let handle =
            unsafe { CreateFileA(name.as_ptr(), 
                                 GENERIC_READ, FILE_SHARE_READ,
                                 ptr::null_mut(), OPEN_EXISTING, 
                                 FILE_ATTRIBUTE_NORMAL, ptr::null_mut()) };

        if handle != INVALID_HANDLE_VALUE as *mut c_void {
            let mut file_size: i64 = 0;
            if unsafe { GetFileSizeEx(handle, &mut file_size) } != 0 {
                let memory: *mut c_void = 
                    unsafe { VirtualAlloc(ptr::null_mut(), file_size as SIZE_T,
                                          MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE) };

                if memory.is_not_null() {
                let size = util::safe_truncate_u64(file_size as u64);
                    let mut bytes_read = 0;
                    if (unsafe { ReadFile(handle, memory, size, 
                                &mut bytes_read, ptr::null_mut()) } != 0)
                        && (bytes_read == size) {

                        result = Ok(ReadFileResult {
                                        size: size,
                                        contents: memory,
                                      });
                    } else {
                        platform_free_file_memory(memory);
                    }
                }
            }
            unsafe { CloseHandle(handle); }
        }

        result
    }
    
    pub fn platform_free_file_memory(memory: *mut c_void) {
        if memory.is_not_null() {
            unsafe { VirtualFree(memory, 0, MEM_RELEASE); }
        }
    }

    pub fn platform_write_entire_file(filename: &str, size: DWORD,
                                      memory: *mut c_void) -> bool {
        debug_assert!(filename.len() <= MAX_PATH);

        let mut result = false;
        let name = filename.to_c_str();
        let handle =
            unsafe { CreateFileA(name.as_ptr(), 
                                 GENERIC_WRITE, 0,
                                 ptr::null_mut(), CREATE_ALWAYS, 
                                 FILE_ATTRIBUTE_NORMAL, ptr::null_mut()) };

        if handle != INVALID_HANDLE_VALUE as *mut c_void {
            let mut bytes_written = 0;
            if unsafe { WriteFile(handle, memory, size, 
                                  &mut bytes_written, ptr::null_mut()) } != 0 {

                result = bytes_written == size;
            }
            unsafe { CloseHandle(handle); }
        }
        result
    }
}

//TODO: this part should be moved to a platform independant file
mod util {
    use std::u32;

    pub fn safe_truncate_u64(value: u64) -> u32 {
        debug_assert!(value <= u32::MAX as u64);
        value as u32
    }

    pub fn kilo_bytes(b: uint) -> uint {
        b * 1024
    }

    pub fn mega_bytes(mb: uint) -> uint {
        kilo_bytes(mb) * 1024
    }

    pub fn giga_bytes(gb: uint) -> uint {
        mega_bytes(gb) * 1024
    }

    pub fn tera_bytes(tb: uint) -> uint {
        giga_bytes(tb) * 1024
    }

}

//Graphics System constants
const BYTES_PER_PIXEL: c_int = 4;

//Sound System constants
const CHANNELS: WORD = 2;
const BITS_PER_CHANNEL: WORD = 16;
const SAMPLES_PER_SECOND: DWORD = 48000;
const BYTES_PER_SAMPLE: DWORD = 4;
const LATENCY_SAMPLE_COUNT: DWORD = SAMPLES_PER_SECOND / 15;

struct SoundOutput {
    sample_index: DWORD,
    sound_buffer: *mut IDirectSoundBuffer,
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
    backbuffer: Backbuffer,
}

impl Window {
    fn process_messages(&mut self, message: UINT, 
                        wparam: WPARAM, lparam: LPARAM) -> LRESULT {

        let mut res: LRESULT = 0;

        match message {
            WM_DESTROY => self.running = false,
            WM_CLOSE => self.running = false,

            WM_SYSKEYDOWN
            | WM_SYSKEYUP
            | WM_KEYDOWN
            | WM_KEYUP => debug_assert!(false, "There sould be no key-messages in\
                                                the windows message callback!"),

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
            },

            _ => unsafe { 
                res = DefWindowProcA(self.handle, message, wparam, lparam);
            },
        }

        res
    }
}

extern "system" fn process_messages(handle: HWND, message: UINT, wparam: WPARAM, 
                                    lparam: LPARAM) -> LRESULT {
    let mut res: LRESULT = 0;

    match message {
        //When creating the window we pass the window struct containing all our information
        //and registering it with windows
        WM_CREATE => {
            unsafe { 
                let ptr_to_window = (*(lparam as *const CREATESTRUCT)).lpCreateParams as LONG_PTR;
                SetWindowLongPtrA(handle, GWLP_USERDATA, ptr_to_window); 
            }
        },

        //For all the other messages we know we have a window struct registered
        //with windows. So we get it and dispatch to its message handleing
        _ => {
            unsafe {
                let window = GetWindowLongPtrA(handle, GWLP_USERDATA) as *mut Window;
                if window.is_not_null() {
                    res = (*window).process_messages(message, wparam, lparam);
                //During construction when there is still no struct registered we need to 
                //handle all the cases with the default behavior
                } else {
                    res = DefWindowProcA(handle, message, wparam, lparam);
                }
            }
        },
    }

    res
}

fn fill_sound_output(sound_output: &mut SoundOutput, 
                     byte_to_lock: DWORD,
                     bytes_to_write: DWORD,
                     source: &game::SoundBuffer) {

    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0; 
    let mut region2_size: DWORD = 0; 

    let lock = unsafe {
        ((*(*sound_output.sound_buffer).lpVtbl).Lock)(sound_output.sound_buffer, 
                                         byte_to_lock,
                                         bytes_to_write,
                                         &mut region1, &mut region1_size,
                                         &mut region2, &mut region2_size,
                                         0 as DWORD)
    };

    if SUCCEEDED(lock) {
        fn fill_region(region: *mut c_void, region_size: DWORD,
                       sound_output: &mut SoundOutput, buffer: &game::SoundBuffer) {
                           
            debug_assert!((region_size % BYTES_PER_SAMPLE) == 0);
            let region_sample_count = region_size/BYTES_PER_SAMPLE;
            let mut out = region as *mut i16;
            let mut source: *const i16 = buffer.samples.as_ptr();

            for _ in range(0, region_sample_count) {
                unsafe {
                    *out = *source;
                    out = out.offset(1);
                    source = source.offset(1);
                    *out = *source;
                    out = out.offset(1);
                    source = source.offset(1);
                }
                sound_output.sample_index += 1;
            }
        }
        
        fill_region(region1, region1_size, sound_output, source);
        fill_region(region2, region2_size, sound_output, source);

        unsafe {
            ((*(*sound_output.sound_buffer).lpVtbl).Unlock)(sound_output.sound_buffer,
                                               region1, region1_size,
                                               region2, region2_size);
        }
    }
}

fn clear_sound_output(sound_output: &mut SoundOutput, dsbcaps: &DSBCAPS) {

    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0; 
    let mut region2_size: DWORD = 0; 

    let lock = unsafe { 
        ((*(*sound_output.sound_buffer).lpVtbl).Lock)(sound_output.sound_buffer, 
                                         0,
                                         dsbcaps.dwBufferBytes,
                                         &mut region1, &mut region1_size,
                                         &mut region2, &mut region2_size,
                                         0 as DWORD)
    };

    if SUCCEEDED(lock) {

        fn fill_region(region: *mut c_void, region_size: DWORD) {
            let mut out = region as *mut u8;
            for _ in range(0, region_size) {
                unsafe { 
                    *out = 0;
                    out = out.offset(1);
                }
            }
        }

        fill_region(region1, region1_size);
        fill_region(region2, region2_size);

        unsafe {
            ((*(*sound_output.sound_buffer).lpVtbl).Unlock)(sound_output.sound_buffer,
                                               region1, region1_size,
                                               region2, region2_size);
        }
    }
}

fn dsound_init(window: HWND, buffer_size_bytes: DWORD, 
               samples_per_second: DWORD) -> Result<*mut IDirectSoundBuffer, ()> {
    let library_name = "dsound.dll".to_c_str();
    let library = unsafe { LoadLibraryA(library_name.as_ptr()) };

    if library.is_not_null() {
        let create_name = "DirectSoundCreate".to_c_str();
        let ds_create = unsafe { GetProcAddress(library, create_name.as_ptr()) };
        if ds_create.is_null() { return Err(()); }

        //We have DirectSound capabilities
        let DirectSoundCreate: DirectSoundCreate_t = unsafe { mem::transmute(ds_create) };
        let mut direct_sound: *mut IDirectSound = ptr::null_mut();
        if SUCCEEDED(DirectSoundCreate(ptr::null(), &mut direct_sound, ptr::null_mut())) {
            //Creating the primary buffer and setting our format
            let buffer_desc: DSBUFFERDESC = DSBUFFERDESC {
                                                dwSize: mem::size_of::<DSBUFFERDESC>() as DWORD,
                                                dwFlags: DSBCAPS_PRIMARYBUFFER,
                                                dwBufferBytes: 0 as DWORD,
                                                dwReserved: 0 as DWORD,
                                                lpwfxFormat: ptr::null_mut(),
                                                guid: Default::default(),
                                          };
            let mut primary_buffer: *mut IDirectSoundBuffer = ptr::null_mut();
            //Holy shit: it's the syntax from hell!
            unsafe { 
                ((*(*direct_sound).lpVtbl).SetCooperativeLevel)(direct_sound, window, DSSCL_PRIORITY);
                ((*(*direct_sound).lpVtbl).CreateSoundBuffer)(direct_sound, 
                                                              &buffer_desc, 
                                                              &mut primary_buffer, 
                                                              ptr::null_mut());
            }

            let block_align = (CHANNELS * BITS_PER_CHANNEL) / 8;
            let mut wave_format: WAVEFORMATEX = 
                WAVEFORMATEX {
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

            //Creating our secondary buffer
            let buffer_desc_secondary: DSBUFFERDESC = 
                DSBUFFERDESC {
                    dwSize: mem::size_of::<DSBUFFERDESC>() as DWORD,
                    dwFlags: 0 as DWORD,
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

            return Ok(secondary_buffer)
        }
    }

    Err(())
}

//Stub functions if none of the XInput libraries could be loaded!
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD { ERROR_DEVICE_NOT_CONNECTED }
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD { ERROR_DEVICE_NOT_CONNECTED }

fn load_xinput_functions() -> (XInputGetState_t, XInputSetState_t) {

    let xlib_first_name = "xinput1_4.dll".to_c_str();
    let xlib_second_name = "xinput1_3.dll".to_c_str();
    let xlib_third_name = "xinput9_1_0.dll".to_c_str();

    let mut module = unsafe { LoadLibraryA( xlib_first_name.as_ptr() ) };
    
    if module.is_null() {
        module = unsafe { LoadLibraryA( xlib_second_name.as_ptr() ) };
    }
    if module.is_null() {
        module = unsafe { LoadLibraryA( xlib_third_name.as_ptr() ) };
    }

    if module.is_not_null() {
        let get_state_name = "XInputGetState".to_c_str();
        let set_state_name = "XInputSetState".to_c_str();

        let xinput_get_state = unsafe { GetProcAddress(module, get_state_name.as_ptr() ) };
        let xinput_set_state = unsafe { GetProcAddress(module, set_state_name.as_ptr() ) };

        unsafe { (mem::transmute(xinput_get_state), 
                  mem::transmute(xinput_set_state)) }
    } else {
        (xinput_get_state_stub, xinput_set_state_stub)
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
             },
     }
}


fn resize_dib_section(buffer: &mut Backbuffer, width: c_int, height: c_int) {
    if buffer.memory.is_not_null() {
       unsafe { 
           if VirtualFree(buffer.memory, 0 as SIZE_T , MEM_RELEASE) == 0 {
               panic!("VirtualFree ran into an error");
           }
       }
    }

    //Height is negative to denote a top to bottom Bitmap for StretchDIBits
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
        //The last created buffer is implicitly destroyed at end of execution
        buffer.memory = VirtualAlloc(ptr::null_mut(), buffer.size as SIZE_T,
                                     MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE);
        if buffer.memory.is_null() {
            panic!("No memory could be allocated by VirtualAlloc");
        }
    }
}

fn blit_buffer_to_window(context: HDC, buffer: &Backbuffer, client_width: c_int,
                 client_height: c_int) {
    unsafe {
        StretchDIBits(context,
                 0, 0, client_width, client_height,
                 0, 0, buffer.width, buffer.height,
                 buffer.memory as *const c_void,
                 &buffer.info,
                 DIB_RGB_COLORS, SRCCOPY)
    };
}

fn process_xinput_button(xinput_button_state: WORD,
                         old_state: &game::Button, 
                         new_state: &mut game::Button,
                         button_bit: WORD) {

    new_state.ended_down = (xinput_button_state & button_bit) == button_bit;
    new_state.half_transitions =
        if old_state.ended_down != new_state.ended_down {
            1
        } else {
            0
        };
}

fn process_pending_messages(window: &mut Window, 
                            keyboard_controller: &mut game::ControllerInput) {
    let mut msg = Default::default();
    //Process the Message Queue
    while unsafe {PeekMessageA(&mut msg, 0 as HWND,
                       0 as UINT, 0 as UINT, PM_REMOVE) } != 0 {
        match msg.message {
            WM_QUIT => window.running = false,

            WM_SYSKEYDOWN
            | WM_SYSKEYUP
            | WM_KEYDOWN
            | WM_KEYUP => {
                let vk_code = msg.wparam as u8;
                let was_down = (msg.lparam & (1 << 30)) != 0;
                let is_down = (msg.lparam & (1 << 31)) == 0;

                //Rust currently doesn't allow casts in matches so we have to
                //conform to one type which is u8 here
                const W: u8 = 'W' as u8;
                const A: u8 = 'A' as u8;
                const S: u8 = 'S' as u8;
                const D: u8 = 'D' as u8;
                const Q: u8 = 'Q' as u8;
                const E: u8 = 'E' as u8;

                if was_down != is_down {
                    match vk_code {
                        VK_UP => process_keyboard_message(
                                        &mut keyboard_controller.action_up, is_down),
                        VK_DOWN => process_keyboard_message(
                                        &mut keyboard_controller.action_down, is_down),
                        VK_LEFT => process_keyboard_message(
                                        &mut keyboard_controller.action_left, is_down),
                        VK_RIGHT => process_keyboard_message(
                                        &mut keyboard_controller.action_right, is_down),
                        W => process_keyboard_message(
                                        &mut keyboard_controller.move_up, is_down),
                        A => process_keyboard_message(
                                        &mut keyboard_controller.move_left, is_down),
                        S => process_keyboard_message(
                                        &mut keyboard_controller.move_down, is_down),
                        D => process_keyboard_message(
                                        &mut keyboard_controller.move_right, is_down),
                        E => process_keyboard_message(
                                        &mut keyboard_controller.right_shoulder, is_down),
                        Q => process_keyboard_message(
                                        &mut keyboard_controller.left_shoulder, is_down),
                        VK_ESCAPE => process_keyboard_message(
                                        &mut keyboard_controller.back, is_down),
                        VK_SPACE => process_keyboard_message(
                                        &mut keyboard_controller.start, is_down),
                        VK_F4 => {
                            let alt_key_down = (msg.lparam & (1 << 29)) != 0;
                            if alt_key_down {
                                window.running = false; 
                            }
                        },
                        _ => (),
                    }
                }
            },

            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            },
        }
    }
}

fn process_keyboard_message(button: &mut game::Button, is_down: bool) {
    debug_assert!(is_down != button.ended_down);
    button.ended_down = is_down;
    button.half_transitions += 1;
}


fn get_wall_clock() -> i64 {
    let mut res: i64 = 0;
    unsafe { QueryPerformanceCounter(&mut res); }
    res
}

fn get_seconds_elapsed(start: i64, end: i64, frequency: i64) -> f32 {
    debug_assert!(start < end);
    (end - start) as f32 / frequency as f32
}

#[main]
fn main() {
    let (XInputGetState, _) = load_xinput_functions();
    
    let module_handle = unsafe { GetModuleHandleA(ptr::null()) };
    if module_handle.is_null() {
        panic!("Handle to our executable could not be obtained!");
    }

    let mut window = Window {
                        handle: ptr::null_mut(),
                        running: false,
                        backbuffer: Backbuffer {
                            info: Default::default(), 
                            memory: ptr::null_mut(),
                            height: 0,
                            width: 0,
                            pitch: 0,
                            size: 0,
                        },
                    };

    resize_dib_section(&mut window.backbuffer, 1280, 720); 

    let class_str = "HandmadeHeroWindowClass".to_c_str();
    let window_class = WNDCLASS{style: CS_OWNDC|CS_HREDRAW|CS_VREDRAW, 
                                lpfnWndProc: process_messages,
                                cbClsExtra: 0 as c_int,
                                cbWndExtra: 0 as c_int,
                                hInstance: module_handle,
                                hIcon: 0 as HICON,
                                hCursor: 0 as HCURSOR,
                                hbrBackground: 0 as HBRUSH,
                                lpszMenuName: 0 as LPCTSTR,
                                lpszClassName: class_str.as_ptr()};

    unsafe { RegisterClassA(&window_class); }

    let window_title = "Handmade Hero".to_c_str();

    window.handle = unsafe {
        CreateWindowExA(0 as DWORD, window_class.lpszClassName, 
                        window_title.as_ptr(),
                        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                        CW_USEDEFAULT, CW_USEDEFAULT, 
                        CW_USEDEFAULT, CW_USEDEFAULT, 
                        0 as HWND, 0 as HWND, module_handle,
                        (&mut window) as *mut _ as *mut c_void)
    };

    if window.handle.is_null() {
        panic!("Window could not be created!");
    }

    //Needed for Sound System test
    let mut sound_output = SoundOutput {
        sample_index: 0,
        //TODO: Handle all the cases when there was no DirectSound and unwrap fails!
        sound_buffer: dsound_init(window.handle, SAMPLES_PER_SECOND * BYTES_PER_SAMPLE,
                                  SAMPLES_PER_SECOND).unwrap(),
    };

    let mut dsbcaps: DSBCAPS = Default::default();
    dsbcaps.dwSize = mem::size_of::<DSBCAPS>() as DWORD;

    unsafe {
        ((*(*sound_output.sound_buffer).lpVtbl).GetCaps)(sound_output.sound_buffer, &mut dsbcaps);

        clear_sound_output(&mut sound_output, &dsbcaps);
        ((*(*sound_output.sound_buffer).lpVtbl).Play)(sound_output.sound_buffer, 0, 0, DSBPLAY_LOOPING);
    }

    let context = unsafe { GetDC(window.handle) };
    if context.is_null() {
        panic!("DC for the Window not available!");
    }

    //TODO: not safe operation because VirtualAlloc could fail!
    let mut sound_samples: &mut [i16] = unsafe { 
            //Allocation implicitly freed at the end of the execution
            let data = VirtualAlloc(ptr::null_mut(), dsbcaps.dwBufferBytes as SIZE_T,
                                     MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE);
            mem::transmute(
                raw::Slice { data: data as *const i16,
                             len: dsbcaps.dwBufferBytes as uint / mem::size_of::<i16>()})
        };
   
    let base_address = if cfg!(ndebug) { 0 } else { util::tera_bytes(2) };
    let permanent_store_size = util::mega_bytes(64);
    let transient_store_size = util::giga_bytes(4);
    let memory = unsafe { VirtualAlloc(base_address as LPVOID, 
                                       (permanent_store_size + transient_store_size) as SIZE_T,
                                       MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE) };

    if memory.is_null() { panic!("Memory for the Game could not be obtained!"); }

    let mut game_memory: game::GameMemory = 
        game::GameMemory {
            initialized: false,
            permanent: unsafe { 
                        mem::transmute( raw::Slice { 
                                            data: memory as *const u8, 
                                            len: permanent_store_size
                                        } 
                                      ) 
                        },
            transient: unsafe { 
                        mem::transmute( raw::Slice { 
                                            data: (memory as *const u8)
                                                   .offset(permanent_store_size as int), 
                                            len: transient_store_size
                                        }
                                      ) 
                        },
        };

    let sleep_is_granular = unsafe { timeBeginPeriod(1) == TIMERR_NOERROR };

    //TODO: we need to get the actual refresh rate here
    let moniter_refresh_rate: u32 = 60;
    let game_refresh_rate =  moniter_refresh_rate / 2;
    let target_seconds_per_frame = 1.0 / game_refresh_rate as f32;

    let mut new_input: &mut game::Input = &mut Default::default();
    let mut old_input: &mut game::Input = &mut Default::default();

    let mut counter_frequency: i64 = 0;
    unsafe { QueryPerformanceFrequency(&mut counter_frequency); }

    let mut last_cycles = intrinsics::__rdtsc();

    let mut last_counter: i64 = get_wall_clock();

    window.running = true;
    while window.running {

        //Keep the old button state but zero out the halftransitoncount to not mess up
        new_input.controllers[0] = old_input.controllers[0];
        new_input.controllers[0].is_connected = true;
        new_input.controllers[0].zero_half_transitions();
        process_pending_messages(&mut window, &mut new_input.controllers[0]);

        let max_controller_count = 
            if XUSER_MAX_COUNT > (new_input.controllers.len() - 1) as u32 {
                (new_input.controllers.len() - 1) as u32
            } else {
                XUSER_MAX_COUNT
            };

        for controller in range(0, max_controller_count) {
            let mut state = Default::default();
            let controller_num: uint = controller as uint + 1;
            let res: u32 = XInputGetState(controller, &mut state);
            match res {
                //Case the Controller is connected and we got data
                ERROR_SUCCESS => {
                    let old_controller = &old_input.controllers[controller_num];
                    let mut new_controller = &mut new_input.controllers[controller_num];
                    new_controller.is_connected = true;

                    let gamepad = &state.Gamepad;

                    fn process_xinput_stick(value: SHORT,
                                            dead_zone: SHORT) -> f32 {
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

                    //If the DPAD was not used default to analog input with
                    //the controllers
                    if !(dpad_up || dpad_down || dpad_left || dpad_right) {
                        new_controller.average_x = Some(xvalue);
                        new_controller.average_y = Some(yvalue);
                    } else {
                        new_controller.average_x = None;
                        new_controller.average_y = None;
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
                    let up_fake = if yvalue > threshhold { 1 } else { 0 };
                    let down_fake = if yvalue < -threshhold { 1 } else { 0 };
                    let left_fake = if xvalue < -threshhold { 1 } else { 0 };
                    let right_fake = if xvalue > threshhold { 1 } else { 0 };
                    process_xinput_button(up_fake,
                                           &old_controller.move_up,
                                           &mut new_controller.move_up,
                                           1);
                    process_xinput_button(down_fake,
                                           &old_controller.move_down,
                                           &mut new_controller.move_down,
                                           1);
                    process_xinput_button(left_fake,
                                           &old_controller.move_left,
                                           &mut new_controller.move_left,
                                           1);
                    process_xinput_button(right_fake,
                                           &old_controller.move_right,
                                           &mut new_controller.move_right,
                                           1);

                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.left_shoulder,
                                           &mut new_controller.left_shoulder,
                                           XINPUT_GAMEPAD_LEFT_SHOULDER);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.right_shoulder, 
                                           &mut new_controller.right_shoulder,
                                           XINPUT_GAMEPAD_RIGHT_SHOULDER);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.action_up, 
                                           &mut new_controller.action_up,
                                           XINPUT_GAMEPAD_Y);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.action_down, 
                                           &mut new_controller.action_down,
                                           XINPUT_GAMEPAD_A);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.action_left, 
                                           &mut new_controller.action_left,
                                           XINPUT_GAMEPAD_X);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.action_right, 
                                           &mut new_controller.action_right,
                                           XINPUT_GAMEPAD_B);

                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.start,
                                           &mut new_controller.start,
                                           XINPUT_GAMEPAD_START);
                    process_xinput_button(gamepad.wButtons,
                                           &old_controller.back,
                                           &mut new_controller.back,
                                           XINPUT_GAMEPAD_BACK);
                },

                //Case the Controller is not connected
                ERROR_DEVICE_NOT_CONNECTED => 
                    new_input.controllers[controller_num].is_connected = false,


                //Some arbitrary Error was found
                _ => (),
            }
        }

        let mut write_cursor: DWORD = 0;
        let mut play_cursor: DWORD = 0;
        let mut bytes_to_write = 0;
        let mut byte_to_lock = 0;

        let sound_is_valid = 
            if SUCCEEDED( unsafe { ((*(*sound_output.sound_buffer).lpVtbl).GetCurrentPosition)
                         (sound_output.sound_buffer, &mut play_cursor, &mut write_cursor) }) {

                byte_to_lock = (sound_output.sample_index * BYTES_PER_SAMPLE) % dsbcaps.dwBufferBytes; 
                let target_cursor = (play_cursor + LATENCY_SAMPLE_COUNT * BYTES_PER_SAMPLE) % dsbcaps.dwBufferBytes;
                bytes_to_write = if byte_to_lock > target_cursor {
                    dsbcaps.dwBufferBytes - byte_to_lock + target_cursor
                } else {
                    target_cursor - byte_to_lock
                };

                true
            } else { false };

        let mut sound_buf = game::SoundBuffer {
            samples: sound_samples.slice_to_mut(bytes_to_write as uint/
                                                mem::size_of::<i16>()),
            samples_per_second: SAMPLES_PER_SECOND,
        };

        let mut video_buf = game::VideoBuffer {
            memory: unsafe { mem::transmute(
                             raw::Slice { data: window.backbuffer.memory as *const u32, 
                                 len: (window.backbuffer.size/BYTES_PER_PIXEL) as uint})
                           },
            width: window.backbuffer.width as uint,
            height: window.backbuffer.height as uint,
            pitch: (window.backbuffer.pitch/BYTES_PER_PIXEL) as uint,
        };

        game::game_update_and_render(&mut game_memory,
                                     new_input,
                                     &mut video_buf,
                                     &mut sound_buf);

        if sound_is_valid {
            fill_sound_output(&mut sound_output, byte_to_lock, bytes_to_write, &sound_buf);
        }

        let mut seconds_elapsed_for_work = get_seconds_elapsed(last_counter,
                                                               get_wall_clock(),
                                                               counter_frequency);
        if seconds_elapsed_for_work < target_seconds_per_frame {
            while seconds_elapsed_for_work < target_seconds_per_frame {
                if sleep_is_granular {
                    let sleep_ms = (1000.0 * (target_seconds_per_frame 
                                              - seconds_elapsed_for_work)) as DWORD;
                    if sleep_ms > 0 {
                        unsafe { Sleep(sleep_ms); }
                    }
                }
                seconds_elapsed_for_work = get_seconds_elapsed(last_counter,
                                                               get_wall_clock(),
                                                               counter_frequency);
            }
        } else {
            //TODO: missed frame time we have to put out a log
        }

        let (width, height) = get_client_dimensions(window.handle).unwrap();
        blit_buffer_to_window(context, &window.backbuffer, width, height);

        let ms_per_frame = 1000.0 * get_seconds_elapsed(last_counter,
                                                        get_wall_clock(),
                                                        counter_frequency);
        let fps: f32 = 1000.0 / ms_per_frame;
        let display_cicles = intrinsics::__rdtsc();
        let mc_per_second = (display_cicles - last_cycles) as f32/ (1000.0 * 1000.0);

        println!("{:.2}ms/f, {:.2}f/s, {:.2}mc/s", ms_per_frame, fps, mc_per_second);

        //TODO: clear the inputs here?
        mem::swap(new_input, old_input);

        last_counter = get_wall_clock();
        last_cycles = intrinsics::__rdtsc();
    }

    //Reset the Sheduler resolution back to normal
    //TODO: the timer resolution change should happen in an RAII manner
    unsafe { timeEndPeriod(1); }
}
