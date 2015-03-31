use std::mem;
use std::default::Default;
use std::rc::Rc;
use std::cell::RefCell;

use common::{GameMemory, SoundBuffer, VideoBuffer, Input};
use common::{ThreadContext, MAX_CONTROLLERS};

mod graphics;
mod tilemap;
mod memory;
mod random;
mod math;

use self::tilemap::{TileMap, subtract, is_tile_value_empty};
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

        //TODO: move this from the Heap into our own memory region!
        for index in 0..MAX_ENTITIES {
            state.hf_entities[index] = Rc::new(RefCell::new(Default::default()));
            state.lf_entities[index] = Rc::new(RefCell::new(Default::default()));
            state.dorm_entities[index] = Rc::new(RefCell::new(Default::default()));
        }

        game_memory.initialized = true;
    }

    for (c_index, controller) in input.controllers.iter().enumerate() {

        if let Some(e_index) = state.player_index_for_controller[c_index] {
            //in m/s^2
            let mut acc = V2f { x: 0.0, y: 0.0 };

            //Analog movement
            if controller.is_analog() {
                let avg_x = controller.average_x.unwrap_or_default();
                let avg_y = controller.average_y.unwrap_or_default();
                acc = V2f { x: avg_x, y: avg_y };

            //Digital movement
            } else {
                if controller.move_up.ended_down {
                    acc.y = 1.0;
                }
                if controller.move_down.ended_down {
                    acc.y = -1.0;
                }
                if controller.move_left.ended_down {
                    acc.x = -1.0;
                }
                if controller.move_right.ended_down {
                    acc.x = 1.0;
                }
            }

            move_player(e_index, acc, state, input.delta_t);
        } else {
            if controller.start.ended_down {
                let e_index = add_entity(state);
                state.player_index_for_controller[c_index] = Some(e_index);
                add_player(state, e_index);
            }
        }
    }

    let world = &state.world;
    
    let tile_side_pixels = 60;
    let meters_to_pixel = tile_side_pixels as f32 / world.tilemap.tile_side_meters;


    
    //Adjust the camera to look at the right room
    if state.camera_follows_entity_index.is_some() {
        let cam_entity_idx = state.camera_follows_entity_index.unwrap();
        let camera_entity = get_entity(state, Residence::High, cam_entity_idx);
        let cam_pos = &mut state.camera_position;
        let dorm = camera_entity.dorm.borrow();
        cam_pos.tile_z = dorm.tile_position.tile_z;

        let tilemap = &state.world.tilemap;
        //TODO: FIX THIS
        let TilemapDifference { dx, dy, dz: _ } = 
            subtract(tilemap, &dorm.tile_position, cam_pos);

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
    }

    //Clear the screen to pink! And start rendering
    let buffer_dim = V2f{ x: video_buffer.width as f32, y: video_buffer.height as f32 };
    graphics::draw_rect(video_buffer, Default::default(), buffer_dim, 
                        1.0, 0.0, 1.0);

    graphics::draw_bitmap(video_buffer, &state.test_bitmap, 0.0, 0.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    let cam_pos = &mut state.camera_position;
    for rel_row in -6i32..6 {
        for rel_column in -10i32..10 {
            let row = (rel_row + cam_pos.tile_y as i32) as u32;
            let column = (rel_column + cam_pos.tile_x as i32) as u32;

            let elem = world.tilemap.get_tile_value(column as u32, row as u32,
                                                    cam_pos.tile_z);
            if let Some(value) = elem {
                if value > 0 {
                    let mut color = 0.5;
                    if value == 1 {
                        color = 1.0;
                    } else if value > 1 {
                        color = 0.3;
                    }

                    let center = V2f{ x: screen_center_x - meters_to_pixel * cam_pos.offset.x
                                           + rel_column as f32 * tile_side_pixels as f32,
                                       y: screen_center_y + meters_to_pixel * cam_pos.offset.y
                                           - rel_row as f32 * tile_side_pixels as f32, };

                    let tile_side_pixels_v = V2f{ x: tile_side_pixels as f32,
                                                  y: tile_side_pixels as f32, };
                    let min = center - tile_side_pixels_v * 0.5;
                    let max = min + tile_side_pixels_v;
                    graphics::draw_rect(video_buffer, min, max,
                                        color, color, color);
                }
            }
        }
    }


    for index in 0..state.entity_count {
        let residence = &state.entity_residence[index];
        match residence {
             &Residence::High => {
                let entity = state.hf_entities[index].borrow();
                let dorm_part = state.dorm_entities[index].borrow();

                let entity_groundpoint = V2f{
                    x: screen_center_x + meters_to_pixel * entity.position.x,
                    y: screen_center_y - meters_to_pixel * entity.position.y,
                };
                let entity_top_left = entity_groundpoint - dorm_part.dim * 0.5 * meters_to_pixel;
                let entity_bottom_right = entity_top_left + dorm_part.dim * meters_to_pixel;
                graphics::draw_rect(video_buffer, 
                                    entity_top_left, entity_bottom_right,
                                    1.0, 1.0, 0.0);

                let hero_bitmaps = &state.hero_bitmaps[entity.face_direction];
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.torso,
                                              entity_groundpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.cape,
                                              entity_groundpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.head,
                                              entity_groundpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
            },
            _ => {},
        }
    }

}

// ======== End of the public interface =========

fn get_entity(state: &GameState, res: Residence, index: usize) -> Entity {
    debug_assert!(index < state.entity_count);
    Entity {
        residence: res,
        dorm: state.dorm_entities[index].clone(),
        lf: state.lf_entities[index].clone(),
        hf: state.hf_entities[index].clone(),
    }
}

fn add_entity<'a>(state: &'a mut GameState) -> usize {
    let index = state.entity_count;

    state.entity_count += 1;

    state.entity_residence[index] = Residence::Dormant;
    state.dorm_entities[index] = Rc::new(RefCell::new(Default::default()));
    state.hf_entities[index] = Rc::new(RefCell::new(Default::default()));
    state.lf_entities[index] = Rc::new(RefCell::new(Default::default()));

    index
}

fn add_player<'a>(state: &'a mut GameState, e_index: usize) {
    if state.camera_follows_entity_index.is_none() {
        state.camera_follows_entity_index = Some(e_index);
    }
    let entity = get_entity(state, Residence::Dormant, e_index);
    {
        let hf_entity = entity.hf.borrow_mut();
        let mut dorm_entity = entity.dorm.borrow_mut();


        state.entity_count += 1;

        dorm_entity.dim = V2f{ x: 1.0, 
                               y: 0.5 };
        dorm_entity.tile_position.tile_x = 3;
        dorm_entity.tile_position.tile_y = 3;
        dorm_entity.tile_position.tile_z = 0;
    }

    change_entity_residence(state, entity, Residence::High);
}

fn change_entity_residence<'a>(state: &GameState, entity: Entity, res: Residence) {
    match res {
        Residence::High => {
            match entity.residence {
                Residence::High => {},
                _ => {
                    //map to camera space
                },
            }
        },
        _ => {
        },
    }
}
           

fn move_player<'a>(entity_index: usize, mut acc: V2f, 
                   state: &'a mut GameState<'a>, delta_t: f32) {
    let entity = get_entity(state, Residence::High, entity_index);
    let tilemap = &state.world.tilemap;

    //Diagonal correction.
    if acc.length_sq() > 1.0 {
        acc = acc.normalize();
    }

    let entity_speed = 50.0; // m/s^2

    acc = acc * entity_speed;

    let mut hf_entity = entity.hf.borrow_mut();
    //friction force currently just by rule of thumb;
    acc = acc - hf_entity.velocity * 8.0;


    //Copy old player Position before we handle input 
    let mut entity_delta = acc * 0.5 * delta_t.powi(2) 
                       + hf_entity.velocity * delta_t;
    hf_entity.velocity = acc * delta_t + hf_entity.velocity;

    let new_position = hf_entity.position + entity_delta; 


/*

    let entity_tile_dim = V2f { x: (entity.dim.x / world.tilemap.tile_side_meters).ceil(),
                                y: (entity.dim.y / world.tilemap.tile_side_meters).ceil(), };

    let min_tile_x = cmp::min(entity.position.tile_x as i32, 
                              new_position.tile_x as i32) 
                     - entity_tile_dim.x as i32;
    let min_tile_y = cmp::min(entity.position.tile_y as i32, 
                              new_position.tile_y as i32) 
                     - entity_tile_dim.y as i32;
    let max_tile_x = cmp::max(entity.position.tile_x as i32,
                              new_position.tile_x as i32) 
                     + entity_tile_dim.x as i32 + 1;
    let max_tile_y = cmp::max(entity.position.tile_y as i32, 
                              new_position.tile_y as i32) 
                     + entity_tile_dim.y as i32 + 1;
*/

    let mut t_remaining = 1.0;
    //try the collission detection multiple times to see if we can move with
    //a corrected velocity after a collision
    for _ in 0..4 {
        let mut t_min = 1.0;
        let mut wall_normal = Default::default();
        let mut hit_e_index = None;

        for e_index in 0..state.entity_count {
            if e_index == entity_index {
                continue;
            }
            let dorm_entity = entity.dorm.borrow();
            let test_entity = get_entity(state, Residence::High, e_index);
            let hf_test_entity = test_entity.hf.borrow();
            let dorm_test_entity = test_entity.dorm.borrow();
            if dorm_test_entity.collides {
                //Minkowski Sum
                let diameter = V2f { x: dorm_test_entity.dim.x + dorm_entity.dim.x, 
                                     y: dorm_test_entity.dim.y + dorm_entity.dim.y};

                let min_corner = diameter * -0.5;
                let max_corner = diameter * 0.5;
                let rel = hf_entity.position - hf_test_entity.position;

                //check against the 4 entity walls
                if test_wall(max_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2f{ x: 1.0, y: 0.0 };
                }
                if test_wall(min_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2f{ x: -1.0, y: 0.0 };
                }
                if test_wall(max_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2f{ x: 0.0, y: 1.0 };
                }
                if test_wall(min_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2f{ x: 0.0, y: -1.0 };
                }
            }
        }

        hf_entity.position = hf_entity.position + entity_delta * t_min;
        if hit_e_index.is_some() {
            hf_entity.velocity = hf_entity.velocity 
                - wall_normal * math::dot(hf_entity.velocity, wall_normal);
            entity_delta = entity_delta - wall_normal * math::dot(entity_delta, wall_normal);
            t_remaining -= t_min * t_remaining;

            let hit_entity = get_entity(state, Residence::Dormant, hit_e_index.unwrap());
            let dorm_hit = hit_entity.dorm.borrow();
            let tile_z = hf_entity.tile_z as i32  + dorm_hit.d_tile_z;
            hf_entity.tile_z = tile_z as u32;
        }

        //We walked as far as we want to
        if t_remaining <= 0.0 {
            break;
        }
    }

    //adjust facing direction depending on velocity
    if hf_entity.velocity.x.abs() > hf_entity.velocity.y.abs() {
        if hf_entity.velocity.x > 0.0 {
            hf_entity.face_direction = 0;
        } else {
            hf_entity.face_direction = 2;
        }
    } else if hf_entity.velocity.x != 0.0 && hf_entity.velocity.y != 0.0 {
        if hf_entity.velocity.y > 0.0 {
            hf_entity.face_direction = 1;
        } else {
            hf_entity.face_direction = 3;
        }
    }
}

fn test_wall(wall_value: f32, min_ortho: f32, max_ortho: f32, 
             rel_x: f32, rel_y: f32, delta_x: f32, delta_y: f32,
             t_min: &mut f32) -> bool {
     let t_epsilon = 0.00001;
     if delta_x != 0.0 {
         let t_res = (wall_value - rel_x) / delta_x;
         if t_res >= 0.0 && t_res < *t_min {
             let y = rel_y + t_res * delta_y;
             if min_ortho <= y && y <= max_ortho {
                 if t_res - t_epsilon < 0.0 {
                     *t_min = 0.0;
                     return true;
                 }  else {
                     *t_min = t_res - t_epsilon;
                     return true;
                 }
             }
         }
     }
     return false
}

struct HeroBitmaps<'a> {
    head: graphics::Bitmap<'a>,
    torso: graphics::Bitmap<'a>,
    cape: graphics::Bitmap<'a>,

    align_x: i32,
    align_y: i32
}

struct Entity {
    residence: Residence,
    dorm: Rc<RefCell<DormEntity>>,
    lf: Rc<RefCell<LfEntity>>,
    hf: Rc<RefCell<HfEntity>>,
}

#[derive(Default)]
struct DormEntity {
    tile_position: TilemapPosition,
    dim: V2f,
    collides: bool,
    d_tile_z: i32,
}

#[derive(Default)]
struct LfEntity; 

#[derive(Default)]
struct HfEntity {
    position: V2f, //This position is relative to the camera
    velocity: V2f,
    tile_z: u32,
    face_direction: usize,
}

//NOTE: The order is important here. If High == 0 then this will crash
//at startup because we don't initalize the entities
enum Residence {
    NonExistent,
    Low,
    Dormant,
    High,
}

const MAX_ENTITIES: usize = 256;

//TODO: need to think of ways to restructure this so we can make the 
//borrow checker happy
struct GameState<'a> {
    world_arena: MemoryArena,
    world: &'a mut World<'a>,

    camera_follows_entity_index: Option<usize>,
    camera_position: TilemapPosition,

    player_index_for_controller: [Option<usize>; MAX_CONTROLLERS],
    entity_count: usize,
    entity_residence: [Residence; MAX_ENTITIES],
    dorm_entities: [Rc<RefCell<DormEntity>>; MAX_ENTITIES],
    lf_entities: [Rc<RefCell<LfEntity>>; MAX_ENTITIES],
    hf_entities: [Rc<RefCell<HfEntity>>; MAX_ENTITIES],

    test_bitmap: graphics::Bitmap<'a>,
    hero_bitmaps: [HeroBitmaps<'a>; 4],
} 

struct World<'a> {
    tilemap: &'a mut TileMap<'a>,
}

