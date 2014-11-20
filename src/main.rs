extern crate libc;
use std::ptr;

mod ffi;


extern "system" fn process_messages(window: ffi::HWND, message: ffi::UINT, 
                                    wparam: ffi::WPARAM, lparam: ffi::LPARAM) -> ffi::LRESULT {

    let mut res: ffi::LRESULT = 0;

    match message {
        ffi::WM_SIZE => (),
        ffi::WM_DESTROY => (),
        ffi::WM_ACTIVATEAPP => (),
        ffi::WM_PAINT => unsafe { let mut paint = ffi::PAINTSTRUCT {hdc: 0 as ffi::HDC,
                                                        fErase: 0 as ffi::BOOL,
                                                        rcPaint: ffi::RECT { left: 0, top: 0, right: 0, bottom: 0},
                                                        fRestore: 0 as ffi::BOOL,
                                                        fIncUpdate: 0 as ffi::BOOL,
                                                        rgbReserved: [0 as ffi::BYTE, ..32]};
                           let context = ffi::BeginPaint(window, &mut paint);
                           let x: libc::c_int = paint.rcPaint.left;
                           let y: libc::c_int = paint.rcPaint.top;
                           let width: libc::c_int = paint.rcPaint.right - paint.rcPaint.left;
                           let height: libc::c_int = paint.rcPaint.bottom - paint.rcPaint.top;

                           ffi::PatBlt(context, x, y, width, height, ffi::WHITENESS); 
                         
                           ffi::EndPaint(window, &paint); },

        _ => unsafe { res = ffi::DefWindowProcA(window, message, wparam, lparam); },
    }
    
    res
}

fn main() {
    unsafe {
        let module_handle = ffi::GetModuleHandleA(ptr::null());
        let class_str = "HandmadeHeroWindowClass".to_c_str();
        let window_class = ffi::WNDCLASS { style: 0 as ffi::UINT,
                                           lpfnWndProc: process_messages,
                                           cbClsExtra: 0 as libc::c_int,
                                           cbWndExtra: 0 as libc::c_int,
                                           hInstance: module_handle,
                                           hIcon: 0 as ffi::HICON,
                                           hCursor: 0 as ffi::HCURSOR,
                                           hbrBackground: 0 as ffi::HBRUSH,
                                           lpszMenuName: 0 as ffi::LPCTSTR,
                                           lpszClassName: class_str.as_ptr()};

        ffi::RegisterClassA(&window_class);
        let window_title = "Handmade Hero".to_c_str();

        let _ = ffi::CreateWindowExA(0 as ffi::DWORD, window_class.lpszClassName, 
                                                 window_title.as_ptr(),
                                                 ffi::WS_OVERLAPPEDWINDOW | ffi::WS_VISIBLE,
                                                 ffi::CW_USEDEFAULT, ffi::CW_USEDEFAULT, 
                                                 ffi::CW_USEDEFAULT, ffi::CW_USEDEFAULT, 
                                                 0 as ffi::HWND, 0 as ffi::HWND, module_handle, ptr::null_mut());
        
        loop {
            let mut msg = ffi::MSG {hwnd: 0 as ffi::HWND,
                                message: 0 as ffi::UINT,
                                wparam: 0 as ffi::WPARAM,
                                lparam: 0 as ffi::LPARAM,
                                time: 0 as ffi::DWORD,
                                point: ffi::POINT{x: 0, y: 0} };
            let ret = ffi::GetMessageA(&mut msg, 0 as ffi::HWND, 0 as ffi::UINT, 0 as ffi::UINT);

            if ret > 0 {
                ffi::TranslateMessage(&msg);
                ffi::DispatchMessageA(&msg);
            } else {
                break;
            }
        }
    }
}
