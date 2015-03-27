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
        state.player_position.tile_rel_x = 1.0;
        state.player_position.tile_rel_y = 1.0;
        state.player_position.tile_x = 3;
        state.player_position.tile_y = 3;
        state.player_position.tilemap_x = 0;
        state.player_position.tilemap_y = 0;

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
                    tile_side_pixels: 60,
                    upper_left_x: -60.0/2.0,
                    upper_left_y: 0.0,
                    tile_side_meters: 1.4,
                    tilemap_count_x: 2,
                    _tilemap_count_y: 2,
                    tilemaps: &tilemaps[..]
                };

    let current_map = world.get_tilemap(state.player_position.tilemap_x, state.player_position.tilemap_y);

    let player_width = 0.75 * world.tile_side_meters;
    let player_height = world.tile_side_meters;

    for controller in input.controllers.iter() {

        //Analog movement
        if controller.is_analog() {

        //Digital movement
        } else {
            let mut dplayer_x = 0.0; //In Meters per second
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

            dplayer_x *= 2.0;
            dplayer_y *= 2.0;

            let mut new_position = state.player_position;
            new_position.tile_rel_x += dplayer_x * input.delta_time;
            new_position.tile_rel_y += dplayer_y * input.delta_time;
            new_position.recanonicalize(&world);

            let mut player_right_bottom = new_position;
            player_right_bottom.tile_rel_x += 0.5*player_width;
            player_right_bottom.recanonicalize(&world);

            let mut player_left_bottom = new_position;
            player_left_bottom.tile_rel_x -= 0.5*player_width;
            player_left_bottom.recanonicalize(&world);

            if is_world_point_empty(&world, &player_left_bottom) &&
               is_world_point_empty(&world, &player_right_bottom) &&
               is_world_point_empty(&world, &state.player_position) {
                   state.player_position = new_position;
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
            let mut color = 
                if elem == 0 {
                    0.5
                } else {
                    1.0
                };

            if column == state.player_position.tile_x as usize && 
               row == state.player_position.tile_y as usize {
                color = 0.8;
            }
            let min_x = world.upper_left_x + column as f32 * world.tile_side_pixels as f32;
            let min_y = world.upper_left_y + row as f32 * world.tile_side_pixels as f32;
            let max_x = min_x + world.tile_side_pixels as f32;
            let max_y = min_y + world.tile_side_pixels as f32;
            graphics::draw_rect(video_buffer, min_x, min_y, max_x, max_y,
                                color, color, color);
        }
    }


    let player_top = world.upper_left_y + world.tile_side_pixels as f32 * 
                     state.player_position.tile_y as f32 + 
                     world.meters_to_pixels(state.player_position.tile_rel_y) - 
                     world.meters_to_pixels(player_height);
    let player_left = world.upper_left_x + world.tile_side_pixels as f32 * 
                      state.player_position.tile_x as f32 +
                      world.meters_to_pixels(state.player_position.tile_rel_x) - 
                      world.meters_to_pixels(0.5 * player_width);
    graphics::draw_rect(video_buffer, 
                        player_left, player_top,
                        player_left + world.meters_to_pixels(player_width),
                        player_top + world.meters_to_pixels(player_height),
                        1.0, 1.0, 0.0);

}

// ======== End of the public interface =========


struct GameState {
    player_position: CanonicalPosition,
} 


#[derive(Default, Copy)]
struct CanonicalPosition {
    tilemap_x: i32,
    tilemap_y: i32,

    tile_x: i32,
    tile_y: i32,

    //Tile relative
    tile_rel_x: f32,
    tile_rel_y: f32,
}

fn canonicalize_coord(world: &World, tile_count: usize, tilemap: &mut i32, tile: &mut i32, 
                      tile_rel: &mut f32) {

        let offset = (*tile_rel / world.tile_side_meters as f32).floor();
        *tile += offset as i32;

        *tile_rel -= offset * world.tile_side_meters;

        //TODO: for small offset values this debug_assert triggers!
        //rounding puts us back on the tile we came from
        debug_assert!(*tile_rel >= 0.0 && *tile_rel < world.tile_side_meters as f32);

        if *tile < 0 {
            *tile = tile_count as i32 + *tile;
            *tilemap -= 1;
        }

        if *tile >= tile_count as i32 {
            *tile = *tile - tile_count as i32;
            *tilemap += 1;
        }
}

impl CanonicalPosition {
    fn recanonicalize(&mut self, world: &World) {

        canonicalize_coord(&world, world.tile_count_x, &mut self.tilemap_x, 
                           &mut self.tile_x, &mut self.tile_rel_x);
        canonicalize_coord(&world, world.tile_count_y, &mut self.tilemap_y,
                           &mut self.tile_y, &mut self.tile_rel_y);
    }
}

struct World<'a> {
    tilemap_count_x: usize,
    _tilemap_count_y: usize,
    
    upper_left_x: f32,
    upper_left_y: f32,
    tile_count_x: usize,
    tile_count_y: usize,

    tile_side_pixels: u32,
    tile_side_meters: f32,
    
    tilemaps: &'a[TileMap<'a>],
}


impl<'a> World<'a> {
    fn get_tilemap(&self, tilemap_x: i32, tilemap_y: i32) -> &'a TileMap<'a> {
        &self.tilemaps[(tilemap_y * self.tilemap_count_x as i32 + tilemap_x) as usize]
    }

    fn meters_to_pixels(&self, meters: f32) -> f32 {
        meters * self.tile_side_pixels as f32 / self.tile_side_meters
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


fn is_world_point_empty(world: &World, position: &CanonicalPosition) -> bool {

    let tilemap = world.get_tilemap(position.tilemap_x, position.tilemap_y);

    is_tilemap_point_empty(position.tile_x, position.tile_y, tilemap, world)
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



