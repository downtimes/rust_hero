use libc::{c_int, UINT}

#[repr(C)]
pub struct WNDCLASS {
    style: UINT,
    lpFnWndProc: WNDPROC ,
    cbClsExtra: c_int,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackround: HBRUSH,
    lpszMenuName: LPCTSTR,
    lpszClassName: LPCTSTR,
}
