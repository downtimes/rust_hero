use std::mem;
use std::slice;

#[allow(unused_imports)]
use common::{GameMemory, SoundBuffer, VideoBuffer, Input, ReadFileResult};
use common::{ThreadContext};

mod graphics;
mod tilemap;

use self::tilemap::{TileMap, is_tilemap_point_empty};
use self::tilemap::{TilemapPosition};

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
        state.player_position.offset_x = 0.5;
        state.player_position.offset_y = 0.5;
        state.player_position.tile_x = 3;
        state.player_position.tile_y = 3;

        let game_state_size = mem::size_of::<GameState>();
        state.world_arena = MemoryArena::new(game_memory.permanent.len() - game_state_size,
                                        unsafe { game_memory.permanent.as_ptr().offset(game_state_size as isize) });
        let world_arena = &mut state.world_arena;

        state.world = world_arena.push_struct();

        state.world.tilemap = world_arena.push_struct();
        
        let tilemap = &mut state.world.tilemap;

        const CHUNK_SHIFT: u32 = 8;
        const CHUNK_DIM: usize = 1 << CHUNK_SHIFT;

        tilemap.tile_side_pixels = 60;
        tilemap.tile_side_meters = 1.4;
        tilemap.tilechunk_count_x = 4;
        tilemap.tilechunk_count_y = 4;
        tilemap.chunk_shift = CHUNK_SHIFT;
        tilemap.chunk_mask = CHUNK_DIM as u32 - 1;

        //Allocating enough chunks so our empty map hopefully fits
        tilemap.tilechunks = world_arena.push_slice(tilemap.tilechunk_count_x 
                                                    * tilemap.tilechunk_count_y);

        for chunk_y in 0..tilemap.tilechunk_count_x {
            for chunk_x in 0..tilemap.tilechunk_count_y {
                tilemap.tilechunks[chunk_y * tilemap.tilechunk_count_x + chunk_x].chunk_dim = CHUNK_DIM;
                tilemap.tilechunks[chunk_y * tilemap.tilechunk_count_x + chunk_x].tiles = 
                    world_arena.push_slice(CHUNK_DIM * CHUNK_DIM);
            }
        }

        //Generating a random
        let tiles_per_screen_x = 17;
        let tiles_per_screen_y = 9;

        for screen_y in 0..32 {
            for screen_x in 0..32 {
                for tile_y in 0..tiles_per_screen_y {
                    for tile_x in 0..tiles_per_screen_x {
                        let abs_tile_x = screen_x * tiles_per_screen_x + tile_x;
                        let abs_tile_y = screen_y * tiles_per_screen_y +  tile_y;
                        let value = 
                            if tile_x == tile_y && tile_y % 2 == 0{
                                1
                            } else {
                                0
                            };
                        tilemap.set_tile_value(world_arena, abs_tile_x, 
                                               abs_tile_y, value);
                    }
                }
            }
        }


        game_memory.initialized = true;
    }

    let world = &state.world;
    
    let player_width = 0.75 * world.tilemap.tile_side_meters;
    let player_height = world.tilemap.tile_side_meters;

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

            let player_speed = 
                if controller.start.ended_down {
                    10.0
                } else {
                    2.0
                };

            dplayer_x *= player_speed;
            dplayer_y *= player_speed;

            let mut new_position = state.player_position;
            new_position.offset_x += dplayer_x * input.delta_time;
            new_position.offset_y += dplayer_y * input.delta_time;
            new_position.recanonicalize(&world.tilemap);

            let mut player_right_bottom = new_position;
            player_right_bottom.offset_x += 0.5*player_width;
            player_right_bottom.recanonicalize(&world.tilemap);

            let mut player_left_bottom = new_position;
            player_left_bottom.offset_x -= 0.5*player_width;
            player_left_bottom.recanonicalize(&world.tilemap);

            if is_tilemap_point_empty(&world.tilemap, &player_left_bottom) &&
               is_tilemap_point_empty(&world.tilemap, &player_right_bottom) &&
               is_tilemap_point_empty(&world.tilemap, &state.player_position) {
                   state.player_position = new_position;
            }
        }
    }

    //Clear the screen to pink!
    let buffer_width = video_buffer.width;
    let buffer_height = video_buffer.height;
    graphics::draw_rect(video_buffer, 0.0, 0.0, buffer_width as f32, buffer_height as f32,
                        1.0, 0.0, 1.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    for rel_row in -10i32..10 {
        for rel_column in -18i32..18 {
            let row = (rel_row + state.player_position.tile_y as i32) as u32;
            let column = (rel_column + state.player_position.tile_x as i32) as u32;

            let position = TilemapPosition{
                tile_x: column,
                tile_y: row,
                offset_x: 0.0,
                offset_y: 0.0,
            };
            let elem = world.tilemap.get_tile_value(&position);
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

            let center_x = screen_center_x - world.tilemap.meters_to_pixels(state.player_position.offset_x)
                        + rel_column as f32 * world.tilemap.tile_side_pixels as f32;
            let center_y = screen_center_y + world.tilemap.meters_to_pixels(state.player_position.offset_y)
                        - rel_row as f32 * world.tilemap.tile_side_pixels as f32;
            let min_x = center_x - 0.5 * world.tilemap.tile_side_pixels as f32;
            let min_y = center_y - 0.5 * world.tilemap.tile_side_pixels as f32;
            let max_x = min_x + world.tilemap.tile_side_pixels as f32;
            let max_y = min_y + world.tilemap.tile_side_pixels as f32;
            graphics::draw_rect(video_buffer, min_x, min_y, max_x, max_y,
                                color, color, color);
        }
    }


    let player_top = screen_center_y - world.tilemap.meters_to_pixels(player_height);
    let player_left = screen_center_x - world.tilemap.meters_to_pixels(0.5 * player_width);
    graphics::draw_rect(video_buffer, 
                        player_left, player_top,
                        player_left + world.tilemap.meters_to_pixels(player_width),
                        player_top + world.tilemap.meters_to_pixels(player_height),
                        1.0, 1.0, 0.0);

}

// ======== End of the public interface =========

struct MemoryArena {
    size: usize,
    used: usize,
    base_ptr: *mut u8,
}

impl MemoryArena {
    fn new(new_size: usize, base_ptr: *const u8) -> MemoryArena {
        MemoryArena {
            size: new_size,
            used: 0,
            base_ptr: base_ptr as *mut u8,
        }
    }

    //TODO: Think about clear to zero options
    fn push_struct<T>(&mut self) -> &'static mut T {
        let size = mem::size_of::<T>();
        debug_assert!(self.used + size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += size;
        
        unsafe { mem::transmute(result_ptr) }
    }

    fn push_slice<T>(&mut self, count: usize) -> &'static mut [T] {
        let mem_size = count * mem::size_of::<T>();
        debug_assert!(self.used + mem_size <= self.size);

        let result_ptr = unsafe { self.base_ptr.offset(self.used as isize) };
        self.used += mem_size;
        
        unsafe { slice::from_raw_parts_mut(result_ptr as *mut T, count) }
    }
}

struct GameState<'a> {
    world_arena: MemoryArena,
    player_position: TilemapPosition,
    world: &'a mut World<'a>,
} 

struct World<'a> {
    tilemap: &'a mut TileMap<'a>,
}

