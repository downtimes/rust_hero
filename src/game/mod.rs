use std::f32;
use std::num::FloatMath;
use std::mem;

use common::{GameMemory, SoundBuffer, VideoBuffer, Input, ReadFileResult};
use common::{ThreadContext};

// ============= The public interface ===============
//Has to be very low latency!
#[no_mangle]
pub extern fn get_sound_samples(_context: &ThreadContext,
                                game_memory: &mut GameMemory,
                                sound_buffer: &mut SoundBuffer) {
                                  
    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };
    generate_sound(sound_buffer, state.frequency, &mut state.tsine);
}

#[no_mangle]
pub extern fn update_and_render(context: &ThreadContext,
                                game_memory: &mut GameMemory,
                                input: &Input,
                                video_buffer: &mut VideoBuffer) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    if !game_memory.initialized {
        let file = (game_memory.platform_read_entire_file)(context, "test.txt");
        match file {
            Ok(ReadFileResult{ size, contents }) => {
                (game_memory.platform_write_entire_file)(context, "tester.txt", size, contents);
                (game_memory.platform_free_file_memory)(context, contents);
            },
            Err(_) => panic!("File was not found!"),
        }

        state.player_x = 100;
        state.player_y = 100;
        game_memory.initialized = true;
    }
    
    for controller in input.controllers.iter() {

        if controller.is_analog() {
            let avg_x = controller.average_x.unwrap();
            let avg_y = controller.average_y.unwrap();
            state.blue_offset += (4.0f32 * avg_x) as i32;

            state.frequency = (512f32 + 128.0f32 * avg_y) as u32;

            state.player_x += (4.0f32 * avg_x) as i32;
            state.player_y -= (4.0f32 * avg_y) as i32;

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

    render_weird_gradient(video_buffer, state.green_offset, state.blue_offset);

    let white = 0xFFFFFFFFu32;
    draw_10_quad(video_buffer, state.player_x as uint, state.player_y as uint, white);
    draw_10_quad(video_buffer, input.mouse_x as uint, input.mouse_y as uint, white);
    if input.mouse_l.ended_down {
        draw_10_quad(video_buffer, 10, 10, white);
    }
    if input.mouse_r.ended_down {
        draw_10_quad(video_buffer, 30, 10, white);
    }
    if input.mouse_m.ended_down {
        draw_10_quad(video_buffer, 50, 10, white);
    }
    if input.mouse_x1.ended_down {
        draw_10_quad(video_buffer, 70, 10, white);
    }
    if input.mouse_x2.ended_down {
        draw_10_quad(video_buffer, 90, 10, white);
    }
}

// ======== End of the public interface =========


struct GameState {
    frequency: u32,
    green_offset: i32,
    blue_offset: i32,
    tsine: f32,

    player_x: i32,
    player_y: i32,
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

fn draw_10_quad(buffer: &mut VideoBuffer, x: uint, y: uint, color: u32) {
    for row in buffer.memory.chunks_mut(buffer.pitch).skip(y).take(10) {
        for pixel in row.iter_mut().skip(x).take(10) {
           *pixel = color; 
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
