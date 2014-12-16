use std::ptr;
use std::raw::Slice;
use std::mem; 
use std::i16;

use common::{Input, GameMemory, SoundBuffer, ControllerInput, Button, VideoBuffer};
use ffi::*;

type GetSoundSamplesT = extern fn(&mut GameMemory, &mut SoundBuffer);
type UpdateAndRenderT = extern fn(&mut GameMemory, &Input, &mut VideoBuffer); 

#[cfg(not(ndebug))]
pub mod debug {
    use ffi::*;
    use std::ptr;
    use super::util;
    use common::ReadFileResult;

    pub struct SoundTimeMarker {
        pub flip_play_cursor: DWORD,
        pub flip_write_cursor: DWORD,

        pub output_play_cursor: DWORD,
        pub output_write_cursor: DWORD,

        pub output_location: DWORD,
        pub output_byte_count: DWORD,

        pub expected_frame_byte: DWORD,
    }

    impl Default for SoundTimeMarker {
        fn default() -> SoundTimeMarker {
            SoundTimeMarker {
                flip_play_cursor: 0,
                flip_write_cursor: 0,

                output_play_cursor: 0,
                output_write_cursor: 0,

                output_location: 0,
                output_byte_count: 0,

                expected_frame_byte: 0,
            }
        }
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

    unsafe fn draw_vertical(buffer: &mut super::Backbuffer,
                            x: i32, top: i32, bottom: i32, color: u32) {
        let mut pixel = buffer.memory as *mut u8;
        pixel = pixel.offset((x * super::BYTES_PER_PIXEL + top * buffer.pitch) as int);
        for _ in range(top, bottom) {
            let out = pixel as *mut u32;
            *out = color;
            pixel = pixel.offset(buffer.pitch as int);
        }
    }

    fn draw_sound_buffer_marker(c: f32, sound_size: DWORD, cursor: DWORD, 
                                buffer: &mut super::Backbuffer,
                                pad_x: i32, 
                                top: i32, bottom: i32, color: u32) {
        assert!(cursor < sound_size);
        let x: i32 = pad_x + (c * cursor as f32) as i32;
        unsafe { draw_vertical(buffer, x, top, bottom, color); }
    }

    pub fn sound_sync_display(video_backbuffer: &mut super::Backbuffer, 
                              last_time_markers: &[SoundTimeMarker],
                              current_marker: uint,
                              sound_output: &super::SoundOutput) {

        let pad_x: i32 = 16;
        let pad_y: i32 = 16;
        let line_height = 64;
        let sound_size: DWORD = sound_output.get_buffer_size();

        let c = (video_backbuffer.width - 2 * pad_x) as f32 / 
                        sound_size as f32;

        for (index, marker) in last_time_markers.iter().enumerate() {

            let play_color = 0xFFFFFFFF;
            let write_color = 0xFFFF0000;
            let expected_color = 0xFFFFFF00;

            let mut bottom = pad_y + line_height;
            let mut top = pad_y;
            if index == current_marker {
                let byte_loc_plus_count = (marker.output_location + marker.output_byte_count)
                                            % sound_size;
                let expected_pos = marker.expected_frame_byte % sound_size;

                bottom += line_height + pad_y; 
                top += line_height + pad_y;
                let firsttop = top;
                draw_sound_buffer_marker(c, sound_size, marker.output_play_cursor,
                                         video_backbuffer, pad_x, top, bottom, play_color);
                draw_sound_buffer_marker(c, sound_size, marker.output_write_cursor,
                                         video_backbuffer, pad_x, top, bottom, write_color);
                bottom += line_height + pad_y; 
                top += line_height + pad_y;

                draw_sound_buffer_marker(c, sound_size, marker.output_location,
                                         video_backbuffer, pad_x, top, bottom, play_color);
                draw_sound_buffer_marker(c, sound_size, byte_loc_plus_count,
                                         video_backbuffer, pad_x, top, bottom, write_color);
                bottom += line_height + pad_y; 
                top += line_height + pad_y;

                draw_sound_buffer_marker(c, sound_size, expected_pos,
                                         video_backbuffer, pad_x, firsttop, bottom, expected_color);

            }

            draw_sound_buffer_marker(c, sound_size, marker.flip_play_cursor,
                                     video_backbuffer, pad_x, top, bottom, play_color);
            draw_sound_buffer_marker(c, sound_size, marker.flip_write_cursor,
                                     video_backbuffer, pad_x, top, bottom, write_color);
        }
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
//TODO: get the actual refresh rate from windows reliably
const MONITOR_REFRESH_RATE: uint = 60;
const GAME_REFRESH_RATE: uint = MONITOR_REFRESH_RATE / 2;

//Sound System constants
const CHANNELS: WORD = 2;
const BITS_PER_CHANNEL: WORD = 16;
const BYTES_PER_SAMPLE: DWORD = 4;
const SOUND_BYTES_PER_SECOND: DWORD = 48000 * BYTES_PER_SAMPLE;
//TODO: see how low we can go with this value reasonably
const SOUND_SAFETY_BYTES: DWORD = (SOUND_BYTES_PER_SECOND / GAME_REFRESH_RATE as u32) / 2;

struct Game {
    handle: HMODULE,
    get_sound_samples: GetSoundSamplesT, 
    update_and_render: UpdateAndRenderT,
}

//impl Game {
//    fn is_valid(&self) -> bool {
//        self.handle.is_not_null()
//    }
//}

struct SoundOutput {
    byte_index: DWORD,
    sound_buffer: *mut IDirectSoundBuffer,
}


impl SoundOutput {
    fn get_buffer_size(&self) -> DWORD {
        let mut dsbcaps: DSBCAPS = Default::default();
        dsbcaps.dwSize = mem::size_of::<DSBCAPS>() as DWORD;

        unsafe { ((*(*self.sound_buffer).lpVtbl).GetCaps)
                        (self.sound_buffer, &mut dsbcaps); }
        
        dsbcaps.dwBufferBytes
    }
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
    pause: bool,
    backbuffer: Backbuffer,
}


//Struct to do RAII with timeBeginPeriod and timeEndPeriod on win32
struct TimePeriod(bool);

impl TimePeriod {
    pub fn new() -> TimePeriod {
        TimePeriod( 
            unsafe{ timeBeginPeriod(1) == TIMERR_NOERROR }
        )
    }

    pub fn was_set(&self) -> bool {
        let &TimePeriod(res) = self;
        res
    }
}

impl Drop for TimePeriod {
    fn drop(&mut self) {
        //Reset the Sheduler resolution back to normal
        //if it was successfully set
        let &TimePeriod(successful) = self;
        if successful {
            unsafe { timeEndPeriod(1); }
        }
    }
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
                     source: &SoundBuffer) {

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
        
        debug_assert!(bytes_to_write == region1_size + region2_size);

        let (samples_1, samples_2) = 
            source.samples.split_at(region1_size as uint / mem::size_of::<i16>());
        fill_region(region1, region1_size, sound_output, samples_1);
        fill_region(region2, region2_size, sound_output, samples_2);

        fn fill_region(region: *mut c_void, region_size: DWORD,
                       sound_output: &mut SoundOutput, source: &[i16]) {
                           
            let out: &mut [i16] = unsafe { mem::transmute( 
                                        Slice {
                                            data: region as *const i16,
                                            len: region_size as uint / mem::size_of::<i16>()
                                        }) };
            debug_assert!((region_size % BYTES_PER_SAMPLE) == 0);
            debug_assert!(out.len() == source.len());

            for (output, input) in out.iter_mut().zip(source.iter()) {
                *output = *input;
                sound_output.byte_index += 2;
            }
        }

        unsafe {
            ((*(*sound_output.sound_buffer).lpVtbl).Unlock)(sound_output.sound_buffer,
                                               region1, region1_size,
                                               region2, region2_size);
        }
    }
}

fn clear_sound_output(sound_output: &mut SoundOutput) {

    let mut region1: *mut c_void = ptr::null_mut();
    let mut region2: *mut c_void = ptr::null_mut();
    let mut region1_size: DWORD = 0; 
    let mut region2_size: DWORD = 0; 

    let lock = unsafe { 
        ((*(*sound_output.sound_buffer).lpVtbl).Lock)(sound_output.sound_buffer, 
                                         0,
                                         sound_output.get_buffer_size(),
                                         &mut region1, &mut region1_size,
                                         &mut region2, &mut region2_size,
                                         0 as DWORD)
    };

    if SUCCEEDED(lock) {


        fill_region(region1, region1_size);
        fill_region(region2, region2_size);

        fn fill_region(region: *mut c_void, region_size: DWORD) {
            let out: &mut [i32] = unsafe { mem::transmute( 
                                        Slice {
                                            data: region as *const i32,
                                            len: (region_size / BYTES_PER_SAMPLE) as uint, 
                                        }) };
            for sample in out.iter_mut() {
                *sample = 0;
            }
        }

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
                    dwFlags: DSBCAPS_GETCURRENTPOSITION2,
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


//Stub functons if none of the game Code could be loaded! 
extern fn get_sound_samples_stub(_: &mut GameMemory, _: &mut SoundBuffer) { }
extern fn update_and_render_stub(_: &mut GameMemory, _: &Input, _: &mut VideoBuffer) { } 


fn load_game_functions() -> Game {
    let game_dll_name = "game.dll".to_c_str();
    let temp_dll_name = "game_temp.dll".to_c_str();

    unsafe { CopyFileA(game_dll_name.as_ptr(), temp_dll_name.as_ptr(), FALSE); }

    let mut result = Game {
                        handle: ptr::null_mut(),
                        get_sound_samples: get_sound_samples_stub,
                        update_and_render: update_and_render_stub,
                    };

    result.handle = unsafe { LoadLibraryA( temp_dll_name.as_ptr() ) };

    if result.handle.is_not_null() {
        let get_sound_samples_name = "get_sound_samples".to_c_str();
        let update_and_render_name = "update_and_render".to_c_str();

        let get_sound_samples  = unsafe { GetProcAddress(result.handle, get_sound_samples_name.as_ptr() ) };
        let update_and_render = unsafe { GetProcAddress(result.handle, update_and_render_name.as_ptr() ) };

        if get_sound_samples.is_not_null() && update_and_render.is_not_null() {
            unsafe {
                result.get_sound_samples = mem::transmute(get_sound_samples); 
                result.update_and_render = mem::transmute(update_and_render);
            }
        }
    }
    result
}


fn unload_game_functions(game: &mut Game) {
    if game.handle.is_not_null() {
        unsafe { FreeLibrary(game.handle); }
        game.handle = ptr::null_mut();
    }
    game.get_sound_samples = get_sound_samples_stub;
    game.update_and_render = update_and_render_stub;
}

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

fn process_xinput(XInputGetState: XInputGetState_t,
                  new_controllers: &mut [ControllerInput], 
                  old_controllers: &[ControllerInput]) {

    debug_assert!(XUSER_MAX_COUNT >= new_controllers.len() as u32);

    for (index, controller) in new_controllers.iter_mut().enumerate() {
        let mut state = Default::default();

        let res: u32 = XInputGetState(index as u32, &mut state);
        match res {
            //Case the Controller is connected and we got data
            ERROR_SUCCESS => {
                let old_controller = &old_controllers[index];
                controller.is_connected = true;

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
                    controller.average_x = Some(xvalue);
                    controller.average_y = Some(yvalue);
                } else {
                    controller.average_x = None;
                    controller.average_y = None;
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
                                       &mut controller.move_up,
                                       1);
                process_xinput_button(down_fake,
                                       &old_controller.move_down,
                                       &mut controller.move_down,
                                       1);
                process_xinput_button(left_fake,
                                       &old_controller.move_left,
                                       &mut controller.move_left,
                                       1);
                process_xinput_button(right_fake,
                                       &old_controller.move_right,
                                       &mut controller.move_right,
                                       1);

                process_xinput_button(gamepad.wButtons,
                                       &old_controller.left_shoulder,
                                       &mut controller.left_shoulder,
                                       XINPUT_GAMEPAD_LEFT_SHOULDER);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.right_shoulder, 
                                       &mut controller.right_shoulder,
                                       XINPUT_GAMEPAD_RIGHT_SHOULDER);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.action_up, 
                                       &mut controller.action_up,
                                       XINPUT_GAMEPAD_Y);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.action_down, 
                                       &mut controller.action_down,
                                       XINPUT_GAMEPAD_A);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.action_left, 
                                       &mut controller.action_left,
                                       XINPUT_GAMEPAD_X);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.action_right, 
                                       &mut controller.action_right,
                                       XINPUT_GAMEPAD_B);

                process_xinput_button(gamepad.wButtons,
                                       &old_controller.start,
                                       &mut controller.start,
                                       XINPUT_GAMEPAD_START);
                process_xinput_button(gamepad.wButtons,
                                       &old_controller.back,
                                       &mut controller.back,
                                       XINPUT_GAMEPAD_BACK);
            },

            //Case the Controller is not connected
            ERROR_DEVICE_NOT_CONNECTED => {
                controller.is_connected = false
            },


            //Some arbitrary Error was found
            _ => (),
        }
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
                         old_state: &Button, 
                         new_state: &mut Button,
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
                            keyboard_controller: &mut ControllerInput) {
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
                        b'W' => process_keyboard_message(
                                        &mut keyboard_controller.move_up, is_down),
                        b'A' => process_keyboard_message(
                                        &mut keyboard_controller.move_left, is_down),
                        b'S' => process_keyboard_message(
                                        &mut keyboard_controller.move_down, is_down),
                        b'D' => process_keyboard_message(
                                        &mut keyboard_controller.move_right, is_down),
                        b'E' => process_keyboard_message(
                                        &mut keyboard_controller.right_shoulder, is_down),
                        b'Q' => process_keyboard_message(
                                        &mut keyboard_controller.left_shoulder, is_down),
                        b'P' => {
                            if is_down {
                                window.pause = !window.pause;
                            }
                        },
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

fn process_keyboard_message(button: &mut Button, is_down: bool) {
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
                        pause: false,
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
        byte_index: 0,
        //TODO: Handle all the cases when there was no DirectSound and unwrap fails!
        sound_buffer: dsound_init(window.handle, SOUND_BYTES_PER_SECOND,
                                  SOUND_BYTES_PER_SECOND / BYTES_PER_SAMPLE).unwrap(),
    };

    unsafe {
        clear_sound_output(&mut sound_output);
        ((*(*sound_output.sound_buffer).lpVtbl).Play)(sound_output.sound_buffer, 0, 0, DSBPLAY_LOOPING);
    }

//    loop {
//        let mut play_cursor: DWORD = 0;
//        let mut write_cursor: DWORD = 0;
//        if unsafe { ((*(*sound_output.sound_buffer).lpVtbl).GetCurrentPosition)
//                     (sound_output.sound_buffer, 
//                      &mut play_cursor,
//                      &mut write_cursor) == DS_OK } {
//            println!("PC:{} WC:{}", play_cursor, write_cursor);
//        }
//    }

    let context = unsafe { GetDC(window.handle) };
    if context.is_null() {
        panic!("DC for the Window not available!");
    }

    let mut sound_samples: &mut [i16] = unsafe { 
            //Allocation implicitly freed at the end of the execution
            let data = VirtualAlloc(ptr::null_mut(), sound_output.get_buffer_size() as SIZE_T,
                                     MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE);
            if data.is_null() {
                panic!("Couldn't allocate the resources for the Sound-Buffer!");
            }

            mem::transmute(
                Slice { data: data as *const i16,
                             len: sound_output.get_buffer_size() as uint / mem::size_of::<i16>()})
        };
   
    let base_address = if cfg!(ndebug) { 0 } else { util::tera_bytes(2) };
    let permanent_store_size = util::mega_bytes(64);
    let transient_store_size = util::giga_bytes(4);
    let memory = unsafe { VirtualAlloc(base_address as LPVOID, 
                                       (permanent_store_size + transient_store_size) as SIZE_T,
                                       MEM_RESERVE|MEM_COMMIT, PAGE_READWRITE) };

    if memory.is_null() { panic!("Memory for the Game could not be obtained!"); }

    let mut game_memory: GameMemory = 
        GameMemory {
            initialized: false,
            permanent: unsafe { 
                        mem::transmute( Slice { 
                                            data: memory as *const u8, 
                                            len: permanent_store_size
                                        } 
                                      ) 
                        },
            transient: unsafe { 
                        mem::transmute( Slice { 
                                            data: (memory as *const u8)
                                                   .offset(permanent_store_size as int), 
                                            len: transient_store_size
                                        }
                                      ) 
                        },
            platform_read_entire_file: debug::platform_read_entire_file,
            platform_write_entire_file: debug::platform_write_entire_file,
            platform_free_file_memory: debug::platform_free_file_memory,
        };

    let sleep_is_granular = TimePeriod::new();

    let target_seconds_per_frame = 1.0 / GAME_REFRESH_RATE as f32;

    let mut sound_is_valid = false;
    let mut last_time_marker_index: uint = 0;
    let mut last_time_markers: [debug::SoundTimeMarker, ..GAME_REFRESH_RATE/2] = 
                                [Default::default(), ..GAME_REFRESH_RATE/2];

    let mut new_input: &mut Input = &mut Default::default();
    let mut old_input: &mut Input = &mut Default::default();

    let mut audio_latency_bytes: uint;
    let mut audio_latency_seconds: f32;

    let mut counter_frequency: i64 = 0;
    unsafe { QueryPerformanceFrequency(&mut counter_frequency); }

    let mut last_cycles = intrinsics::__rdtsc();

    let mut last_counter: i64 = get_wall_clock();
    let mut flip_wall_clock: i64 = 0;

    let mut load_counter: u8 = 0;
    let mut game = load_game_functions();

    window.running = true;
    while window.running {

        load_counter += 1;
        if load_counter > 120 {
            unload_game_functions(&mut game);
            game = load_game_functions();
        }

        //Keep the old button state but zero out the halftransitoncount to not mess up
        new_input.controllers[0] = old_input.controllers[0];
        new_input.controllers[0].is_connected = true;
        new_input.controllers[0].zero_half_transitions();
        process_pending_messages(&mut window, &mut new_input.controllers[0]);

        if !window.pause {
        process_xinput(XInputGetState,
                       new_input.controllers.slice_from_mut(1), 
                       old_input.controllers.slice_from(1));

        let mut video_buf = VideoBuffer {
            memory: unsafe { mem::transmute(
                             Slice { data: window.backbuffer.memory as *const u32, 
                                 len: (window.backbuffer.size/BYTES_PER_PIXEL) as uint})
                           },
            width: window.backbuffer.width as uint,
            height: window.backbuffer.height as uint,
            pitch: (window.backbuffer.pitch/BYTES_PER_PIXEL) as uint,
        };

        (game.update_and_render)(&mut game_memory,
                          new_input,
                          &mut video_buf);

        let from_begin_to_audio = get_seconds_elapsed(flip_wall_clock, get_wall_clock(),
                                                       counter_frequency);
        let mut play_cursor: DWORD = 0;
        let mut write_cursor: DWORD = 0;

        if unsafe { ((*(*sound_output.sound_buffer).lpVtbl).GetCurrentPosition)
                     (sound_output.sound_buffer, 
                      &mut play_cursor,
                      &mut write_cursor) == DS_OK } {

            if !sound_is_valid {
                sound_output.byte_index = write_cursor; 
                sound_is_valid = true;
            }

            let sound_buffer_size = sound_output.get_buffer_size(); 
            let byte_to_lock = sound_output.byte_index % sound_buffer_size; 

            let safe_write_cursor = 
                if write_cursor < play_cursor {
                    write_cursor + SOUND_SAFETY_BYTES + sound_buffer_size
                } else {
                    write_cursor + SOUND_SAFETY_BYTES
                };
            debug_assert!(safe_write_cursor >= play_cursor);

            let expected_sound_bytes_per_frame = SOUND_BYTES_PER_SECOND / GAME_REFRESH_RATE as u32;
            let seconds_to_flip = target_seconds_per_frame - from_begin_to_audio;
            let expected_bytes_to_flip = (expected_sound_bytes_per_frame as f32 * 
                                        (seconds_to_flip / target_seconds_per_frame)) as DWORD;
            let expected_frame_boundary_byte = play_cursor + expected_bytes_to_flip;
            let audio_card_not_latent = safe_write_cursor < expected_frame_boundary_byte;

            //The division with BYTES_PER_SAMPLE as well as the multiplication afterwards
            //are for the fact that we need a multiple of 4 to be sample aligned
            //because we calculated the expected_bytes_to_flip exactly it may
            //not be 4byte aligned so we do the alignment here!
            let target_cursor = 
                if audio_card_not_latent {
                    (((expected_frame_boundary_byte + expected_sound_bytes_per_frame) 
                        / BYTES_PER_SAMPLE) * BYTES_PER_SAMPLE)
                    % sound_buffer_size
                } else {
                    (((write_cursor + expected_sound_bytes_per_frame + SOUND_SAFETY_BYTES) 
                        / BYTES_PER_SAMPLE) * BYTES_PER_SAMPLE)
                     % sound_buffer_size
                };

            let bytes_to_write = 
                if byte_to_lock > target_cursor {
                    sound_buffer_size - byte_to_lock + target_cursor
                } else {
                    target_cursor - byte_to_lock
                };

            let mut sound_buf = SoundBuffer {
                samples: sound_samples.slice_to_mut(bytes_to_write as uint/
                                                    mem::size_of::<i16>()),
                samples_per_second: SOUND_BYTES_PER_SECOND / BYTES_PER_SAMPLE,
            };

            (game.get_sound_samples)(&mut game_memory, &mut sound_buf);

            fill_sound_output(&mut sound_output, byte_to_lock, bytes_to_write, &sound_buf);

            if cfg!(not(ndebug)) {
                last_time_markers[last_time_marker_index].output_write_cursor = write_cursor;
                last_time_markers[last_time_marker_index].expected_frame_byte = expected_frame_boundary_byte;
                last_time_markers[last_time_marker_index].output_play_cursor = play_cursor;
                last_time_markers[last_time_marker_index].output_location = byte_to_lock;
                last_time_markers[last_time_marker_index].output_byte_count = bytes_to_write;
                let unwraped_write_cursor = 
                    if write_cursor < play_cursor {
                        write_cursor + sound_output.get_buffer_size()
                    } else {
                        write_cursor
                    };

                audio_latency_bytes = (unwraped_write_cursor - play_cursor) as uint;
                audio_latency_seconds = audio_latency_bytes as f32 /
                                            SOUND_BYTES_PER_SECOND as f32;

                println!("BTL:{} BTW:{} - PC:{} WC:{} Delta:{} ({}s)",
                         byte_to_lock,
                         bytes_to_write, play_cursor, write_cursor,
                         audio_latency_bytes, audio_latency_seconds);
            }
        } else {
            sound_is_valid = false;
        }

        let mut seconds_elapsed_for_work = get_seconds_elapsed(last_counter,
                                                               get_wall_clock(),
                                                               counter_frequency);
        if seconds_elapsed_for_work < target_seconds_per_frame {
            while seconds_elapsed_for_work < target_seconds_per_frame {
                if sleep_is_granular.was_set() {
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

        let end_counter = get_wall_clock();

        if cfg!(not(ndebug)) {
            let current_marker = 
                if last_time_marker_index == 0 {
                    last_time_markers.len() - 1
                } else {
                    last_time_marker_index - 1
                };
            debug::sound_sync_display(&mut window.backbuffer, &last_time_markers,
                                      current_marker, &sound_output);
        }

        let (width, height) = get_client_dimensions(window.handle).unwrap();
        blit_buffer_to_window(context, &window.backbuffer, width, height);

        flip_wall_clock = get_wall_clock();

        if cfg!(not(ndebug)) {
            let mut play_cursor: DWORD = 0;
            let mut write_cursor: DWORD = 0;
            if unsafe { ((*(*sound_output.sound_buffer).lpVtbl).GetCurrentPosition)
                         (sound_output.sound_buffer, 
                          &mut play_cursor,
                          &mut write_cursor) == DS_OK } {
                last_time_markers[last_time_marker_index].flip_play_cursor = play_cursor;
                last_time_markers[last_time_marker_index].flip_write_cursor = write_cursor;

                last_time_marker_index += 1;
                if last_time_marker_index >= last_time_markers.len()  {
                    last_time_marker_index = 0;
                }
            }
        }

        let ms_per_frame = 1000.0 * get_seconds_elapsed(last_counter,
                                                        end_counter,
                                                        counter_frequency);
        let fps: f32 = 1000.0 / ms_per_frame;
        let display_cicles = intrinsics::__rdtsc();
        let mc_per_second = (display_cicles - last_cycles) as f32/ (1000.0 * 1000.0);

        println!("{:.2}ms/f, {:.2}f/s, {:.2}mc/s", ms_per_frame, fps, mc_per_second);


        mem::swap(new_input, old_input);

        last_counter = end_counter;
        last_cycles = intrinsics::__rdtsc();
    }
    }
}
