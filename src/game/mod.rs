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
        state.player_x = 150.0;
        state.player_y = 150.0;

        game_memory.initialized = true;
    }
    
    const TILEMAP_COUNT_Y: usize = 9;
    const TILEMAP_COUNT_X: usize = 17;

    let tiles0: [u32; TILEMAP_COUNT_X * TILEMAP_COUNT_Y] = 
        [
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1, 1 ,
             1, 1, 0, 0,   0, 1, 0, 0,   0, 0, 0, 0,   0, 1, 0, 0, 1 ,
             1, 1, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 1, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 0, 0, 0, 0 ,
             1, 1, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 1, 0, 0, 1 ,
             1, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   1, 0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 0, 0, 0,   0, 0, 0, 0,   0, 1, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 
        ];

    let tiles1: [u32; TILEMAP_COUNT_X * TILEMAP_COUNT_Y] = 
        [
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 0 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1, 1 
        ];

    let tiles2: [u32; TILEMAP_COUNT_X * TILEMAP_COUNT_Y] = 
        [
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 
        ];

    let tiles3: [u32; TILEMAP_COUNT_X * TILEMAP_COUNT_Y] = 
        [
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1, 1 
        ];

    let tilemaps = [TileMap {
                        count_x: TILEMAP_COUNT_X,
                        count_y: TILEMAP_COUNT_Y,
                        upper_left_x: -30.0,
                        upper_left_y: 0.0,
                        tile_width: 60.0,
                        tile_height: 60.0,
                        tiles: &tiles0[..],
                    },TileMap {
                        count_x: TILEMAP_COUNT_X,
                        count_y: TILEMAP_COUNT_Y,
                        upper_left_x: -30.0,
                        upper_left_y: 0.0,
                        tile_width: 60.0,
                        tile_height: 60.0,
                        tiles: &tiles1[..],
                    },TileMap {
                        count_x: TILEMAP_COUNT_X,
                        count_y: TILEMAP_COUNT_Y,
                        upper_left_x: -30.0,
                        upper_left_y: 0.0,
                        tile_width: 60.0,
                        tile_height: 60.0,
                        tiles: &tiles2[..],
                    },TileMap {
                        count_x: TILEMAP_COUNT_X,
                        count_y: TILEMAP_COUNT_Y,
                        upper_left_x: -30.0,
                        upper_left_y: 0.0,
                        tile_width: 60.0,
                        tile_height: 60.0,
                        tiles: &tiles3[..],
                    }];


    let current_map = &tilemaps[0];

    let world = World {
                    count_x: 2,
                    count_y: 2,
                    tilemaps: &tilemaps[..]
                };

    let player_width = 0.75 * current_map.tile_width;
    let player_height = current_map.tile_height;

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

            let new_player_x = state.player_x + dplayer_x * input.delta_time;
            let new_player_y = state.player_y + dplayer_y * input.delta_time;

            if is_tilemap_point_empty(new_player_x - 0.5*player_width, new_player_y, &current_map) &&
               is_tilemap_point_empty(new_player_x + 0.5*player_width, new_player_y, &current_map) &&
               is_tilemap_point_empty(new_player_x, new_player_y, &current_map) {

                state.player_x = new_player_x;
                state.player_y = new_player_y;
            }
        }
    }

    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    graphics::draw_rect(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
                        1.0, 0.0, 1.0);

    for row in 0..current_map.count_y {
        for column in 0..current_map.count_x {
            let elem = current_map.get_tile_value(column as i32, row as i32);
            let color = 
                if elem == 0 {
                    0.5
                } else {
                    1.0
                };
            let min_x = current_map.upper_left_x + column as f32 * current_map.tile_width;
            let min_y = current_map.upper_left_y + row as f32 * current_map.tile_height;
            let max_x = min_x + current_map.tile_width;
            let max_y = min_y + current_map.tile_height;
            graphics::draw_rect(video_buffer, min_x, min_y, max_x, max_y,
                                color, color, color);
        }
    }

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


struct World<'a> {
    count_x: usize,
    count_y: usize,
    
    tilemaps: &'a[TileMap<'a>],
}


impl<'a> World<'a> {
    fn get_tilemap(&self, tilemap_x: i32, tilemap_y: i32) -> &'a TileMap<'a> {
        &self.tilemaps[(tilemap_y * self.count_x as i32 + tilemap_x) as usize]
    }
}


struct TileMap<'a> {
    count_x: usize,
    count_y: usize,

    upper_left_x: f32,
    upper_left_y: f32,
    tile_width: f32,
    tile_height: f32,

    tiles: &'a[u32]
}

impl<'a> TileMap<'a> {
    fn get_tile_value(&self, tile_x: i32, tile_y: i32) -> u32 {
        self.tiles[(tile_y * self.count_x as i32 + tile_x) as usize]
    }
}


fn is_world_point_empty(world: &World, tilemap_x: i32, tilemap_y: i32,
                        test_x: f32, test_y: f32, tilemap: &TileMap) -> bool {

    let tilemap = world.get_tilemap(tilemap_x, tilemap_y);

    is_tilemap_point_empty(test_x, test_y, tilemap)

}

fn is_tilemap_point_empty(test_x: f32, test_y: f32, tilemap: &TileMap) -> bool {

    let mut is_empty = false;

    let tile_x = ((test_x - tilemap.upper_left_x) / tilemap.tile_width) as i32;
    let tile_y = ((test_y - tilemap.upper_left_y) / tilemap.tile_height) as i32;

    if tile_x >= 0 && tile_x < tilemap.count_x as i32 &&
    tile_y >= 0 && tile_y < tilemap.count_y as i32 {

        let tile_elem = tilemap.get_tile_value(tile_x, tile_y);
        is_empty = tile_elem == 0;
    }

    is_empty
}



