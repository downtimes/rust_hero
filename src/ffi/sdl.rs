#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub use libc::{c_int, c_char, c_void};
use std::default::Default;

pub type SDL_Keycode = i32;
pub type SDL_AudioFormat = u16;
pub type SDLAudioCallbackT = extern "C" fn(*mut c_void, *mut u8, c_int);

pub const AUDIO_S16LSB: u16 = 0x8010;

pub const SDL_INIT_TIMER: u32 = 0x00000001;
pub const SDL_INIT_AUDIO: u32 = 0x00000010;
pub const SDL_INIT_VIDEO: u32 = 0x00000020;
pub const SDL_INIT_GAMECONTROLLER: u32 = 0x00002000;
pub const SDL_INIT_EVENTS: u32 = 0x000004000;

pub const SDL_WINDOWPOS_UNDEFINED: c_int = 0x1FFF0000;

pub const SDL_WINDOWEVENT_EXPOSED: u8 = 3;
pub const SDL_WINDOWEVENT_RESIZED: u8 = 5;
pub const SDL_WINDOWEVENT_SIZE_CHANGED: u8 = 6;

pub const SDL_PIXELFORMAT_ARGB8888: u32 = 0x16462004;
pub const SDL_PIXELFORMAT_BGRA8888: u32 = 0x16862004;

pub const SDL_TEXTUREACCESS_STREAMING: c_int = 1;

pub const SDL_WINDOW_RESIZABLE: u32 = 0x00000020;

pub const SDL_QUIT: u32 = 0x100;
pub const SDL_WINDOWEVENT: u32 = 0x200;
pub const SDL_KEYDOWN: u32 = 0x300;
pub const SDL_KEYUP: u32 = 0x301;

pub const SDL_PRESSED: u8 = 1;


pub const SDLK_w: i32 = 'w' as i32;
pub const SDLK_a: i32 = 'a' as i32;
pub const SDLK_s: i32 = 's' as i32;
pub const SDLK_d: i32 = 'd' as i32;
pub const SDLK_e: i32 = 'e' as i32;
pub const SDLK_p: i32 = 'p' as i32;
pub const SDLK_q: i32 = 'q' as i32;
pub const SDLK_l: i32 = 'l' as i32;
pub const SDLK_ESCAPE: i32 = 27;
pub const SDLK_SPACE: i32 = ' ' as i32;
pub const SDLK_F4: i32 = (1 << 30) | 61;
pub const SDLK_RIGHT: i32 = (1 << 30) | 79;
pub const SDLK_LEFT: i32 = (1 << 30) | 80;
pub const SDLK_DOWN: i32 = (1 << 30) | 81;
pub const SDLK_UP: i32 = (1 << 30) | 82;

#[repr(C)]
pub struct SDL_Window;

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum SDL_bool {
	SDL_FALSE,
	SDL_TRUE,
}

#[repr(C)]
pub struct SDL_Renderer;

#[repr(C)]
pub struct SDL_GameController;

#[repr(C)]
pub struct SDL_Texture;

#[repr(C)]
#[derive(Copy)]
pub struct SDL_WindowEvent {
    pub _type: u32,
    pub timestamp: u32,
    pub windowID: u32,
    pub event: u8,
    pub padding1: u8,
    pub padding2: u8,
    pub padding3: u8,
    pub data1: i32,
    pub data2: i32,
}

#[repr(C)]
pub struct SDL_AudioSpec {
	pub freq: c_int,
	pub format: SDL_AudioFormat,
	pub channels: u8,
	pub silence: u8,
	pub samples: u16,
	pub padding: u16,
	pub size: u32,
	pub callback: SDLAudioCallbackT,
	pub userdata: *mut c_void,
}

#[repr(C)]
pub enum SDL_GameControllerAxis {
	SDL_CONTROLLER_AXIS_INVALID = -1,
	SDL_CONTROLLER_AXIS_LEFTX,
	SDL_CONTROLLER_AXIS_LEFTY,
	SDL_CONTROLLER_AXIS_RIGHTX,
	SDL_CONTROLLER_AXIS_RIGHTY,
	SDL_CONTROLLER_AXIS_TRIGGERLEFT,
	SDL_CONTROLLER_AXIS_TRIGGERRIGHT,
	SDL_CONTROLLER_AXIS_MAX,
}

#[repr(C)]
pub enum SDL_GameControllerButton {
	SDL_CONTROLLER_BUTTON_INVALID = -1,
	SDL_CONTROLLER_BUTTON_A,
	SDL_CONTROLLER_BUTTON_B,
	SDL_CONTROLLER_BUTTON_X,
	SDL_CONTROLLER_BUTTON_Y,
	SDL_CONTROLLER_BUTTON_BACK,
	SDL_CONTROLLER_BUTTON_GUIDE,
	SDL_CONTROLLER_BUTTON_START,
	SDL_CONTROLLER_BUTTON_LEFTSTICK,
	SDL_CONTROLLER_BUTTON_RIGHTSTICK,
	SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
	SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
	SDL_CONTROLLER_BUTTON_DPAD_UP,
	SDL_CONTROLLER_BUTTON_DPAD_DOWN,
	SDL_CONTROLLER_BUTTON_DPAD_LEFT,
	SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
	SDL_CONTROLLER_BUTTON_MAX,
}

#[repr(C)]
#[derive(Copy)]
pub enum SDL_Scancode {
	SDL_SCANCODE_UNKNOWN = 0,
	SDL_SCANCODE_MAX = 512,
}


#[repr(C)]
#[derive(Copy)]
pub struct SDL_Keysym {
	pub scancode: SDL_Scancode,
	pub sym: SDL_Keycode,
	pub _mod: u16,
	pub unused: u32,
}

#[repr(C)]
#[derive(Copy)]
pub struct SDL_KeyboardEvent {
	pub _type: u32,
	pub timestamp: u32,
	pub windowID: u32,
	pub state: u8,
	pub repeat: u8,
	pub padding2: u8,
	pub padding3: u8,
	pub keysym: SDL_Keysym,
}

#[repr(C)]
pub struct SDL_Quit {
    pub _type: u32,
    pub timestamp: u32,
}

#[repr(C)]
pub struct SDL_Rect {
	pub x: c_int,
	pub y: c_int,
	pub w: c_int,
	pub h: c_int,
}

#[repr(C)]
pub struct SDL_Event {
    pub data: [u8; 56],
}

impl Default for SDL_Event {
    fn default() -> SDL_Event {
        SDL_Event {
            data: [0; 56],
        }
    }
}

impl SDL_Event {
    pub fn _type(&self) -> u32 {
        unsafe { *(self.data.as_ptr() as *const _) }
    }

    pub fn window_event(&self) -> SDL_WindowEvent {
        unsafe { *(self.data.as_ptr() as *const _) }
    }

	pub fn keyboard_event(&self) -> SDL_KeyboardEvent {
        unsafe { *(self.data.as_ptr() as *const _) }
	}
}

extern "C" {
    pub fn SDL_Init(flags: u32) -> c_int;
    pub fn SDL_Quit();
    pub fn SDL_CreateWindow(title: *const c_char, x: c_int,
                            y: c_int, w: c_int, h: c_int,
                            flags: u32) -> *mut SDL_Window;
    pub fn SDL_WaitEvent(event: *mut SDL_Event) -> c_int;
	pub fn SDL_PollEvent(event: *mut SDL_Event) -> c_int;
    pub fn SDL_CreateRenderer(window: *mut SDL_Window,
                              index: c_int, flags: u32) -> *mut SDL_Renderer;
    pub fn SDL_SetRenderDrawColor(renderer: *mut SDL_Renderer,
                                 r: u8, g: u8, b: u8, a: u8)
                                    -> c_int;
    pub fn SDL_GetWindowFromID(id: u32) -> *mut SDL_Window;
    pub fn SDL_GetRenderer(window: *mut SDL_Window)
                                -> *mut SDL_Renderer;
    pub fn SDL_RenderClear(renderer: *mut SDL_Renderer) -> c_int;
    pub fn SDL_RenderPresent(renderer: *mut SDL_Renderer);
    pub fn SDL_CreateTexture(renderer: *mut SDL_Renderer,
                             format: u32, access: c_int,
                             w: c_int, h: c_int) -> *mut SDL_Texture;
    pub fn SDL_GetWindowSize(window: *mut SDL_Window,
                             w: *mut c_int,
                             h: *mut c_int);
    pub fn SDL_UpdateTexture(texture: *mut SDL_Texture, rect: *const SDL_Rect,
                             pixels: *const c_void, pitch: c_int) -> c_int;
    pub fn SDL_RenderCopy(renderer: *mut SDL_Renderer,
                          texture: *mut SDL_Texture,
                          srcrect: *const SDL_Rect,
                          dstrect: *const SDL_Rect) -> c_int;
    pub fn SDL_DestroyTexture(texture: *mut SDL_Texture);
    pub fn SDL_NumJoysticks() -> c_int;
    pub fn SDL_IsGameController(joystick_index: c_int) -> SDL_bool;
    pub fn SDL_GameControllerOpen(joystick_index: c_int)
        -> *mut SDL_GameController;
    pub fn SDL_GameControllerClose(game_controller: *mut SDL_GameController);
    pub fn SDL_GameControllerGetAttached(
        game_controller: *mut SDL_GameController) -> SDL_bool;
    pub fn SDL_GameControllerGetButton(
        game_controller: *mut SDL_GameController,
        button: SDL_GameControllerButton) -> u8;
    pub fn SDL_GameControllerGetAxis(
        game_controller: *mut SDL_GameController,
        axis: SDL_GameControllerAxis) -> i16;
    pub fn SDL_OpenAudio(desired: *mut SDL_AudioSpec,
                         obtained: *mut SDL_AudioSpec) -> c_int;
}
