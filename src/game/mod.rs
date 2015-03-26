use std::mem;

#[allow(unused_imports)]
use common::{GameMemory, SoundBuffer, VideoBuffer, Input, ReadFileResult};
use common::{ThreadContext};

mod graphics;

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

    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    if !game_memory.initialized {
        state.player_x = 100.0;
        state.player_y = 100.0;

        game_memory.initialized = true;
    }
    
    for controller in input.controllers.iter() {

        //Analog movement
        if controller.is_analog() {

        //Digital movement
        } else {
            let mut dplayer_x = 0.0; //In Pixels per second
            let mut dplayer_y = 0.0;
            
            if controller.move_up.ended_down {
                dplayer_y = -1.0;
            }
            if controller.move_down.ended_down {
                dplayer_y = 1.0;
            }
            if controller.move_left.ended_down {
                dplayer_x = -1.0;
            }
            if controller.move_right.ended_down {
                dplayer_x = 1.0;
            }

            dplayer_x *= 64.0;
            dplayer_y *= 64.0;

            state.player_x += dplayer_x * input.delta_time;
            state.player_y += dplayer_y * input.delta_time;
        }
    }

    let tilemap: [[u32; 17]; 9] = 
        [
            [ 1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 ],
            [ 1, 1, 0, 0,   0, 1, 0, 0,   0, 0, 0, 0,   0, 1, 0, 0, 1 ],
            [ 1, 1, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 1, 0, 1 ],
            [ 1, 0, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 0, 0, 1 ],
            [ 0, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 0, 0, 0, 0 ],
            [ 1, 1, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 1, 0, 0, 1 ],
            [ 1, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   1, 0, 0, 0, 1 ],
            [ 1, 1, 1, 1,   1, 0, 0, 0,   0, 0, 0, 0,   0, 1, 0, 0, 1 ],
            [ 1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 ]
        ];

    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    graphics::draw_rect(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
                        1.0, 0.0, 1.0);

    let upper_left_x = -30.0;
    let upper_left_y = 0.0;
    let tile_width = 60.0;
    let tile_height = 60.0;
    
    for row in 0..tilemap.len() {
        for column in 0..tilemap[row].len() {
            let elem = tilemap[row][column];
            let color = 
                if elem == 0 {
                    0.5
                } else {
                    1.0
                };
            let min_x = upper_left_x + column as f32 * tile_width;
            let min_y = upper_left_y + row as f32 * tile_height;
            let max_x = min_x + tile_width;
            let max_y = min_y + tile_height;
            graphics::draw_rect(video_buffer, min_x, min_y, max_x, max_y,
                                color, color, color);
        }
    }

    let player_width = 0.75 * tile_width;
    let player_height = tile_height;
    let player_top = state.player_y - player_height;
    let player_left = state.player_x - 0.5 * player_width;
    graphics::draw_rect(video_buffer, 
                        player_left, player_top,
                        player_left + player_width, player_top + player_height,
                        1.0, 1.0, 0.0);

}

// ======== End of the public interface =========


struct GameState {
    player_x: f32,
    player_y: f32,
} 

