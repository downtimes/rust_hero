#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub use libc::{c_int, c_uint, c_long, c_char};
pub use libc::{LPCSTR, LPVOID, BOOL, SIZE_T};
pub use libc::{QueryPerformanceCounter, QueryPerformanceFrequency};
pub use libc::{VirtualAlloc, VirtualFree};
pub use libc::{CloseHandle, ReadFile, WriteFile};
pub use std::default::Default;

pub use self::direct_sound::*;
pub use self::pointer::{LONG_PTR, UINT_PTR};

pub mod direct_sound;

#[cfg(target_arch = "x86")]
pub mod pointer {
    use libc::{c_long, c_uint};
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
pub type LPTSTR = *mut c_char;
pub type HGDIOBJ = HANDLE;
pub type MMRESULT = UINT;
pub type SHORT = i16;

type WNDPROC = extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
pub type XInputGetState_t = extern "system" fn(DWORD, *mut XINPUT_STATE) -> DWORD;
pub type XInputSetState_t = extern "system" fn(DWORD, *mut XINPUT_VIBRATION) -> DWORD;


pub const WM_CREATE: UINT = 0x0001;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_ACTIVATEAPP: UINT = 0x001C;
pub const WM_QUIT: UINT = 0x0012;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_SYSKEYDOWN: UINT = 0x0104;
pub const WM_SYSKEYUP: UINT = 0x0105;

pub const GWLP_USERDATA: c_int = -21;

pub const VK_LBUTTON: u8 = 0x01u8;
pub const VK_RBUTTON: u8 = 0x02u8;
pub const VK_MBUTTON: u8 = 0x04u8;
pub const VK_XBUTTON1: u8 = 0x05u8;
pub const VK_XBUTTON2: u8 = 0x06u8;
pub const VK_ESCAPE: u8 = 0x1Bu8;
pub const VK_SPACE: u8 = 0x20u8;
pub const VK_LEFT: u8 = 0x25u8;
pub const VK_UP: u8 = 0x26u8;
pub const VK_RIGHT: u8 = 0x27u8;
pub const VK_DOWN: u8 = 0x28u8;
pub const VK_F4: u8 = 0x73u8;

pub const CS_OWNDC: UINT = 0x0020;
pub const CS_HREDRAW: UINT = 0x0002;
pub const CS_VREDRAW: UINT = 0x0001;

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

pub const MEM_COMMIT: DWORD = 0x00001000;
pub const MEM_RESERVE: DWORD = 0x00002000;
pub const MEM_RELEASE: DWORD = 0x00008000;

pub const PAGE_READWRITE: DWORD = 0x00000004;

pub const PM_REMOVE: DWORD = 0x00000001;

pub const XINPUT_GAMEPAD_DPAD_UP: WORD = 0x0001;
pub const XINPUT_GAMEPAD_DPAD_DOWN: WORD = 0x0002;
pub const XINPUT_GAMEPAD_DPAD_LEFT: WORD = 0x0004;
pub const XINPUT_GAMEPAD_DPAD_RIGHT: WORD = 0x0008;
pub const XINPUT_GAMEPAD_START: WORD = 0x0010;
pub const XINPUT_GAMEPAD_BACK: WORD = 0x0020;
pub const XINPUT_GAMEPAD_LEFT_SHOULDER: WORD = 0x0100;
pub const XINPUT_GAMEPAD_RIGHT_SHOULDER: WORD = 0x0200;
pub const XINPUT_GAMEPAD_A: WORD = 0x1000;
pub const XINPUT_GAMEPAD_B: WORD = 0x2000;
pub const XINPUT_GAMEPAD_X: WORD = 0x4000;
pub const XINPUT_GAMEPAD_Y: WORD = 0x8000;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

pub const XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE: SHORT = 7849;

pub const XUSER_MAX_COUNT: DWORD = 4;

pub const ERROR_SUCCESS: DWORD = 0;
pub const ERROR_DEVICE_NOT_CONNECTED: DWORD = 1167;

pub const GENERIC_READ: DWORD = 0x80000000;
pub const GENERIC_WRITE: DWORD = 0x40000000;

pub const FILE_SHARE_READ: DWORD = 0x00000001;

pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;

pub const VREFRESH: c_int = 116;

pub const CREATE_ALWAYS: DWORD = 2;
pub const OPEN_EXISTING: DWORD = 3;

pub const MAX_PATH: uint = 260;

pub const TIMERR_NOERROR: UINT = 0;

#[allow(overflowing_literals)]
pub const CW_USEDEFAULT: c_int = 0x80000000;

pub const INVALID_HANDLE_VALUE: int = -1;

#[repr(C)]
pub struct XINPUT_GAMEPAD {
    pub wButtons: WORD,
    pub bLeftTrigger: BYTE,
    pub bRightTrigger: BYTE,
    pub sThumbLX: SHORT,
    pub sThumbLY: SHORT,
    pub sThumbRX: SHORT,
    pub sThumbRY: SHORT,
}

impl Default for XINPUT_GAMEPAD {
    fn default() -> XINPUT_GAMEPAD {
        XINPUT_GAMEPAD { 
            wButtons: 0,
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0,
        }
    }
}

#[repr(C)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}


#[repr(C)]
pub struct WIN32_FILE_ATTRIBUTE_DATA {
    pub dwFileAttributes: DWORD,
    pub ftCreateonTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHeigh: DWORD,
    pub nFileSizeLow: DWORD,
}

impl Default for WIN32_FILE_ATTRIBUTE_DATA {
    fn default() -> WIN32_FILE_ATTRIBUTE_DATA {
        WIN32_FILE_ATTRIBUTE_DATA {
            dwFileAttributes: 0,
            ftCreateonTime: FILETIME { 
                                dwLowDateTime: 0,
                                dwHighDateTime: 0,
                            },
            ftLastAccessTime: FILETIME { 
                                dwLowDateTime: 0,
                                dwHighDateTime: 0,
                            },
            ftLastWriteTime: FILETIME { 
                                dwLowDateTime: 0,
                                dwHighDateTime: 0,
                            },
            nFileSizeHeigh: 0,
            nFileSizeLow: 0,
        }
    }
}

#[repr(C)]
pub enum GET_FILEEX_INFO_LEVELS {
    GET_FILE_EX_INFO_STANDARD,
    GET_FILE_EX_MAX_INFO_LEVEL,
}

#[repr(C)]
pub struct XINPUT_STATE {
    pub dwPacketNumber: DWORD,
    pub Gamepad: XINPUT_GAMEPAD,
}


impl Default for XINPUT_STATE {
    fn default() -> XINPUT_STATE {
        XINPUT_STATE {
            dwPacketNumber: 0,
            Gamepad: Default::default(),
        }
    }
}

#[repr(C)]
pub struct XINPUT_VIBRATION {
    pub wLeftMotorSpeed: WORD,
    pub wRightMotorSpeed: WORD,
}


#[repr(C)]
pub struct CREATESTRUCT {
    pub lpCreateParams: *mut c_void,
    pub hInstance: HINSTANCE,
    pub hMenu: HMENU,
    pub hwndParent: HWND,
    pub cy: c_int,
    pub cx: c_int,
    pub y: c_int,
    pub x: c_int,
    pub style: LONG,
    pub lpszName: LPCTSTR,
    pub lpszClass: LPCTSTR,
    pub dwExStyle: DWORD,
}

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

impl Default for BITMAPINFOHEADER {
    fn default() -> BITMAPINFOHEADER {
        BITMAPINFOHEADER {
            biSize: 0,
            biWidth: 0,
            biHeight: 0,
            biPlanes: 0,
            biBitCount: 0,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        }
    }
}

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    pub nLenght: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: bool,
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

impl Default for BITMAPINFO {
    fn default() -> BITMAPINFO {
        BITMAPINFO {
            bmiHeader: Default::default(),
            bmiColors: 0 as *mut RGBQUAD,
        }
    }
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
    pub fn GetMessageA(msg: LPMSG, hwnd: HWND, msgFilterMin: UINT,
                       msgFilterMax: UINT) -> BOOL;
    pub fn PeekMessageA(msg: LPMSG, hwnd: HWND, msgFilterMin: UINT,
                        msgFIlterMax: UINT, removeMsg: UINT) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageA(msg: *const MSG) -> LRESULT;
    pub fn GetClientRect(hwnd: HWND, lpRect: LPRECT) -> BOOL;
    pub fn ReleaseDC(hWnd: HWND, hDC : HDC) -> c_int;
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn LoadLibraryA(lpFileName: LPCSTR) -> HMODULE;
    pub fn FreeLibrary(hModule: HMODULE) -> BOOL;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *const c_void;
    pub fn GetWindowLongPtrA(hWnd: HWND, nIndex: c_int) -> pointer::LONG_PTR;
    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: c_int, 
                             dwNewLong: pointer::LONG_PTR) -> pointer::LONG_PTR;
    pub fn CreateFileA(lpFileName: LPCTSTR, dwDesiredAccess: DWORD,
                      dwShareMode: DWORD, lpSecurityAttributes: *mut SECURITY_ATTRIBUTES,
                      dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD,
                      hTemplateFile: HANDLE) -> HANDLE;
    pub fn GetFileSizeEx(hFile: HANDLE, lpFileSize: *mut i64) -> BOOL;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn CopyFileA(lpExistingFileName: LPCTSTR, lpNewFileName: LPCTSTR,
                    bFeilIfExists: BOOL) -> BOOL;
    pub fn GetFileAttributesExA(lpFileName: LPCTSTR, 
                                fInfoLevelId: GET_FILEEX_INFO_LEVELS,
                                lpFileInformation: LPVOID) -> BOOL;
    pub fn CompareFileTime(lpFileTime1: *const FILETIME,
                           lpFileTime2: *const FILETIME) -> LONG;
    pub fn GetModuleFileNameA(hModule: HMODULE, lpFilename: LPTSTR,
                             nSize: DWORD) -> DWORD;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn GetKeyState(nVirtKey: c_int) -> SHORT;
    pub fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
}

// gdi32
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
    pub fn GetDeviceCaps(hdc: HDC, nIndex: c_int) -> c_int;
}


// Winmm
extern "system" {
    pub fn timeBeginPeriod(uPeriod: UINT) -> MMRESULT;
    pub fn timeEndPeriod(uPeriod: UINT) -> MMRESULT;
}

#[inline(always)]
pub mod intrinsics {
    pub fn __rdtsc() -> u64 {
        let lower: u32;
        let higher: u32;

        unsafe {
            asm!("rdtsc"
                 : "={eax}"(lower)
                   "={edx}"(higher));
        }

        ((higher as u64) << 32) | lower as u64
    }
}
