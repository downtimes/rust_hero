#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub use libc::{c_int, c_uint, c_long, c_void};
pub use libc::{HANDLE, LPCSTR, WORD, DWORD, LPVOID, BOOL, LONG, BYTE};
pub use std::default::Default;

#[cfg(target_arch = "x86")]
pub mod pointer {
    pub type LONG_PTR = c_long;
    pub type UINT_PTR = c_uint;
}
#[cfg(target_arch = "x86_64")]
pub mod pointer {
    pub type LONG_PTR = i64;
    pub type UINT_PTR = u64;
}


pub type UINT = c_uint;
pub type HINSTANCE = HANDLE;
pub type HBITMAP = HANDLE;
pub type HMODULE = HINSTANCE;
pub type HICON = HANDLE;
pub type HCURSOR = HANDLE;
pub type HBRUSH = HANDLE;
pub type HWND = HANDLE;
pub type HMENU = HANDLE;
pub type LPCTSTR = LPCSTR;
pub type LPARAM = pointer::LONG_PTR;
pub type LRESULT = pointer::LONG_PTR;
pub type WPARAM = pointer::UINT_PTR;
pub type ATOM = WORD;
pub type LPMSG = *mut MSG;
pub type LPPAINTSTRUCT = *mut PAINTSTRUCT;
pub type HDC = HANDLE;
pub type LPRECT = *mut RECT;
pub type HGDIOBJ = HANDLE;

type WNDPROC = extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;


pub const WM_CLOSE: UINT = 0x0010;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_ACTIVATEAPP: UINT = 0x001C;


pub const DIB_RGB_COLORS: UINT = 0;

pub const BI_RGB: DWORD = 0;

pub const SRCCOPY: DWORD = 0x00CC0020;

pub const WS_OVERLAPPED: DWORD = 0x00000000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const WS_THICKFRAME: DWORD = 0x00040000;
pub const WS_MINIMIZEBOX: DWORD = 0x00020000;
pub const WS_MAXIMIZEBOX: DWORD = 0x00010000;
pub const WS_OVERLAPPEDWINDOW: DWORD = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU |
                                     WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_VISIBLE: DWORD = 0x10000000;

pub const BLACKNESS: DWORD = 0x00000042;
pub const WHITENESS: DWORD = 0x00FF0062;

#[allow(overflowing_literals)]
pub const CW_USEDEFAULT: c_int = 0x80000000;

#[repr(C)]
pub struct WNDCLASS {
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCTSTR,
    pub lpszClassName: LPCTSTR,
}

#[repr(C)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
    pub time: DWORD,
    pub point: POINT,
}

impl Default for MSG{
    fn default() -> MSG {
        MSG{hwnd: 0 as HWND,
            message: 0 as UINT,
            wparam: 0 as WPARAM,
            lparam: 0 as LPARAM,
            time: 0 as DWORD,
            point: POINT{x: 0, y: 0}}
    }
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [BYTE, ..32],
}

impl Default for PAINTSTRUCT {
    fn default() -> PAINTSTRUCT {
        PAINTSTRUCT{hdc: 0 as HDC,
                    fErase: 0 as BOOL,
                    rcPaint: Default::default(),
                    fRestore: 0 as BOOL,
                    fIncUpdate: 0 as BOOL,
                    rgbReserved: [0 as BYTE, ..32]}
    }
}


#[repr(C)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

impl Default for RECT {
    fn default() -> RECT {
        RECT { left: 0, right:0,
               bottom: 0, top: 0 }
    }
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: WORD,
    pub biBitCount: WORD,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}

#[repr(C)]
pub struct RGBQUAD {
    pub rgbBlue: BYTE,
    pub rgbGreen: BYTE,
    pub rgbRed: BYTE,
    pub rgbReserved: BYTE,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: *mut RGBQUAD,
}

//user32 and kernel32
extern "system" {
    pub fn GetModuleHandleA(lpModuleName: LPCTSTR) -> HMODULE;
    pub fn DefWindowProcA(window: HWND, message: UINT, 
                         wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    pub fn RegisterClassA(class: *const WNDCLASS) -> ATOM;
    pub fn CreateWindowExA(exStyle: DWORD, className: LPCTSTR, windowName : LPCTSTR,
                          style: DWORD, x: c_int, y: c_int, width: c_int,
                          height: c_int, parent: HWND, menu: HMENU,
                          instance: HINSTANCE, param: LPVOID) -> HWND;
    pub fn GetMessageA(msg: LPMSG, hwnd: HWND, msgFilterMin: UINT, msgFilterMax: UINT) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageA(msg: *const MSG) -> LRESULT;
    pub fn GetClientRect(hwnd: HWND, lpRect: LPRECT) -> BOOL;
}

// gdi32
#[link(name = "gdi32")]
extern "system" {
    pub fn BeginPaint(hwnd: HWND, lpPaint: LPPAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn PatBlt(hdc: HDC, nXLeft: c_int, nYLeft: c_int, nWidth: c_int, nHeight: c_int,
                  dwRop: DWORD) -> BOOL;
    pub fn CreateDIBSection(hdc: HDC, pbmi: *const BITMAPINFO, iUsage: UINT,
                            pvBits: *mut *mut c_void, hSection: HANDLE,
                            dwOffset: DWORD) -> HBITMAP;
    pub fn StretchDIBits(hdc: HDC, XDest: c_int, YDEst: c_int, nDestWidth: c_int,
                        nDestHeight: c_int, XSrc: c_int, YSrc: c_int, nSrcWidth: c_int,
                        nSrcHeight: c_int, lpBits: *const c_void,
                        lpBitsInfo: *const BITMAPINFO, iUsage: UINT,
                        dwRop: DWORD) -> c_int;
    pub fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;

}

