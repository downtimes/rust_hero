use std::f32;
use std::num::FloatMath;
use std::default::Default;
use std::mem;

#[cfg(target_os="windows")]
use win32::debug;
#[cfg(target_os="linux")]
use linux::debug;

// ============= The public interface ===============
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

    pub fn is_analog(&self) -> bool {
        self.average_x.is_some() && self.average_y.is_some()
    }
}

pub struct Input {
    //TODO: see if it fits rustaceans better if we have an Option of 
    //ControllerInputs here?
    //The 0 Controller is the keyboard all the others are possible joysticks
    pub controllers: [ControllerInput, ..5],
}

impl Default for Input {
    fn default() -> Input {
        Input {
            controllers: [Default::default(), ..5],
        }
    }
}


pub struct GameMemory<'a> {
    pub initialized: bool,
    pub permanent: &'a mut[u8], //REQUIRED to be zeroed
    pub transient: &'a mut[u8], //REQUIRED to be zeroed
}

//Has to be very low latency!
pub fn get_sound_samples(game_memory: &mut GameMemory,
                              sound_buffer: &mut SoundBuffer) {
                                  
    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };
    generate_sound(sound_buffer, state.frequency, &mut state.tsine);
}

pub fn update_and_render(game_memory: &mut GameMemory,
                              input: &Input,
                              vidoe_buffer: &mut VideoBuffer) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    if !game_memory.initialized {
        let file = debug::platform_read_entire_file("test.txt");
        match file {
            Ok(debug::ReadFileResult{ size, contents }) => {
                debug::platform_write_entire_file("tester.txt", size, contents);
                debug::platform_free_file_memory(contents);
            },
            Err(_) => println!("Error as expected"),
        }
        game_memory.initialized = true;
    }
    
    for controller in input.controllers.iter() {

        if controller.is_analog() {
            state.blue_offset += (4.0f32 * controller.average_x.unwrap()) as i32;

            state.frequency = (512f32 + 128.0f32 * controller.average_y.unwrap()) as u32;
        } else {
            if controller.move_left.ended_down {
                state.blue_offset -= 1;
            } else if controller.move_right.ended_down {
                state.blue_offset += 1;
            }
        }

        if controller.action_down.ended_down {
            state.green_offset += 1;
        }
    }

    render_weird_gradient(vidoe_buffer, state.green_offset, state.blue_offset);
}

// ======== End of the public interface =========


struct GameState {
    frequency: u32,
    green_offset: i32,
    blue_offset: i32,
    tsine: f32,
}


fn generate_sound(buffer: &mut SoundBuffer, tone_frequency: u32, tsine: &mut f32) {
    let volume: f32 = 3000.0;
    let wave_period = buffer.samples_per_second / tone_frequency;

    debug_assert!(buffer.samples.len() % 2 == 0);
    for sample in buffer.samples.chunks_mut(2) {
        let sine_value: f32 = tsine.sin();
        let value = (sine_value * volume as f32) as i16;

        *tsine += f32::consts::PI_2 / (wave_period as f32); 
        if *tsine > f32::consts::PI_2 {
            *tsine -= f32::consts::PI_2;
        }

        for channel in sample.iter_mut() {
            *channel = value;
        }
    }
}

fn render_weird_gradient(buffer: &mut VideoBuffer, green_offset: i32, blue_offset: i32) {

    for (y, row) in buffer.memory.chunks_mut(buffer.pitch).take(buffer.width).enumerate() {
        let green_color = (y as i32 + green_offset) as u8;

        for (x, pixel) in row.iter_mut().enumerate() {
            let blue_color = (x as i32 + blue_offset) as u8;
            *pixel = (green_color as u32) << 8 | blue_color as u32;
        }
    }
}
