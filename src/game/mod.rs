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

        game_memory.initialized = true;
    }
    
    const CHUNK_SHIFT: u32 = 5;
    const CHUNK_DIM: usize = 1 << CHUNK_SHIFT;

    let tiles0: [u32; CHUNK_DIM * CHUNK_DIM] = 
        [
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1 ,  1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1 ,
             1, 1, 0, 0,   0, 1, 0, 0,   0, 0, 0, 0,   0, 1, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 1, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 1, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   1, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 1, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   0, 1, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 1, 0, 0,   1, 0, 0, 0,   1, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 0, 0, 0,   0, 0, 0, 0,   0, 1, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1 ,  1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1 , 
             1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1 ,  1, 1, 1, 1,   1, 1, 1, 1,   0, 1, 1, 1,   1, 1, 1, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,  1, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 1 ,
             1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1 ,  1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1,   1, 1, 1, 1 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 , 
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
             0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,  0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0,   0, 0, 0, 0 ,
        ];

    let tilechunks = [TileChunk {
                        tiles: &tiles0[..],
                    }];


    let world = World {
                    tile_side_pixels: 60,
                    tile_side_meters: 1.4,
                    tilechunk_count_x: 1,
                    _tilechunk_count_y: 1,
                    chunk_shift: CHUNK_SHIFT,
                    _chunk_mask: CHUNK_DIM as u32 - 1,
                    chunk_dim: CHUNK_DIM,
                    tilechunks: &tilechunks[..]
                };

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
                dplayer_y = 1.0;
            }
            if controller.move_down.ended_down {
                dplayer_y = -1.0;
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

    //Clear the screen to pink!
    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    graphics::draw_rect(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
                        1.0, 0.0, 1.0);

    let chunk_pos = get_chunk_position(&world, state.player_position.tile_x,
                                      state.player_position.tile_y);
    let current_map = world.get_tilechunk(&chunk_pos);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    for rel_row in -10i32..10 {
        for rel_column in -18i32..18 {
            let column = (rel_column + state.player_position.tile_x as i32) as u32;
            let row = (rel_row + state.player_position.tile_y as i32) as u32;

            let elem = current_map.get_tile_value(column as u32, row as u32, &world);
            let mut color = 
                if elem == 0 {
                    0.5
                } else {
                    1.0
                };

            if column == state.player_position.tile_x && 
               row == state.player_position.tile_y {
                color = 0.8;
            }

            let min_x = screen_center_x + rel_column as f32 * world.tile_side_pixels as f32;
            let min_y = screen_center_y - rel_row as f32 * world.tile_side_pixels as f32;
            let max_x = min_x + world.tile_side_pixels as f32;
            let max_y = min_y - world.tile_side_pixels as f32;
            graphics::draw_rect(video_buffer, min_x, max_y, max_x, min_y,
                                color, color, color);
        }
    }


    let player_top = screen_center_y - world.meters_to_pixels(state.player_position.tile_rel_y) - 
                     world.meters_to_pixels(player_height);
    let player_left = screen_center_x + world.meters_to_pixels(state.player_position.tile_rel_x) - 
                      world.meters_to_pixels(0.5 * player_width);
    graphics::draw_rect(video_buffer, 
                        player_left, player_top,
                        player_left + world.meters_to_pixels(player_width),
                        player_top + world.meters_to_pixels(player_height),
                        1.0, 1.0, 0.0);

}

// ======== End of the public interface =========


struct GameState {
    player_position: WorldPosition,
} 

struct TileChunkPosition {
    tilechunk_x: u32,
    tilechunk_y: u32,
}


fn get_chunk_position(world: &World, tile_x: u32, tile_y: u32) -> TileChunkPosition {
    TileChunkPosition {
        tilechunk_x: tile_x >> world.chunk_shift,
        tilechunk_y: tile_y >> world.chunk_shift,
        //tile_x: tile_x & world.chunk_mask,
        //tile_y: tile_y & world.chunk_mask,
    }
}


#[derive(Copy)]
struct WorldPosition {
    tile_x: u32,
    tile_y: u32,
    //Tile relative
    tile_rel_x: f32,
    tile_rel_y: f32,
}

fn canonicalize_coord(world: &World, tile: &mut u32, tile_rel: &mut f32) {

        let offset = (*tile_rel / world.tile_side_meters as f32).floor();

        let new_tile = *tile as i32 + offset as i32;
        *tile = new_tile as u32;

        *tile_rel -= offset * world.tile_side_meters;

        //TODO: for small offset values this debug_assert triggers!
        //rounding puts us back on the tile we came from
        debug_assert!(*tile_rel >= 0.0 && *tile_rel < world.tile_side_meters as f32);
}

impl WorldPosition {
    fn recanonicalize(&mut self, world: &World) {

        canonicalize_coord(&world, &mut self.tile_x, &mut self.tile_rel_x);
        canonicalize_coord(&world, &mut self.tile_y, &mut self.tile_rel_y);
    }
}

struct World<'a> {
    tile_side_pixels: u32,
    tile_side_meters: f32,

    tilechunk_count_x: usize,
    _tilechunk_count_y: usize,

    chunk_shift: u32,
    _chunk_mask: u32,
    chunk_dim: usize,
    tilechunks: &'a[TileChunk<'a>],
}


impl<'a> World<'a> {
    fn get_tilechunk(&self, chunk_pos: &TileChunkPosition) -> &'a TileChunk<'a> {
        &self.tilechunks[(chunk_pos.tilechunk_y * self.tilechunk_count_x as u32
                        + chunk_pos.tilechunk_x) as usize]
    }

    fn meters_to_pixels(&self, meters: f32) -> f32 {
        meters * self.tile_side_pixels as f32 / self.tile_side_meters
    }
}


struct TileChunk<'a> {
    tiles: &'a[u32]
}

impl<'a> TileChunk<'a> {
    fn get_tile_value(&self, tile_x: u32, tile_y: u32, world: &World) -> u32 {
        let index = tile_y as usize * world.chunk_dim + tile_x as usize;
        if index > self.tiles.len() {
            0
        } else {
            self.tiles[index]
        }
    }
}


fn is_world_point_empty(world: &World, position: &WorldPosition) -> bool {
    get_tile_value(&world, &position) == 0
}


fn get_tile_value(world: &World, position: &WorldPosition) -> u32 {

    let chunk_pos = get_chunk_position(&world, position.tile_x, position.tile_y);
    let tilechunk = world.get_tilechunk(&chunk_pos);

    tilechunk.get_tile_value(position.tile_x, position.tile_y, world)
}

