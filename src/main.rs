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
static mut bitmap_handle: HBITMAP = 0 as *mut c_void;
static mut bitmap_memory: *mut c_void = 0 as *mut c_void;
static mut device_context: HDC = 0 as *mut c_void;


fn resize_dib_section(width: libc::c_int, height: libc::c_int) {
    unsafe {
        if bitmap_handle.is_not_null()  {
            DeleteObject(bitmap_handle);
        }

        if device_context.is_null() {
            device_context = CreateCompatibleDC(0 as HDC);
        }

        bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        bitmap_info.bmiHeader.biHeight = height;
        bitmap_info.bmiHeader.biWidth = width;
        bitmap_handle = CreateDIBSection(device_context, &bitmap_info, DIB_RGB_COLORS,
                                         &mut bitmap_memory, 0 as HANDLE, 0 as DWORD);
    }
}

fn update_window(context: HDC, x: libc::c_int, y: libc::c_int, width: libc::c_int, 
                 height: libc::c_int) {
    unsafe {
        StretchDIBits(context,
                 x, y, width, height,
                 x, y, width, height,
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

                           update_window(context, x, y, width, height);

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
                                    cbClsExtra: 0 as libc::c_int,
                                    cbWndExtra: 0 as libc::c_int,
                                    hInstance: module_handle,
                                    hIcon: 0 as HICON,
                                    hCursor: 0 as HCURSOR,
                                    hbrBackground: 0 as HBRUSH,
                                    lpszMenuName: 0 as LPCTSTR,
                                    lpszClassName: class_str.as_ptr()};
        RegisterClassA(&window_class);
        let window_title = "Handmade Hero".to_c_str();

        let _ = CreateWindowExA(0 as DWORD, window_class.lpszClassName, 
                                window_title.as_ptr(),
                                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                                CW_USEDEFAULT, CW_USEDEFAULT, 
                                CW_USEDEFAULT, CW_USEDEFAULT, 
                                0 as HWND, 0 as HWND, module_handle, ptr::null_mut());
        
        running = true;
        while running {
            let mut msg = Default::default();
            let ret = GetMessageA(&mut msg, 0 as HWND, 0 as UINT, 0 as UINT);

            if ret > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            } else {
                break;
            }
        }
    }
}
