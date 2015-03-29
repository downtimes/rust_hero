use std::mem;

#[allow(unused_imports)]
use common::{GameMemory, SoundBuffer, VideoBuffer, Input, ReadFileResult};
use common::{ThreadContext};

mod graphics;
mod tilemap;
mod memory;
mod random;
mod math;

use self::tilemap::{TileMap, substract, is_tilemap_point_empty};
use self::tilemap::{TilemapDifference, TilemapPosition, on_same_tile};
use self::memory::MemoryArena;
use self::math::V2f;

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
pub extern fn update_and_render(context: &ThreadContext,
                                game_memory: &mut GameMemory,
                                input: &Input,
                                video_buffer: &mut VideoBuffer) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let state: &mut GameState = 
        unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    //random table index 6 start to get a room with staircase on the first
    //screen
    let mut rand_index = 6;

    if !game_memory.initialized {
        state.test_bitmap = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_background.bmp").unwrap();

        state.hero_bitmaps[0].head = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_right_head.bmp").unwrap();
        state.hero_bitmaps[0].torso = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_right_torso.bmp").unwrap();
        state.hero_bitmaps[0].cape = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_right_cape.bmp").unwrap();
        state.hero_bitmaps[0].align_x = 72;
        state.hero_bitmaps[0].align_y = 182;

        state.hero_bitmaps[1].head = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_back_head.bmp").unwrap();
        state.hero_bitmaps[1].torso = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_back_torso.bmp").unwrap();
        state.hero_bitmaps[1].cape = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_back_cape.bmp").unwrap();
        state.hero_bitmaps[1].align_x = 72;
        state.hero_bitmaps[1].align_y = 182;

        state.hero_bitmaps[2].head = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_left_head.bmp").unwrap();
        state.hero_bitmaps[2].torso = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_left_torso.bmp").unwrap();
        state.hero_bitmaps[2].cape = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_left_cape.bmp").unwrap();
        state.hero_bitmaps[2].align_x = 72;
        state.hero_bitmaps[2].align_y = 182;

        state.hero_bitmaps[3].head = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_front_head.bmp").unwrap();
        state.hero_bitmaps[3].torso = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_front_torso.bmp").unwrap();
        state.hero_bitmaps[3].cape = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_front_cape.bmp").unwrap();
        state.hero_bitmaps[3].align_x = 72;
        state.hero_bitmaps[3].align_y = 182;

        state.camera_position.tile_x = 17/2;
        state.camera_position.tile_y = 9/2;
        state.camera_position.tile_z = 0;

        state.player_position.offset.x = 0.5;
        state.player_position.offset.y = 0.5;
        state.player_position.tile_x = 3;
        state.player_position.tile_y = 3;
        state.player_position.tile_z = 0;

        let game_state_size = mem::size_of::<GameState>();
        state.world_arena = 
            MemoryArena::new(game_memory.permanent.len() - game_state_size,
                             unsafe { game_memory.permanent.as_ptr().offset(game_state_size as isize) });
        let world_arena = &mut state.world_arena;

        state.world = world_arena.push_struct();

        state.world.tilemap = world_arena.push_struct();
        
        let tilemap = &mut state.world.tilemap;

        const CHUNK_SHIFT: u32 = 4;
        const CHUNK_DIM: usize = 1 << CHUNK_SHIFT;

        tilemap.tilechunk_count_x = 60;
        tilemap.tilechunk_count_y = 60;
        tilemap.tilechunk_count_z = 2;
        tilemap.tile_side_meters = 1.4;
        tilemap.chunk_shift = CHUNK_SHIFT;
        tilemap.chunk_mask = CHUNK_DIM as u32 - 1;
        tilemap.chunk_dim = CHUNK_DIM;

        //Allocating enough chunks so our empty map hopefully fits
        tilemap.tilechunks = world_arena.push_slice(tilemap.tilechunk_count_x 
                                                    * tilemap.tilechunk_count_y
                                                    * tilemap.tilechunk_count_z);

        //Generating a random maze
        let tiles_per_screen_x = 17;
        let tiles_per_screen_y = 9;
        let mut screen_y = 0;
        let mut screen_x = 0;

        let mut door_left = false;
        let mut door_right = false;
        let mut door_top = false;
        let mut door_bottom = false;
        let mut door_up = false;
        let mut door_down = false;

        let mut abs_tile_z = 0;

        for _ in 0..100 {

            let random_choice =
                if door_up || door_down {
                    random::NUMBERS[rand_index] % 2
                } else {
                    random::NUMBERS[rand_index] % 3
                };
            rand_index += 1;
            debug_assert!(random_choice < 3);

            let mut created_z = false;

            if random_choice == 0 {
                door_right = true;
            } else if random_choice == 1{
                door_top = true;
            } else {
                created_z = true;
                if abs_tile_z == 0 { 
                    door_up = true;
                } else { 
                    door_down = true;
                };
            }


            for tile_y in 0..tiles_per_screen_y {
                for tile_x in 0..tiles_per_screen_x {
                    let abs_tile_x = screen_x * tiles_per_screen_x + tile_x;
                    let abs_tile_y = screen_y * tiles_per_screen_y +  tile_y;
                    let mut tile_value = 0;
                    //vertical walls
                    if tile_x == 0 {
                        tile_value = 1;

                        if door_left && tile_y == tiles_per_screen_y / 2 {
                            tile_value = 0;
                        }
                    }
                    
                    if tile_x == tiles_per_screen_x - 1 {
                        tile_value = 1;

                        if door_right && tile_y == tiles_per_screen_y / 2 {
                            tile_value = 0;
                        }
                    }
                    //horizontal walls
                    if tile_y == 0 {
                        tile_value = 1;

                        if door_bottom && tile_x == tiles_per_screen_x / 2 {
                            tile_value = 0;
                        }
                    }
                    if tile_y == tiles_per_screen_y - 1 {
                        tile_value = 1;

                        if door_top && tile_x == tiles_per_screen_x / 2 {
                            tile_value = 0;
                        }
                    }

                    //"Staircases"
                    if tile_x == 10 && tile_y == 6 {
                        if door_up {
                            tile_value = 2;
                        } 
                        if door_down {
                            tile_value = 3;
                        }
                    }

                    tilemap.set_tile_value(world_arena, abs_tile_x, 
                                           abs_tile_y, abs_tile_z,
                                           tile_value);
                }
            }

            door_bottom = door_top;
            door_left = door_right;


            if created_z {
                door_up = !door_up;
                door_down = !door_down;
            } else {
                door_up = false;
                door_down = false;
            }

            door_top = false;
            door_right = false;

            if random_choice == 0 {
                screen_x += 1;
            } else if random_choice == 1 {
                screen_y += 1;
            } else {
                if abs_tile_z == 0 {
                    abs_tile_z += 1;
                } else {
                    abs_tile_z -= 1;
                }
            }

            debug_assert!((abs_tile_z as usize) < tilemap.tilechunk_count_z);
        }

        game_memory.initialized = true;
    }

    let world = &state.world;
    
    let tile_side_pixels = 60;
    let meters_to_pixel = tile_side_pixels as f32 / world.tilemap.tile_side_meters;

    let player_dim = V2f{ x: 0.75 * world.tilemap.tile_side_meters, 
                          y: world.tilemap.tile_side_meters };
    let tilemap = &state.world.tilemap;

    for controller in input.controllers.iter() {

        //Analog movement
        if controller.is_analog() {

        //Digital movement
        } else {
            //in m/s^2
            let mut player_acc = V2f{ x: 0.0, y: 0.0 };
            
            if controller.move_up.ended_down {
                state.hero_face_direction = 1;
                player_acc.y = 1.0;
            }
            if controller.move_down.ended_down {
                state.hero_face_direction = 3;
                player_acc.y = -1.0;
            }
            if controller.move_left.ended_down {
                state.hero_face_direction = 2;
                player_acc.x = -1.0;
            }
            if controller.move_right.ended_down {
                state.hero_face_direction = 0;
                player_acc.x = 1.0;
            }
            
            //Diagonal correction. replace with vector length resize when available
            if player_acc.x != 0.0 && player_acc.y != 0.0 {
                player_acc = player_acc * 0.707106781187;
            }

            let player_speed = 
                if controller.start.ended_down {
                    100.0
                } else {
                    10.0
                };

            player_acc = player_acc * player_speed;

            //friction force currently just by rule of thumb;
            player_acc = player_acc - state.player_velocity * 1.5;


            let mut new_position = state.player_position;
            // new_pos = 1/2*a*t^2 + v*t + old_pos;
            new_position.offset = player_acc * 0.5 * input.delta_t.powi(2) + 
                                  state.player_velocity * input.delta_t +
                                  new_position.offset;
            // new_velocity = a*t + old_velocity;
            state.player_velocity = player_acc * input.delta_t + state.player_velocity;
            new_position.recanonicalize(&world.tilemap);

            let mut new_right_bottom = new_position;
            new_right_bottom.offset.x += 0.5*player_dim.x;
            new_right_bottom.recanonicalize(&world.tilemap);

            let mut new_left_bottom = new_position;
            new_left_bottom.offset.x -= 0.5*player_dim.x;
            new_left_bottom.recanonicalize(&world.tilemap);

            let mut col_p = state.player_position;
            let collided = 
                if !is_tilemap_point_empty(&world.tilemap, &new_position) {
                    col_p = new_position;
                    true
                } else if !is_tilemap_point_empty(&world.tilemap, &new_right_bottom) {
                    col_p = new_right_bottom;
                    true
                } else if !is_tilemap_point_empty(&world.tilemap, &new_left_bottom) {
                    col_p = new_left_bottom;
                    true
                } else {
                    false
                };

            if !collided {
                   let player_p = &mut state.player_position;

                   //trigger stuff if we change tiles
                   if !on_same_tile(player_p, &new_position) {
                       let tile_value = world.tilemap.get_tile_value(new_position.tile_x,
                                                                     new_position.tile_y,
                                                                     new_position.tile_z);
                       if let Some(value) = tile_value {
                           if value == 2 {
                               new_position.tile_z += 1;
                           } else if value == 3 {
                               new_position.tile_z -= 1;
                           }
                       }
                   }

                   //accept the move
                   *player_p = new_position;
                   
                   //Adjust the camera to look at the right room
                   let cam_pos = &mut state.camera_position;
                   cam_pos.tile_z = new_position.tile_z;

                   let TilemapDifference { dx, dy, dz: _ } = 
                       substract(tilemap, player_p, cam_pos);

                   if dx > 9.0 * tilemap.tile_side_meters {
                       cam_pos.tile_x += 17;
                   } else if dx < -(9.0 * tilemap.tile_side_meters) {
                       cam_pos.tile_x -= 17;
                   }

                   if dy > 5.0 * tilemap.tile_side_meters {
                       cam_pos.tile_y += 9;
                   } else if dy < -(5.0 * tilemap.tile_side_meters) {
                       cam_pos.tile_y -= 9;
                   }
            } else {
                let r =
                    if col_p.tile_x < state.player_position.tile_x {
                        V2f { x: -1.0, y: 0.0 }
                    } else if col_p.tile_x > state.player_position.tile_x {
                        V2f { x: 1.0, y: 0.0 }
                    } else if col_p.tile_y < state.player_position.tile_y {
                        V2f { x: 0.0, y: 1.0 }
                    } else if col_p.tile_y > state.player_position.tile_y {
                        V2f { x: 0.0, y: -1.0 }
                    } else {
                        V2f { x: 0.0, y: 0.0 }
                    };

                state.player_velocity = state.player_velocity - r * 
                                        math::dot(r, state.player_velocity) * 1.0;
            }
        }
    }

    //Clear the screen to pink!
    let buffer_dim = V2f{ x: video_buffer.width as f32, y: video_buffer.height as f32 };
    graphics::draw_rect(video_buffer, V2f{ x: 0.0, y: 0.0 }, buffer_dim, 
                        1.0, 0.0, 1.0);

    graphics::draw_bitmap(video_buffer, &state.test_bitmap, 0.0, 0.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    for rel_row in -6i32..6 {
        for rel_column in -10i32..10 {
            let row = (rel_row + state.camera_position.tile_y as i32) as u32;
            let column = (rel_column + state.camera_position.tile_x as i32) as u32;

            let elem = world.tilemap.get_tile_value(column as u32, row as u32,
                                                    state.camera_position.tile_z);
            if let Some(value) = elem {
                if value > 0 {
                    let mut color = 0.5;
                    if value == 1 {
                        color = 1.0;
                    } else if value > 1 {
                        color = 0.3;
                    }

                    if row == state.player_position.tile_y &&
                       column == state.player_position.tile_x {
                           color = 0.8;
                    }

                    let center = V2f{ x: screen_center_x - meters_to_pixel * state.camera_position.offset.x
                                           + rel_column as f32 * tile_side_pixels as f32,
                                       y: screen_center_y + meters_to_pixel * state.camera_position.offset.y
                                           - rel_row as f32 * tile_side_pixels as f32,
                                    };

                    let min = center - 0.5 * tile_side_pixels as f32;
                    let max = min + tile_side_pixels as f32;
                    graphics::draw_rect(video_buffer, min, max,
                                        color, color, color);
                }
            }
        }
    }


    let TilemapDifference { dx, dy, dz: _ } = substract(tilemap, 
                                                        &state.player_position, 
                                                        &state.camera_position);
    let player_groundpoint = V2f{
        x: screen_center_x + meters_to_pixel * dx,
        y: screen_center_y - meters_to_pixel * dy,
    };
    let player_dim_half_x = V2f{ 
        x: 0.5 * player_dim.x, 
        y: player_dim.y,
    };
    let player_top_left = player_groundpoint - player_dim_half_x * meters_to_pixel;
    let player_bottom_right = player_top_left + player_dim * meters_to_pixel;
    graphics::draw_rect(video_buffer, 
                        player_top_left, player_bottom_right,
                        1.0, 1.0, 0.0);
    
    let hero_bitmaps = &state.hero_bitmaps[state.hero_face_direction];
    graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.torso,
                                  player_groundpoint, hero_bitmaps.align_x,
                                  hero_bitmaps.align_y);
    graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.cape,
                                  player_groundpoint, hero_bitmaps.align_x,
                                  hero_bitmaps.align_y);
    graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.head,
                                  player_groundpoint, hero_bitmaps.align_x,
                                  hero_bitmaps.align_y);


}

// ======== End of the public interface =========


struct HeroBitmaps<'a> {
    head: graphics::Bitmap<'a>,
    torso: graphics::Bitmap<'a>,
    cape: graphics::Bitmap<'a>,

    align_x: i32,
    align_y: i32
}

struct GameState<'a> {
    world_arena: MemoryArena,
    player_position: TilemapPosition,
    player_velocity: V2f,
    camera_position: TilemapPosition,

    world: &'a mut World<'a>,

    test_bitmap: graphics::Bitmap<'a>,
    hero_bitmaps: [HeroBitmaps<'a>; 4],
    hero_face_direction: usize,
} 

struct World<'a> {
    tilemap: &'a mut TileMap<'a>,
}

