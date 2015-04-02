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

use self::tilemap::{TileMap, subtract};
use self::tilemap::{TilemapDifference, TilemapPosition};
use self::memory::MemoryArena;
use self::math::{V2f, Rectf};

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

        state.shadow = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_hero_shadow.bmp").unwrap();
        
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


        let game_state_size = mem::size_of::<GameState>();
        state.world_arena = 
            MemoryArena::new(game_memory.permanent.len() - game_state_size,
                             unsafe { game_memory.permanent.as_ptr().offset(game_state_size as isize) });

        state.world = state.world_arena.push_struct();

        state.world.tilemap = state.world_arena.push_struct();
        
        {
            let tilemap = &mut state.world.tilemap;

            const CHUNK_SHIFT: u32 = 4;
            const CHUNK_DIM: usize = 1 << CHUNK_SHIFT;

            tilemap.tile_side_meters = 1.4;
            tilemap.chunk_shift = CHUNK_SHIFT;
            tilemap.chunk_mask = (CHUNK_DIM as u32 - 1) as i32;
            tilemap.chunk_dim = CHUNK_DIM;
        }

        //Generating a random maze
        let tiles_per_screen_x = 17;
        let tiles_per_screen_y = 9;
        let screen_base_x = 0;
        let screen_base_y = 0;
        let screen_base_z = 0;
        let mut screen_y = screen_base_y;
        let mut screen_x = screen_base_x;

        let mut door_left = false;
        let mut door_right = false;
        let mut door_top = false;
        let mut door_bottom = false;
        let mut door_up = false;
        let mut door_down = false;

        let mut abs_tile_z = screen_base_z;

        for _ in 0..2 {

            let random_choice = random::NUMBERS[rand_index] % 2;
              //  if door_up || door_down {
              //  } else {
              //      random::NUMBERS[rand_index] % 3
              //  };
            rand_index += 1;
            debug_assert!(random_choice < 3);

            let mut created_z = false;

            if random_choice == 0 {
                door_right = true;
            } else if random_choice == 1{
                door_top = true;
            } else {
                created_z = true;
                if abs_tile_z == screen_base_z { 
                    door_up = true;
                } else { 
                    door_down = true;
                };
            }


            for tile_y in 0..tiles_per_screen_y {
                for tile_x in 0..tiles_per_screen_x {
                    let abs_tile_x = screen_x * tiles_per_screen_x + tile_x;
                    let abs_tile_y = screen_y * tiles_per_screen_y + tile_y;
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

                    state.world.tilemap.set_tile_value(&mut state.world_arena, abs_tile_x, 
                                           abs_tile_y, abs_tile_z,
                                           tile_value);
                    if tile_value == 1 {
                        add_wall(state, abs_tile_x, abs_tile_y, abs_tile_z);
                    }
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
                if abs_tile_z == screen_base_z {
                    abs_tile_z = screen_base_z + 1;
                } else {
                    abs_tile_z = screen_base_z;
                }
            }
        }

        //TODO: initialize the hf_entities array!
        for i in 0..MAX_HIGH_ENTITIES {
            state.hf_entities[i] = Rc::new(RefCell::new(Default::default()));
        }

        let new_position = TilemapPosition{tile_x: screen_base_x*tiles_per_screen_x + 17/2, 
                                           tile_y: screen_base_y*tiles_per_screen_y + 9/2,
                                           tile_z: screen_base_z,
                                           offset: Default::default()};
        set_camera(state, &new_position);

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

            let entity = get_hf_entity(state, e_index).unwrap();
            {
                let mut hf_entity = entity.hf.borrow_mut();
                if controller.action_up.ended_down {
                    hf_entity.dz = 3.0;
                }
            }
            move_player(entity, acc, state, input.delta_t);
        } else {
            if controller.start.ended_down {
                let e_index = add_player(state);
                state.player_index_for_controller[c_index] = Some(e_index);
            }
        }
    }

    
    let tile_side_pixels = 60;
    let meters_to_pixel = tile_side_pixels as f32 / state.world.tilemap.tile_side_meters;

    //Adjust the camera to look at the right position
    if state.camera_follows_entity_index.is_some() {
        let mut new_cam_pos = state.camera_position;
        {
            let cam_entity_idx = state.camera_follows_entity_index.unwrap();
            let camera_entity = get_hf_entity(state, cam_entity_idx).unwrap();
            let hf = camera_entity.hf.borrow();
            new_cam_pos.tile_z = hf.tile_z;
            let tilemap = &state.world.tilemap;

            if hf.position.x > 9.0 * tilemap.tile_side_meters {
                new_cam_pos.tile_x += 17;
            } else if hf.position.x < -(9.0 * tilemap.tile_side_meters) {
                new_cam_pos.tile_x -= 17;
            }

            if hf.position.y > 5.0 * tilemap.tile_side_meters {
                new_cam_pos.tile_y += 9;
            } else if hf.position.y < -(5.0 * tilemap.tile_side_meters) {
                new_cam_pos.tile_y -= 9;
            }
        }
        set_camera(state, &new_cam_pos);
    }

    //Clear the screen to pink! And start rendering
    let buffer_dim = V2f{ x: video_buffer.width as f32, y: video_buffer.height as f32 };
    graphics::draw_rect(video_buffer, Default::default(), buffer_dim, 
                        1.0, 0.0, 1.0);

    graphics::draw_bitmap(video_buffer, &state.test_bitmap, 0.0, 0.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    for index in 0..state.hf_entity_count {
        let mut entity = state.hf_entities[index].borrow_mut();
        let lf = state.lf_entities[entity.lf_index].borrow();
        //let lf_part = state.lf_entities[index].borrow();
        //TODO: move out just temp code
        let acc = -9.81;
        entity.z += acc * 0.5 * input.delta_t.powi(2) 
                           + entity.dz * input.delta_t;
        entity.dz = acc * input.delta_t + entity.dz;
        if entity.z < 0.0 {
            entity.z = 0.0;
        }

        let entity_groundpoint = V2f{
            x: screen_center_x + meters_to_pixel * entity.position.x,
            y: screen_center_y - meters_to_pixel * entity.position.y,
        };
        let entity_airpoint = V2f {
            x: entity_groundpoint.x,
            y: entity_groundpoint.y - meters_to_pixel * entity.z,
        };

        let z_alpha =
            if entity.z > 1.0 {
                0.0
            } else {
                1.0 - entity.z * 0.8
            };
        let entity_dim = V2f{ x: meters_to_pixel * lf.dim.x, 
                              y: meters_to_pixel * lf.dim.y };
        let top_left = entity_groundpoint - entity_dim * 0.5;
        let bottom_right = top_left + entity_dim;

        match lf.etype {
            EntityType::Hero =>  {
                let hero_bitmaps = &state.hero_bitmaps[entity.face_direction];
                graphics::draw_bitmap_aligned_alpha(video_buffer, &state.shadow,
                                                    entity_groundpoint, hero_bitmaps.align_x,
                                                    hero_bitmaps.align_y, z_alpha);
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.torso,
                                              entity_airpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.cape,
                                              entity_airpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
                graphics::draw_bitmap_aligned(video_buffer, &hero_bitmaps.head,
                                              entity_airpoint, hero_bitmaps.align_x,
                                              hero_bitmaps.align_y);
            },
            _ => {
                graphics::draw_rect(video_buffer, top_left, bottom_right,
                                    1.0, 1.0, 1.0);
            },
        }
    }
}

// ======== End of the public interface =========

fn set_camera(state: &mut GameState, new_position: &TilemapPosition) {

    let TilemapDifference{ dx, dy, dz:_ } = 
        subtract(state.world.tilemap, new_position, &state.camera_position);
    state.camera_position = *new_position;
    let entity_offset_for_frame = V2f{ x: -dx, y: -dy };

    let tile_span_x = 17 * 3;
    let tile_span_y = 9 * 3;
    let tiles_in_work_set = V2f { x: tile_span_x as f32, y: tile_span_y as f32};
    let camera_in_bounds = Rectf::center_dim(Default::default(), 
                                             tiles_in_work_set * state.world.tilemap.tile_side_meters);

    offset_and_check_frequency_by_area(state, entity_offset_for_frame, camera_in_bounds);

    let min_tile_x = new_position.tile_x - tile_span_x;
    let max_tile_x = new_position.tile_x + tile_span_x;
    let min_tile_y = new_position.tile_y - tile_span_y;
    let max_tile_y = new_position.tile_y + tile_span_y;
    for index in 0..state.lf_entity_count {
        let (hf_index, tile_position) =  {
            let test_ent = get_lf_entity(state, index).unwrap();
            let lf = test_ent.borrow();
            (lf.hf_index, lf.tile_position)  
        };
        if hf_index.is_none() && 
           tile_position.tile_z == new_position.tile_z &&
           tile_position.tile_x >= min_tile_x &&
           tile_position.tile_x <= max_tile_x &&
           tile_position.tile_y >= min_tile_y &&
           tile_position.tile_y <= max_tile_y {

            make_high_frequency(state, index);
        }
    }
}

fn get_lf_entity(state: &GameState, index: usize) -> Option<Rc<RefCell<LfEntity>>> {
    if index < state.lf_entity_count {
        Some(state.lf_entities[index].clone())
    } else {
        None
    }
}

fn get_hf_entity(state: &mut GameState, index: usize) -> Option<Entity> {
    
    if index < state.lf_entity_count {
        make_high_frequency(state, index);
        let lf = state.lf_entities[index].clone();
        let hf = state.hf_entities[lf.borrow().hf_index.unwrap()].clone();
        Some(Entity {
                lf: lf,
                hf: hf,
            })
    } else {
        None
    }
}

fn add_lf_entity(state: &mut GameState, etype: EntityType) -> usize {
    let index = state.lf_entity_count;
    debug_assert!(index < state.lf_entities.len());

    state.lf_entity_count += 1;

    //TODO: move this from the Heap into our own memory region!
    state.lf_entities[index] = Rc::new(RefCell::new(Default::default()));
    let mut lf = state.lf_entities[index].borrow_mut();
    lf.etype = etype;

    index
}

fn add_wall(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32,
                  abs_tile_z: i32) -> usize {
    let e_index = add_lf_entity(state, EntityType::Wall);

    let lf = get_lf_entity(state, e_index).unwrap();
    {
        let mut lf_entity = lf.borrow_mut();

        lf_entity.dim = V2f{ x: state.world.tilemap.tile_side_meters, 
                               y: state.world.tilemap.tile_side_meters, };
        lf_entity.tile_position.tile_x = abs_tile_x;
        lf_entity.tile_position.tile_y = abs_tile_y;
        lf_entity.tile_position.tile_z = abs_tile_z;
        lf_entity.collides = true;
    }

    e_index
}

fn add_player(state: &mut GameState) -> usize {
    let e_index = add_lf_entity(state, EntityType::Hero);

    if state.camera_follows_entity_index.is_none() {
        state.camera_follows_entity_index = Some(e_index);
    }

    let lf = get_lf_entity(state, e_index).unwrap();
    {
        let mut lf_entity = lf.borrow_mut();

        lf_entity.dim = V2f{ x: 1.0, 
                               y: 0.5 };
        lf_entity.tile_position = state.camera_position;
        lf_entity.collides = true;
    }

    make_high_frequency(state, e_index);

    e_index
}

fn offset_and_check_frequency_by_area(state: &mut GameState, offset: V2f, 
                                      bounds: Rectf) {
    let mut to_remove = [None; MAX_HIGH_ENTITIES];
    for index in 0..state.hf_entity_count {
        let (lf_index, check_position) = {
            let mut hf = state.hf_entities[index].borrow_mut();
            hf.position = hf.position + offset;
            (hf.lf_index, hf.position)
        };

        if !bounds.p_inside(check_position) {
            to_remove[index] = Some(lf_index);
        }
    }

    //map the fuckers out
    for lf_index in to_remove.iter() {
        if lf_index.is_some() {
            make_low_frequency(state, lf_index.unwrap());
        }
    }
}

fn make_high_frequency(state: &mut GameState, lf_index: usize) {

    let mut lf = state.lf_entities[lf_index].borrow_mut();
    if lf.hf_index.is_none() {
        if state.hf_entity_count < state.hf_entities.len() {
            let hf_index = state.hf_entity_count;
            state.hf_entity_count += 1;

            let mut hf = state.hf_entities[hf_index].borrow_mut();
            let TilemapDifference{ dx, dy, dz:_ } =
                subtract(state.world.tilemap, &lf.tile_position, &state.camera_position); 
            hf.position = V2f{ x: dx, y: dy };
            hf.velocity = Default::default();
            hf.tile_z = lf.tile_position.tile_z;
            hf.face_direction = 0;
            hf.lf_index = lf_index;

            lf.hf_index = Some(hf_index);
        } else {
            debug_assert!(true, "Should not reach this");
        }
    }
}

fn make_low_frequency(state: &mut GameState, lf_index: usize) {
    let mut lf = state.lf_entities[lf_index].borrow_mut();
    if lf.hf_index.is_some() {
        let hf_index = lf.hf_index.unwrap();
        if hf_index != state.hf_entity_count - 1 {
            state.hf_entities[hf_index] = state.hf_entities[state.hf_entity_count - 1].clone();
            let old_last = state.hf_entities[hf_index].borrow();
            let mut lf_to_last = state.lf_entities[old_last.lf_index].borrow_mut();
            lf_to_last.hf_index = Some(hf_index);
        }

        lf.hf_index = None;
        state.hf_entity_count -= 1;
    }
}

fn move_player<'a>(entity: Entity, mut acc: V2f, 
                   state: &'a mut GameState<'a>, delta_t: f32) {

    //Diagonal correction.
    if acc.length_sq() > 1.0 {
        acc = acc.normalize();
    }

    let entity_speed = 15.0; // m/s^2

    acc = acc * entity_speed;

    let mut hf_entity = entity.hf.borrow_mut();
    //friction force currently just by rule of thumb;
    acc = acc - hf_entity.velocity * 13.0;


    //Copy old player Position before we handle input 
    let mut entity_delta = acc * 0.5 * delta_t.powi(2) 
                       + hf_entity.velocity * delta_t;
    hf_entity.velocity = acc * delta_t + hf_entity.velocity;


    //try the collission detection multiple times to see if we can move with
    //a corrected velocity after a collision
    //TODO: we always loop 4 times. Look for a fast out if we moved enough
    for _ in 0..4 {
        let mut t_min = 1.0;
        let mut wall_normal = Default::default();
        let mut hit_hf_e_index = None;
        
        let target_pos = hf_entity.position + entity_delta;

        let lf_entity = entity.lf.borrow();
        for e_index in 0..state.hf_entity_count {
            if e_index == lf_entity.hf_index.unwrap() {
                continue;
            }
            let hf_test_entity = state.hf_entities[e_index].borrow();
            let lf_test_entity = state.lf_entities[hf_test_entity.lf_index].borrow();
            if lf_test_entity.collides {
                //Minkowski Sum
                let diameter = V2f { x: lf_test_entity.dim.x + lf_entity.dim.x, 
                                     y: lf_test_entity.dim.y + lf_entity.dim.y};

                let min_corner = diameter * -0.5;
                let max_corner = diameter * 0.5;
                let rel = hf_entity.position - hf_test_entity.position;

                //check against the 4 entity walls
                if test_wall(max_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2f{ x: 1.0, y: 0.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(min_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2f{ x: -1.0, y: 0.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(max_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2f{ x: 0.0, y: 1.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(min_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2f{ x: 0.0, y: -1.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
            }
        }

        hf_entity.position = hf_entity.position + entity_delta * t_min;
        if hit_hf_e_index.is_some() {
            hf_entity.velocity = hf_entity.velocity 
                - wall_normal * math::dot(hf_entity.velocity, wall_normal);
            entity_delta = target_pos - hf_entity.position;
            entity_delta = entity_delta - wall_normal * math::dot(entity_delta, wall_normal);

            let hf_hit_entity = state.hf_entities[hit_hf_e_index.unwrap()].borrow();
            let lf_hit_entity = state.lf_entities[hf_hit_entity.lf_index].borrow();

            let tile_z = hf_entity.tile_z as i32  + lf_hit_entity.d_tile_z;
            hf_entity.tile_z = tile_z;
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
     let t_epsilon = 0.001;
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

#[derive(Copy)]
enum EntityType {
    None,
    Hero,
    Wall,
}

impl Default for EntityType {
    fn default() -> EntityType {
        EntityType::None
    }
}

struct Entity {
    lf: Rc<RefCell<LfEntity>>,
    hf: Rc<RefCell<HfEntity>>,
}

#[derive(Default, Copy)]
struct LfEntity {
    etype: EntityType,
    tile_position: TilemapPosition,
    dim: V2f,
    collides: bool,
    d_tile_z: i32,

    hf_index: Option<usize>,
}

#[derive(Default)]
struct HfEntity {
    position: V2f, //This position is relative to the camera
    velocity: V2f,
    tile_z: i32,
    face_direction: usize,

    z: f32,
    dz: f32,

    lf_index: usize,
}


const MAX_HIGH_ENTITIES: usize = 256;

struct GameState<'a> {
    world_arena: MemoryArena,
    world: &'a mut World<'a>,

    camera_follows_entity_index: Option<usize>,
    camera_position: TilemapPosition,

    player_index_for_controller: [Option<usize>; MAX_CONTROLLERS],

    lf_entity_count: usize,
    hf_entity_count: usize,
    lf_entities: [Rc<RefCell<LfEntity>>; 4096],
    hf_entities: [Rc<RefCell<HfEntity>>; MAX_HIGH_ENTITIES],

    test_bitmap: graphics::Bitmap<'a>,
    shadow: graphics::Bitmap<'a>,
    hero_bitmaps: [HeroBitmaps<'a>; 4],
} 

struct World<'a> {
    tilemap: &'a mut TileMap,
}

