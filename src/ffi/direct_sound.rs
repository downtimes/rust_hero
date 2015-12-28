pub use libc::c_void;
pub use libc::{BYTE, HANDLE, WORD, DWORD, LONG};

use std::default::Default;

pub type HRESULT = LONG;
pub type HWND = HANDLE;

pub type DirectSoundCreate_t = extern "system" fn(pcGuidDevice: *const GUID,
                                                  ppDSC: *mut *mut IDirectSound,
                                                  pUnkOuter: *mut IUnknown)
                                                  -> HRESULT;

pub const DSBCAPS_PRIMARYBUFFER: DWORD = 0x00000001;
pub const DSBCAPS_GETCURRENTPOSITION2: DWORD = 0x00010000;

pub const DSBPLAY_LOOPING: DWORD = 0x00000001;

pub const WAVE_FORMAT_PCM: WORD = 1;

pub const DSSCL_PRIORITY: DWORD = 0x00000002;

pub const DS_OK: HRESULT = 0x00000000;


pub fn SUCCEEDED(hr: HRESULT) -> bool {
    hr as i32 >= 0
}

#[repr(C)]
pub struct IDirectSoundVtbl {
    // IUnknown sizeerface Functions which we don't use so we just use c_void
    // posizeers instead of the actual function signature
    pub Querysizeerface: *const c_void,
    pub AddRef: *const c_void,
    pub Release: *const c_void,

    // Functions from this sizeerface we only implement the once we actually use
    pub CreateSoundBuffer: extern "system" fn(this: *mut IDirectSound,
                                              ds_buffer_desc: *const DSBUFFERDESC,
                                              ds_buffer: *mut *mut IDirectSoundBuffer,
                                              pUnkOuter: *mut IUnknown)
                                              -> HRESULT,
    pub GetCaps: *const c_void,
    pub DuplicateSoundBuffer: *const c_void,
    pub SetCooperativeLevel: extern "system" fn(this: *mut IDirectSound,
                                                hwnd: HWND,
                                                dwLevel: DWORD)
                                                -> HRESULT,
    pub Compact: *const c_void,
    pub GetSpeakerConfig: *const c_void,
    pub SetSpeakerConfig: *const c_void,
    pub Initialize: *const c_void,
}

#[repr(C)]
pub struct DSBCAPS {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwUnlockTransferRate: DWORD,
    pub dwPlayCpuOverhead: DWORD,
}

impl Default for DSBCAPS {
    fn default() -> DSBCAPS {
        DSBCAPS {
            dwSize: 0,
            dwFlags: 0,
            dwBufferBytes: 0,
            dwUnlockTransferRate: 0,
            dwPlayCpuOverhead: 0,
        }
    }
}

#[repr(C)]
pub struct DSBUFFERDESC {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwReserved: DWORD,
    pub lpwfxFormat: *mut WAVEFORMATEX,
    pub guid: GUID,
}

#[repr(C)]
pub struct WAVEFORMATEX {
    pub wFormatTag: WORD,
    pub nChannels: WORD,
    pub nSamplesPerSec: DWORD,
    pub nAvgBytesPerSec: DWORD,
    pub nBlockAlign: WORD,
    pub wBitsPerSample: WORD,
    pub cbSize: WORD,
}

#[repr(C)]
pub struct IDirectSound {
    pub lpVtbl: *const IDirectSoundVtbl,
}

#[repr(C)]
pub struct IDirectSoundBufferVtbl {
    // IUnknown sizeerface Functions which we don't use so we just use c_void
    // posizeers instead of the actual function signature
    pub Querysizeerface: *const c_void,
    pub AddRef: *const c_void,
    pub Release: *const c_void,

    // Functions from this sizeerface we only implement the once we actually use
    pub GetCaps: extern "system" fn(this: *mut IDirectSoundBuffer, pDSBufferCaps: *mut DSBCAPS)
                                    -> HRESULT,
    pub GetCurrentPosition: extern "system" fn(this: *mut IDirectSoundBuffer,
                                               pdwCurrentPlayCursor: *mut DWORD,
                                               pdwCurrentWriteCursor: *mut DWORD)
                                               -> HRESULT,
    pub GetFormat: *const c_void,
    pub GetVolume: *const c_void,
    pub GetPan: *const c_void,
    pub GetFrequency: *const c_void,
    pub GetStatus: *const c_void,
    pub Initialize: *const c_void,
    pub Lock: extern "system" fn(this: *mut IDirectSoundBuffer,
                                 dwOffset: DWORD,
                                 dwBytes: DWORD,
                                 ppvAudioPtr1: *mut *mut c_void,
                                 pdwAudioBytes1: *mut DWORD,
                                 ppvAudioPtr2: *mut *mut c_void,
                                 pdwAudioBytes2: *mut DWORD,
                                 dwFlags: DWORD)
                                 -> HRESULT,
    pub Play: extern "system" fn(this: *mut IDirectSoundBuffer,
                                 dwReserved1: DWORD,
                                 dwPriority: DWORD,
                                 dwFlags: DWORD)
                                 -> HRESULT,
    pub SetCurrentPosition: *const c_void,
    pub SetFormat: extern "system" fn(this: *mut IDirectSoundBuffer,
                                      pcfxFormat: *const WAVEFORMATEX)
                                      -> HRESULT,
    pub setVolume: *const c_void,
    pub SetPan: *const c_void,
    pub SetFrequency: *const c_void,
    pub Stop: *const c_void,
    pub Unlock: extern "system" fn(this: *mut IDirectSoundBuffer,
                                   pvAudioPtr1: *mut c_void,
                                   dwAudioBytes1: DWORD,
                                   pvAudioPtr2: *mut c_void,
                                   dwAudioBytes2: DWORD)
                                   -> HRESULT,
    pub Restore: *const c_void,
}

#[repr(C)]
pub struct IDirectSoundBuffer {
    pub lpVtbl: *const IDirectSoundBufferVtbl,
}

#[repr(C)]
pub struct IUnknown {
    // VTable Posizeer which we don't use so we just say c_void posizeer instead
    // of actual struct
    pub vtable: *const c_void,
}

#[repr(C)]
pub struct GUID {
    pub Data1: DWORD,
    pub Data2: WORD,
    pub Data3: WORD,
    pub Data4: [BYTE; 8],
}

impl Default for GUID {
    fn default() -> GUID {
        GUID {
            Data1: 0 as DWORD,
            Data2: 0 as WORD,
            Data3: 0 as WORD,
            Data4: [0 as BYTE; 8],
        }
    }
}
