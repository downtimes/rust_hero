use std::mem;
use std::default::Default;

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

    let tiles2: [u32; TILEMAP_COUNT_X * TILEMAP_COUNT_Y] = 
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
                        tiles: &tiles0[..],
                    },TileMap {
                        tiles: &tiles1[..],
                    },TileMap {
                        tiles: &tiles2[..],
                    },TileMap {
                        tiles: &tiles3[..],
                    }];

    let world = World {
                    tile_count_x: TILEMAP_COUNT_X,
                    tile_count_y: TILEMAP_COUNT_Y,
                    upper_left_x: -30.0,
                    upper_left_y: 0.0,
                    tile_width: 60.0,
                    tile_height: 60.0,
                    tilemap_count_x: 2,
                    tilemap_count_y: 2,
                    tilemaps: &tilemaps[..]
                };

    let current_map = world.get_tilemap(state.player_tilemap_x, state.player_tilemap_y);

    let player_width = 0.75 * world.tile_width;
    let player_height = world.tile_height;

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

            dplayer_x *= 128.0;
            dplayer_y *= 128.0;

            let new_player_x = state.player_x + dplayer_x * input.delta_time;
            let new_player_y = state.player_y + dplayer_y * input.delta_time;

            let player_middle_bottom = RawPosition {
                tilemap_x: state.player_tilemap_x,
                tilemap_y: state.player_tilemap_y,

                x: new_player_x,
                y: new_player_y,
            };

            let mut player_right_bottom = player_middle_bottom;
            player_right_bottom.x += 0.5*player_width;

            let mut player_left_bottom = player_middle_bottom;
            player_left_bottom.x -= 0.5*player_width;

            if is_world_point_empty(&world, &player_left_bottom) &&
               is_world_point_empty(&world, &player_right_bottom) &&
               is_world_point_empty(&world, &player_middle_bottom) {

                let new_position = player_middle_bottom.canonicalize(&world);

                state.player_tilemap_x = new_position.tilemap_x;
                state.player_tilemap_y = new_position.tilemap_y;

                state.player_x = world.upper_left_x + new_position.tile_x as f32 * world.tile_width + new_position.x;
                state.player_y = world.upper_left_y + new_position.tile_y as f32 * world.tile_height + new_position.y;
            }
        }
    }

    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    graphics::draw_rect(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
                        1.0, 0.0, 1.0);

    for row in 0..world.tile_count_y {
        for column in 0..world.tile_count_x {
            let elem = current_map.get_tile_value(column as i32, row as i32, &world);
            let color = 
                if elem == 0 {
                    0.5
                } else {
                    1.0
                };
            let min_x = world.upper_left_x + column as f32 * world.tile_width;
            let min_y = world.upper_left_y + row as f32 * world.tile_height;
            let max_x = min_x + world.tile_width;
            let max_y = min_y + world.tile_height;
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
    player_tilemap_x: i32,
    player_tilemap_y: i32,
    
    player_x: f32,
    player_y: f32,
} 


#[derive(Default)]
struct CanonicalPosition {
    tilemap_x: i32,
    tilemap_y: i32,

    tile_x: i32,
    tile_y: i32,

    //Tile relative
    x: f32,
    y: f32,
}

#[derive(Copy)]
struct RawPosition {
    tilemap_x: i32,
    tilemap_y: i32,

    //Tilemap relative
    x: f32,
    y: f32,
}

impl RawPosition {
    fn canonicalize(&self, world: &World) -> CanonicalPosition {
        let mut result: CanonicalPosition = Default::default();

        result.tilemap_x = self.tilemap_x;
        result.tilemap_y = self.tilemap_y;

        let rel_x = self.x - world.upper_left_x;
        let rel_y = self.y - world.upper_left_y;
        let mut tile_x = (rel_x / world.tile_width).floor() as i32;
        let mut tile_y = (rel_y / world.tile_height).floor() as i32;

        result.x = rel_x - tile_x as f32 * world.tile_width;
        result.y = rel_y - tile_y as f32 * world.tile_height;

        if tile_x < 0 {
            tile_x = world.tile_count_x as i32 + tile_x;
            result.tilemap_x -= 1;
        }

        if tile_y < 0 {
            tile_y = world.tile_count_y as i32 + tile_y;
            result.tilemap_y -= 1;
        }

        if tile_x >= world.tile_count_x as i32 {
            tile_x = tile_x - world.tile_count_x as i32;
            result.tilemap_x += 1;
        }

        if tile_y >= world.tile_count_y as i32 {
            tile_y = tile_y - world.tile_count_y as i32;
            result.tilemap_y += 1;
        }

        result.tile_x = tile_x;
        result.tile_y = tile_y;

        result
    }
}


struct World<'a> {
    tilemap_count_x: usize,
    tilemap_count_y: usize,
    
    upper_left_x: f32,
    upper_left_y: f32,
    tile_width: f32,
    tile_height: f32,
    tile_count_x: usize,
    tile_count_y: usize,
    
    tilemaps: &'a[TileMap<'a>],
}


impl<'a> World<'a> {
    fn get_tilemap(&self, tilemap_x: i32, tilemap_y: i32) -> &'a TileMap<'a> {
        &self.tilemaps[(tilemap_y * self.tilemap_count_x as i32 + tilemap_x) as usize]
    }
}


struct TileMap<'a> {
    tiles: &'a[u32]
}

impl<'a> TileMap<'a> {
    fn get_tile_value(&self, tile_x: i32, tile_y: i32, world: &World) -> u32 {
        self.tiles[(tile_y * world.tile_count_x as i32 + tile_x) as usize]
    }
}


fn is_world_point_empty(world: &World, position: &RawPosition) -> bool {

    let can_position = position.canonicalize(&world);

    let tilemap = world.get_tilemap(can_position.tilemap_x, can_position.tilemap_y);

    is_tilemap_point_empty(can_position.tile_x, can_position.tile_y, tilemap, world)
}

fn is_tilemap_point_empty(test_x: i32, test_y: i32, tilemap: &TileMap,
                          world: &World) -> bool {

    let mut is_empty = false;

    if test_x >= 0 && test_x < world.tile_count_x as i32 &&
    test_y >= 0 && test_y < world.tile_count_y as i32 {

        let tile_elem = tilemap.get_tile_value(test_x, test_y, world);
        is_empty = tile_elem == 0;
    }

    is_empty
}



