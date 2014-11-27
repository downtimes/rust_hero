#![feature(globs)]
#![allow(non_snake_case)]

extern crate libc;

use std::ptr;
use std::mem;
use ffi::*;

mod ffi;


//TODO:These are a global for now needs to cleanup later on by packing them in a window
//struct. Need to be global at the Moment because we also need them in the callback from
//Windows. See SFML Source code for a solution to the "global problem" 
static mut running: bool = false;
static mut back_buffer: Backbuffer = 
                        Backbuffer { info: BITMAPINFO {
                                            bmiHeader: BITMAPINFOHEADER {
                                                biSize: 0 as DWORD,
                                                biWidth: 0 as LONG,
                                                biHeight: 0 as LONG,
                                                biPlanes: 1 as WORD,
                                                biBitCount: 0 as WORD,
                                                biCompression: BI_RGB,
                                                biSizeImage: 0 as DWORD,
                                                biXPelsPerMeter: 0 as LONG,
                                                biYPelsPerMeter: 0 as LONG,
                                                biClrUsed: 0 as DWORD,
                                                biClrImportant: 0 as DWORD,
                                            },
                                            bmiColors: 0 as *mut RGBQUAD,
                                           },
                                     memory: 0 as *mut c_void,
                                     height: 0 as c_int,
                                     width: 0 as c_int,
                                     pitch: 0 as c_int,
                                    };

//Graphics System constants
const BYTES_PER_PIXEL: c_int = 4;

//Sound System constants
const CHANNELS: WORD = 2;
const BITS_PER_CHANNEL: WORD = 16;
const SAMPLES_PER_SECOND: DWORD = 48000;
const TONE_FREQUENCY: DWORD = 261;
const SQUARE_WAVE_PERIOD: DWORD = SAMPLES_PER_SECOND/TONE_FREQUENCY;
const BYTES_PER_SAMPLE: DWORD = 4;
const VOLUME: i16 = 3000;

struct Backbuffer {
    info: BITMAPINFO,
    memory: *mut c_void,
    height: c_int,
    width: c_int,
    pitch: c_int,
}

//Stub functions if none of the XInput libraries could be loaded!
extern "system" fn xinput_get_state_stub(_: DWORD, _: *mut XINPUT_STATE) -> DWORD { ERROR_DEVICE_NOT_CONNECTED }
extern "system" fn xinput_set_state_stub(_: DWORD, _: *mut XINPUT_VIBRATION) -> DWORD { ERROR_DEVICE_NOT_CONNECTED }


//TODO: instead of giving the buffer_size in here extract it directly from sound_buffer
//GetCaps method returns a DSBCAPS which in turn contains the buffer size
fn generate_sound(sound_buffer: *mut IDirectSoundBuffer, buffer_size: DWORD, 
                  sample_index: &mut DWORD) {
    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0; 
    let mut region2_size: DWORD = 0; 

    let mut write_cursor: DWORD = 0;
    let mut play_cursor: DWORD = 0;
    let byte_to_lock: DWORD = *sample_index * BYTES_PER_SAMPLE % buffer_size; 

    unsafe {
        
        if SUCCEEDED(((*(*sound_buffer).lpVtbl).GetCurrentPosition)
                            (sound_buffer, &mut play_cursor, &mut write_cursor)) {

            //TODO: In release mode we collide with the cursor. Fast Fixup
            //is to substract one BYTES_PER_SAMPLE from this value!
            let bytes_to_write = if byte_to_lock >= play_cursor {
                                     buffer_size - byte_to_lock + play_cursor
                                 } else {
                                     play_cursor - byte_to_lock
                                 };
            ((*(*sound_buffer).lpVtbl).Lock)(sound_buffer, 
                                             byte_to_lock,
                                             bytes_to_write,
                                             &mut region1, &mut region1_size,
                                             &mut region2, &mut region2_size,
                                             0 as DWORD);
        }
    }

    assert!((region1_size % BYTES_PER_SAMPLE) == 0);
    let region1_sample_count = region1_size/BYTES_PER_SAMPLE;
    let mut out = region1 as *mut i16;
    for _ in range(0, region1_sample_count) {
        let value = if ((*sample_index / (SQUARE_WAVE_PERIOD/2)) % 2) == 0 {
                        VOLUME
                    } else {
                        -VOLUME
                    };
        *sample_index += 1;

        unsafe {
            *out = value;
            out = out.offset(1);
            *out = value;
            out = out.offset(1);
        }
    }

    assert!((region2_size % BYTES_PER_SAMPLE) == 0);
    let region2_sample_count = region2_size/BYTES_PER_SAMPLE;
    out = region2 as *mut i16;
    for _ in range(0, region2_sample_count) {
        let value = if ((*sample_index / (SQUARE_WAVE_PERIOD/2)) % 2) == 0 {
                        VOLUME
                    } else {
                        -VOLUME
                    };
        *sample_index += 1; 

        unsafe {
            *out = value;
            out = out.offset(1);
            *out = value;
            out = out.offset(1);
        }
    }

    unsafe {
        ((*(*sound_buffer).lpVtbl).Unlock)(sound_buffer,
                                           region1, region1_size,
                                           region2, region2_size);
    }
}

fn dsound_init(window: HWND, buffer_size_bytes: DWORD, 
               samples_per_second: DWORD) -> Option<*mut IDirectSoundBuffer> {
    let library_name = "dsound.dll".to_c_str();
    let library = unsafe { LoadLibraryA(library_name.as_ptr()) };

    if library.is_not_null() {
        let create_name = "DirectSoundCreate".to_c_str();
        let ds_create = unsafe { GetProcAddress(library, create_name.as_ptr()) };
        if ds_create.is_null() { return None; }

        //We have DirectSound capabilities
        let DirectSoundCreate: DirectSoundCreate_t = unsafe { mem::transmute(ds_create) };
        let mut direct_sound: *mut IDirectSound = ptr::null_mut();
        if SUCCEEDED(DirectSoundCreate(ptr::null(), &mut direct_sound, ptr::null_mut())) {
            //Creating the primary buffer and setting our format
            let buffer_desc: DSBUFFERDESC = DSBUFFERDESC {
                                                dwSize: std::mem::size_of::<DSBUFFERDESC>() as DWORD,
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
                        dwSize: std::mem::size_of::<DSBUFFERDESC>() as DWORD,
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

            return Some(secondary_buffer)
        }
    }

    None
}

fn load_xinput_functions() -> (XInputGetState_t, XInputSetState_t) {

    let xlib_first_name = "xinput1_4.dll".to_c_str();
    let xlib_second_name = "xinput1_3.dll".to_c_str();
    let xlib_third_name = "xinput9_1_0.dll".to_c_str();

    let mut module = unsafe { LoadLibraryA( xlib_first_name.as_ptr() ) };
    
    if module.is_null() {
        module = unsafe { LoadLibraryA( xlib_second_name.as_ptr() ) };
        if module.is_null() {
            module = unsafe { LoadLibraryA( xlib_third_name.as_ptr() ) };
        }
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

fn render_weird_gradient(buffer: &Backbuffer, green_offset: c_int, blue_offset: c_int) {
    let mut row = buffer.memory as *mut u8;

    for y in range(0, buffer.height) {
        let mut pixel = row as *mut u32;
        for x in range(0, buffer.width) {
            let green = (y + green_offset) as u8;
            let blue = (x + blue_offset) as u8;
            
            unsafe { 
                *pixel = (green as u32 << 8)  | blue as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe { row = row.offset(buffer.pitch as int); }
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
                            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
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

    let bitmap_size = buffer.width * buffer.height * BYTES_PER_PIXEL;

    unsafe {
        buffer.memory = VirtualAlloc(0 as LPVOID, bitmap_size as SIZE_T,
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

extern "system" fn process_messages(window: HWND, message: UINT, 
                                    wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    let mut res: LRESULT = 0;

    match message {
        WM_DESTROY => unsafe {running = false},
        WM_CLOSE => unsafe {running = false},

        WM_SYSKEYDOWN |
        WM_SYSKEYUP   |
        WM_KEYDOWN    |
        WM_KEYUP       => {
            let vk_code = wparam as u8;
            let was_down = (lparam & (1 << 30)) != 0;
            let is_down = (lparam & (1 << 31)) == 0;

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
                    VK_UP => (),
                    VK_DOWN => (),
                    VK_LEFT => (),
                    VK_RIGHT => (),
                    W => (),
                    A => (),
                    S => (),
                    D => (),
                    E => (),
                    Q => (),
                    VK_ESCAPE => (),
                    VK_SPACE => (),
                    VK_F4 => {
                        let alt_key_down = (lparam & (1 << 29)) != 0;
                        if alt_key_down {
                            unsafe { running = false; }
                        }
                    },
                    _ => (),
                }
            }
        },


        WM_PAINT => { 
            let mut paint = Default::default(); 

            let context = unsafe { BeginPaint(window, &mut paint) };
            if context.is_null() {
                panic!("BeginPaint failed!");
            }

            let (width, height) = get_client_dimensions(window).unwrap();
            unsafe { 
                blit_buffer_to_window(context, &back_buffer, width, height);
                EndPaint(window, &paint);
            }
        },

        _ => unsafe { 
                res = DefWindowProcA(window, message, wparam, lparam);
        },
    }
    
    res
}

fn main() {
    let (XInputGetState, XInputSetState) = load_xinput_functions();
    
    let module_handle = unsafe { GetModuleHandleA(ptr::null()) };
    if module_handle.is_null() {
        panic!("Handle to our executable could not be obtained!");
    }
    

    unsafe { resize_dib_section(&mut back_buffer, 1280, 720); }

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

    let window = unsafe {
        CreateWindowExA(0 as DWORD, window_class.lpszClassName, 
                        window_title.as_ptr(),
                        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                        CW_USEDEFAULT, CW_USEDEFAULT, 
                        CW_USEDEFAULT, CW_USEDEFAULT, 
                        0 as HWND, 0 as HWND, module_handle, ptr::null_mut())
    };

    if window.is_null() {
        panic!("Window could not be created!");
    }

    //Needed for Sound System test
    let mut sample_index: DWORD = 0;
    let buffer_size = SAMPLES_PER_SECOND * BYTES_PER_SAMPLE;

    //TODO: Handle all the cases when there was no DirectSound and unwrap fails!
    let sound_buffer = dsound_init(window, buffer_size, SAMPLES_PER_SECOND).unwrap();
    unsafe { ((*(*sound_buffer).lpVtbl).Play)(sound_buffer, 0, 0, DSBPLAY_LOOPING); }

    let context = unsafe { GetDC(window) };
    if context.is_null() {
        panic!("DC for the Window not available!");
    }
    
    let mut msg = Default::default();

    //Needed for graphics test
    let mut green_offset: c_int = 0;
    let mut blue_offset: c_int = 0;


    unsafe {
        running = true;
        while running {
            while PeekMessageA(&mut msg, 0 as HWND,
                               0 as UINT, 0 as UINT, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    running = false;
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            for controller in range(0, XUSER_MAX_COUNT) {
                let mut state = Default::default();
                let res: u32 = XInputGetState(controller, &mut state);
                match res {
                    //Case the Controller is connected and we got data
                    ERROR_SUCCESS => {
                        let gamepad = &state.Gamepad;

                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_UP) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_START) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_BACK) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_A) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_B) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_X) != 0;
                        let _ = (gamepad.wButtons & XINPUT_GAMEPAD_Y) != 0;

                        let stick_x = gamepad.sThumbLX;
                        let stick_y = gamepad.sThumbLY;

                        green_offset -= stick_y as c_int >> 12;
                        blue_offset += stick_x as c_int >> 12;
                    },

                    //Case the Controller is not connected
                    ERROR_DEVICE_NOT_CONNECTED => (),


                    //Some arbitrary Error was found
                    _ => (),
                }
            }

            render_weird_gradient(&back_buffer, green_offset, blue_offset);

            generate_sound(sound_buffer, buffer_size, &mut sample_index);

            let (width, height) = get_client_dimensions(window).unwrap();
            blit_buffer_to_window(context, &back_buffer, width, height);
        }
    }
}
