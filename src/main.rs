#![feature(globs)]
extern crate libc;

use std::ptr;
use std::mem;
use ffi::*;

mod ffi;


//TODO:These are a global for now needs to cleanup later on by packing them in a window
//struct. See SFML Source code for a solution to the "global problem" 
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

const BYTES_PER_PIXEL: c_int = 4;

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
                                     MEM_COMMIT, PAGE_READWRITE);
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

    unsafe { running = true; }
        
    let context = unsafe { GetDC(window) };
    if context.is_null() {
        panic!("DC for the Window not available!");
    }
    
    let mut msg = Default::default();

    let mut green_offset: c_int = 0;
    let mut blue_offset: c_int = 0;

    unsafe {
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

                        let dpad_up = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_UP) != 0;
                        let dpad_down = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN) != 0;
                        let dpad_left = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT) != 0;
                        let dpad_right = (gamepad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT) != 0;
                        let start = (gamepad.wButtons & XINPUT_GAMEPAD_START) != 0;
                        let back = (gamepad.wButtons & XINPUT_GAMEPAD_BACK) != 0;
                        let left_shoulder = (gamepad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER) != 0;
                        let right_shoulder = (gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        let abutton = (gamepad.wButtons & XINPUT_GAMEPAD_A) != 0;
                        let bbutton = (gamepad.wButtons & XINPUT_GAMEPAD_B) != 0;
                        let xbutton = (gamepad.wButtons & XINPUT_GAMEPAD_X) != 0;
                        let ybutton = (gamepad.wButtons & XINPUT_GAMEPAD_Y) != 0;

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

            let (width, height) = get_client_dimensions(window).unwrap();
            blit_buffer_to_window(context, &back_buffer, width, height);
        }
    }
}
