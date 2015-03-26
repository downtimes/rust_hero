use std::mem;

#[allow(unused_imports)]
use common::{GameMemory, SoundBuffer, VideoBuffer, Input, ReadFileResult};
use common::{ThreadContext};

// ============= The public interface ===============
//Has to be very low latency!
#[no_mangle]
pub extern fn get_sound_samples(_context: &ThreadContext,
                                game_memory: &mut GameMemory,
                                _sound_buffer: &mut SoundBuffer) {
                                  
    let _state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };
}

#[no_mangle]
pub extern fn update_and_render(_context: &ThreadContext,
                                game_memory: &mut GameMemory,
                                input: &Input,
                                video_buffer: &mut VideoBuffer) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let _state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    if !game_memory.initialized {
        game_memory.initialized = true;
    }
    
    for controller in input.controllers.iter() {

        if controller.is_analog() {

        } else {
        }

    }

    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    draw_quad(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
              0x00FF00FF);
    draw_quad(video_buffer, -1.0, -10.0, -40.0, -40.0, 0x00FFFFFF);
}

// ======== End of the public interface =========


struct GameState; 




fn draw_quad(buffer: &mut VideoBuffer, mut min_x: f32, mut min_y: f32, mut max_x: f32,
             mut max_y: f32, color: u32) {
    if min_x < 0.0 {
        min_x = 0.0;
    }
    if min_y < 0.0 {
        min_y = 0.0;
    }
    if max_x > buffer.width as f32 {
        max_x = buffer.width as f32;
    }
    if max_y > buffer.height as f32{
        max_y = buffer.height as f32;
    }

    let width = if min_x < max_x {
        (max_x - min_x) as usize
    } else {
        0
    };
    let height = if min_y < max_y {
        (max_y - min_y) as usize
    } else {
        0
    };

    for row in buffer.memory.chunks_mut(buffer.pitch).skip(min_y as usize).take(height) {
        for pixel in row.iter_mut().skip(min_x as usize).take(width) {
           *pixel = color; 
        }
    }
}

