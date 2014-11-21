#![feature(globs)]
extern crate libc;

use std::ptr;
use ffi::*;

mod ffi;


//TODO:These are a global for now needs to cleanup later on
static mut running: bool = false;

static mut bitmap_info: BITMAPINFO = BITMAPINFO { bmiHeader: BITMAPINFOHEADER {
                                                    biSize: 0 as DWORD,
                                                    biWidth: 0 as LONG,
                                                    biHeight: 0 as LONG,
                                                    biPlanes: 1 as WORD,
                                                    biBitCount: 32 as WORD,
                                                    biCompression: BI_RGB,
                                                    biSizeImage: 0 as DWORD,
                                                    biXPelsPerMeter: 0 as LONG,
                                                    biYPelsPerMeter: 0 as LONG,
                                                    biClrUsed: 0 as DWORD,
                                                    biClrImportant: 0 as DWORD,
                                                },
                                   bmiColors: 0 as *mut RGBQUAD,};
static mut bitmap_memory: *mut c_void = 0 as *mut c_void;
static BYTES_PER_PIXEL: c_int = 4;
static mut bitmap_height: c_int = 0;
static mut bitmap_width: c_int = 0;

fn render_weird_gradient(green_offset: c_int, blue_offset: c_int) {
    unsafe {
        let mut row = bitmap_memory as *mut u8;
        let pitch = bitmap_width * BYTES_PER_PIXEL;

        for y in range(0, bitmap_height) {
            let mut pixel = row as *mut u32;
            for x in range(0, bitmap_width) {
                let green = (y + green_offset) as u8;
                let blue = (x + blue_offset) as u8;
                
                *pixel = (green as u32 << 8)  | blue as u32 ;
                pixel = pixel.offset(1);
            }
            row = row.offset(pitch as int);
        }
    }
}


fn resize_dib_section(width: c_int, height: c_int) {
    unsafe {
        if bitmap_memory.is_not_null() {
            VirtualFree(bitmap_memory, 0 as SIZE_T , MEM_RELEASE);
        }

        bitmap_height = height;
        bitmap_width = width;

        bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        bitmap_info.bmiHeader.biHeight = -height;
        bitmap_info.bmiHeader.biWidth = width;

        let bitmap_size = width*height*BYTES_PER_PIXEL;
        bitmap_memory = VirtualAlloc(0 as LPVOID, bitmap_size as SIZE_T,
                                     MEM_COMMIT, PAGE_READWRITE);
    }
}

fn update_window(context: HDC, client_rect: RECT, x: c_int, y: c_int, width: c_int, 
                 height: c_int) {
    unsafe {
        let client_width = client_rect.right - client_rect.left;
        let client_height = client_rect.bottom - client_rect.top;
        StretchDIBits(context,
                 0, 0, bitmap_width, bitmap_height,
                 0, 0, client_width, client_height,
                 /*x, y, width, height,
                 x, y, width, height,*/
                 bitmap_memory as *const c_void,
                 &bitmap_info,
                 DIB_RGB_COLORS, SRCCOPY)
    };
}

extern "system" fn process_messages(window: HWND, message: UINT, 
                                    wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    let mut res: LRESULT = 0;

    match message {
        WM_SIZE => unsafe {
                     let mut client_rect = Default::default();
                     GetClientRect(window, &mut client_rect);
                     let width = client_rect.right - client_rect.left;
                     let height = client_rect.bottom - client_rect.top;
                     resize_dib_section(width, height)
        },

        WM_DESTROY => unsafe {running = false},
        WM_CLOSE => unsafe {running = false},
        WM_PAINT => unsafe { let mut paint = Default::default(); 
                           let context = BeginPaint(window, &mut paint);
                           let x = paint.rcPaint.left;
                           let y = paint.rcPaint.top;
                           let width = paint.rcPaint.right - paint.rcPaint.left;
                           let height = paint.rcPaint.bottom - paint.rcPaint.top;

                           let mut client_rect = Default::default();
                           GetClientRect(window, &mut client_rect);

                           update_window(context, client_rect, x, y, width, height);

                           EndPaint(window, &paint);
        },

        _ => unsafe { 
                res = DefWindowProcA(window, message, wparam, lparam);
        },
    }
    
    res
}

fn main() {
    unsafe {
        let module_handle = GetModuleHandleA(ptr::null());
        let class_str = "HandmadeHeroWindowClass".to_c_str();
        let window_class = WNDCLASS{style: 0 as UINT,
                                    lpfnWndProc: process_messages,
                                    cbClsExtra: 0 as c_int,
                                    cbWndExtra: 0 as c_int,
                                    hInstance: module_handle,
                                    hIcon: 0 as HICON,
                                    hCursor: 0 as HCURSOR,
                                    hbrBackground: 0 as HBRUSH,
                                    lpszMenuName: 0 as LPCTSTR,
                                    lpszClassName: class_str.as_ptr()};
        RegisterClassA(&window_class);
        let window_title = "Handmade Hero".to_c_str();

        let window = CreateWindowExA(0 as DWORD, window_class.lpszClassName, 
                                window_title.as_ptr(),
                                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                                CW_USEDEFAULT, CW_USEDEFAULT, 
                                CW_USEDEFAULT, CW_USEDEFAULT, 
                                0 as HWND, 0 as HWND, module_handle, ptr::null_mut());
        
        running = true;
        let mut msg = Default::default();

        let mut green_offset: c_int = 0;
        let mut blue_offset: c_int = 0;

        while running {
            while PeekMessageA(&mut msg, 0 as HWND,
                               0 as UINT, 0 as UINT, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    running = false;
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            render_weird_gradient(green_offset, blue_offset);

            let context = GetDC(window);

            let mut client_rect = Default::default();
            GetClientRect(window, &mut client_rect);
            let client_height: c_int = client_rect.bottom - client_rect.top;
            let client_width: c_int = client_rect.right - client_rect.left;

            update_window(context, client_rect, 0, 0, client_width, client_height);

            ReleaseDC(window, context);

            green_offset += 1;
            blue_offset += 2;
        }
    }
}
