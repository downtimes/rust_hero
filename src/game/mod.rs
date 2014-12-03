use std::f32;
use std::num::FloatMath;

//The public interface of the game
pub struct VideoBuffer<'a> {
    //Buffer memory is assumed to be BB GG RR xx
    pub memory: &'a mut [u32],
    pub width: uint,
    pub height: uint,
    pub pitch: uint,
}

pub struct SoundBuffer<'a> {
    //Buffer memory is assumed to be two channels interleaved
    pub samples: &'a mut [i16],
    pub samples_per_second: u32,
}

pub fn game_update_and_render(vidoe_buffer: &mut VideoBuffer, green_offset: i32, 
                              blue_offset: i32, sound_buffer: &mut SoundBuffer,
                              frequency: u32) {
    generate_sound(sound_buffer, frequency);
    render_weird_gradient(vidoe_buffer, green_offset, blue_offset);
}
//End of the public interface

static mut T_SINE: f32 = 0.0;

fn generate_sound(buffer: &mut SoundBuffer, tone_frequency: u32) {
    let volume: f32 = 3000.0;
    let wave_period = buffer.samples_per_second / tone_frequency;

    assert!(buffer.samples.len() % 2 == 0);
    for sample in buffer.samples.chunks_mut(2) {
        let sine_value: f32 = unsafe { T_SINE.sin() };
        let value = (sine_value * volume as f32) as i16;
        //TODO: this value gets too big for the sine function so were left
        //with only a few steps after some seconds. Needs a fix.
        unsafe { T_SINE += f32::consts::PI_2 / (wave_period as f32); }

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
