use std::f32;
use std::num::FloatMath;
use std::default::Default;
use std::mem;

#[cfg(target_os="windows")]
use win32::*;
#[cfg(target_os="linux")]
use linux::*;

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
    pub is_analog: bool,

    pub end_x: f32,
    pub end_y: f32,

    pub min_x: f32,
    pub min_y: f32,

    pub max_x: f32,
    pub max_y: f32,

    pub start_x: f32,
    pub start_y: f32,

    pub up: Button,
    pub down: Button,
    pub left: Button,
    pub right: Button,
    pub left_shoulder: Button,
    pub right_shoulder: Button,
}

pub struct Input {
    pub controllers: [ControllerInput, ..4],
}

impl Default for Input {
    fn default() -> Input {
        Input {
            controllers: [Default::default(), .. 4],
        }
    }
}


pub struct GameMemory<'a> {
    pub initialized: bool,
    pub permanent: &'a mut[u8], //REQUIRED to be zeroed
    pub transient: &'a mut[u8], //REQUIRED to be zeroed
}


pub fn game_update_and_render(game_memory: &mut GameMemory,
                              input: &Input,
                              vidoe_buffer: &mut VideoBuffer,
                              sound_buffer: &mut SoundBuffer,) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let mut state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    if !game_memory.initialized {
        let file = debug::platform_read_entire_file("test.txt");
        match file {
            Some(debug::ReadFileResult{ size, contents }) => {
                debug::platform_write_entire_file("tester.txt", size, contents);
                debug::platform_free_file_memory(contents);
            },
            None => println!("Error as expected"),
        }
        game_memory.initialized = true;
    }
    
    let p1_input = &input.controllers[0];
    let frequency = 
        if p1_input.is_analog {
            state.blue_offset += (4.0f32 * p1_input.end_x) as i32;

            (256f32 + 128.0f32 * p1_input.end_y) as u32
        } else { 
            256
        };

    if p1_input.down.ended_down {
        state.green_offset += 1;
    }

    generate_sound(sound_buffer, frequency, &mut state.tsine);
    render_weird_gradient(vidoe_buffer, state.green_offset, state.blue_offset);
}

// ======== End of the public interface =========


struct GameState {
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
        //TODO: this value gets too big for the sine function so we're left
        //with only a few discrete sound steps after some seconds. Needs a fix.
        *tsine += f32::consts::PI_2 / (wave_period as f32); 

        for channel in sample.iter_mut() {
            *channel = value;
        }
    }
}

fn render_weird_gradient(buffer: &mut VideoBuffer, green_offset: i32, blue_offset: i32) {

    for (y, row) in buffer.memory.chunks_mut(buffer.pitch).enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            //if we have padding we don't want to write farther out than 
            //the width of our image
            if x >= buffer.width {
                break;
            }
            let green_color = (y as i32 + green_offset) as u8;
            let blue_color = (x as i32 + blue_offset) as u8;
            *pixel = (green_color as u32) << 8 | blue_color as u32;
        }
    }
}
