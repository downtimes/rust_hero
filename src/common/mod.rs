use libc::{c_void};
use std::default::Default;

type PlatformReadEntireFileT = fn(&ThreadContext, &str) -> Result<ReadFileResult, ()>;
type PlatformFreeFileMemoryT = fn(&ThreadContext, *mut c_void);
type PlatformWriteEntireFileT = fn(&ThreadContext, &str, u32, *mut c_void) -> bool;

pub type GetSoundSamplesT = extern fn(&ThreadContext, &mut GameMemory, &mut SoundBuffer);
pub type UpdateAndRenderT = extern fn(&ThreadContext, &mut GameMemory, &Input, &mut VideoBuffer); 

pub struct ReadFileResult {
    pub size: u32,
    pub contents: *mut c_void,
}

pub struct VideoBuffer<'a> {
    //Buffer memory is assumed to be BB GG RR xx
    pub memory: &'a mut [u32],
    pub width: uint,
    pub height: uint,
    pub pitch: uint,
}

pub struct SoundBuffer<'a> {
    //Samples memory is assumed to be two channels interleaved
    pub samples: &'a mut [i16],
    pub samples_per_second: u32,
}

#[deriving(Default)]
pub struct Button {
    pub ended_down: bool,
    pub half_transitions: u8,
}

#[deriving(Default)]
pub struct ControllerInput {
    pub is_connected: bool,

    pub average_x: Option<f32>,
    pub average_y: Option<f32>,
    
    pub move_up: Button,
    pub move_down: Button,
    pub move_left: Button,
    pub move_right: Button,

    pub action_up: Button,
    pub action_down: Button,
    pub action_left: Button,
    pub action_right: Button,

    pub left_shoulder: Button,
    pub right_shoulder: Button,
    
    pub start: Button,
    pub back: Button,
}

impl ControllerInput {
    //need to allow dead code because the function is only used in exe
    #[allow(dead_code)]
    pub fn zero_half_transitions(&mut self) {
        self.move_up.half_transitions = 0;
        self.move_down.half_transitions = 0;
        self.move_left.half_transitions = 0;
        self.move_right.half_transitions = 0;
        self.action_up.half_transitions = 0;
        self.action_down.half_transitions = 0;
        self.action_left.half_transitions = 0;
        self.action_right.half_transitions = 0;
        self.left_shoulder.half_transitions = 0;
        self.right_shoulder.half_transitions = 0;
        self.start.half_transitions = 0;
        self.back.half_transitions = 0;
    }

    //need to allow dead code because the function is only used in the dll
    #[allow(dead_code)]
    pub fn is_analog(&self) -> bool {
        self.average_x.is_some() && self.average_y.is_some()
    }
}

pub struct ThreadContext;

pub struct Input {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_z: i32,

    pub mouse_l: Button,
    pub mouse_r: Button,
    pub mouse_m: Button,
    pub mouse_x1: Button,
    pub mouse_x2: Button,

    //TODO: see if it fits rustaceans better if we have an Option of 
    //ControllerInputs here?
    //The 0 Controller is the keyboard all the others are possible joysticks
    pub controllers: [ControllerInput, ..5],
}

impl Default for Input {
    fn default() -> Input {
        Input {
            mouse_x: 0,
            mouse_y: 0,
            mouse_z: 0,

            mouse_l: Default::default(),
            mouse_r: Default::default(),
            mouse_m: Default::default(),
            mouse_x1: Default::default(),
            mouse_x2: Default::default(),
            controllers: [Default::default(), ..5],
        }
    }
}

pub struct GameMemory<'a> {
    pub initialized: bool,
    pub permanent: &'a mut[u8], //REQUIRED to be zeroed
    pub transient: &'a mut[u8], //REQUIRED to be zeroed
    pub platform_read_entire_file: PlatformReadEntireFileT, 
    pub platform_write_entire_file: PlatformWriteEntireFileT,
    pub platform_free_file_memory: PlatformFreeFileMemoryT,
}

