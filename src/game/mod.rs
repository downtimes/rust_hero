use std::mem;
use std::default::Default;

use common::{GameMemory, SoundBuffer, VideoBuffer, Input};
use common::{ThreadContext, MAX_CONTROLLERS};

mod graphics;
mod world;
mod memory;
mod random;
mod math;

use self::world::{World, subtract, map_into_world_space};
use self::world::{WorldDifference, WorldPosition, world_pos_from_tile};
use self::memory::MemoryArena;
use self::math::{V2, Rect};

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
        state.background_bitmap = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test/test_background.bmp").unwrap();

        state.tree = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                              context, "test2/tree00.bmp").unwrap();

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

        state.world.initialize();

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

        for _ in 0..2000 {

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

        let new_position = world_pos_from_tile(state.world,
                                               screen_base_x*tiles_per_screen_x + 17/2, 
                                               screen_base_y*tiles_per_screen_y + 9/2,
                                               screen_base_z);
        set_camera(state, &new_position);

        game_memory.initialized = true;
    }

    for (c_index, controller) in input.controllers.iter().enumerate() {

        if let Some(e_index) = state.player_index_for_controller[c_index] {
            //in m/s^2
            let mut acc = V2{ x: 0.0, y: 0.0 };

            //Analog movement
            if controller.is_analog() {
                let avg_x = controller.average_x.unwrap_or_default();
                let avg_y = controller.average_y.unwrap_or_default();
                acc = V2{ x: avg_x, y: avg_y };

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
                let hf_entity = entity.get_hf();
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
    let meters_to_pixel = tile_side_pixels as f32 / state.world.tile_side_meters;

    //Adjust the camera to look at the right position
    if state.camera_follows_entity_index.is_some() {
        let new_cam_pos; 
        {
            let entity_idx = state.camera_follows_entity_index.unwrap();
            new_cam_pos = get_lf_entity(state, entity_idx).unwrap().world_position;
        }
        set_camera(state, &new_cam_pos);
    }

    //Clear the screen to grey And start rendering
    let buffer_dim = V2{ x: video_buffer.width as f32, y: video_buffer.height as f32 };
    graphics::draw_rect(video_buffer, Default::default(), buffer_dim, 
                        0.5, 0.5, 0.5);

//    graphics::draw_bitmap(video_buffer, &state.background_bitmap, 0.0, 0.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;

    for index in 0..state.hf_entity_count as usize {
        let entity = &mut state.hf_entities[index];
        let lf = &state.lf_entities[entity.lf_index as usize];

        //TODO: move out just temp code
        let acc = -9.81;
        entity.z += acc * 0.5 * input.delta_t.powi(2) 
                           + entity.dz * input.delta_t;
        entity.dz = acc * input.delta_t + entity.dz;
        if entity.z < 0.0 {
            entity.z = 0.0;
        }

        let entity_groundpoint = V2{
            x: screen_center_x + meters_to_pixel * entity.position.x,
            y: screen_center_y - meters_to_pixel * entity.position.y,
        };
        let entity_airpoint = V2{
            x: entity_groundpoint.x,
            y: entity_groundpoint.y - meters_to_pixel * entity.z,
        };

        let z_alpha =
            if entity.z > 1.0 {
                0.0
            } else {
                1.0 - entity.z * 0.8
            };
        let entity_dim = V2{ x: meters_to_pixel * lf.dim.x, 
                              y: meters_to_pixel * lf.dim.y };
        let top_left = entity_groundpoint - entity_dim * 0.5;
        let bottom_right = top_left + entity_dim;

        match lf.etype {
            EntityType::Hero =>  {
                let hero_bitmaps = &state.hero_bitmaps[entity.face_direction as usize];
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
//                graphics::draw_rect(video_buffer, top_left, bottom_right,
//                                    1.0, 1.0, 0.0);
                graphics::draw_bitmap_aligned(video_buffer, &state.tree,
                                              entity_airpoint, 40,
                                              80);
            },
        }
    }
}

// ======== End of the public interface =========

fn set_camera(state: &mut GameState, new_position: &WorldPosition) {

    let WorldDifference{ dx, dy, .. } = 
        subtract(state.world, new_position, &state.camera_position);
    state.camera_position = *new_position;
    let entity_offset_for_frame = V2{ x: -dx, y: -dy };

    let tile_span_x = 17 * 3;
    let tile_span_y = 9 * 3;
    let tiles_in_work_set = V2{ x: tile_span_x as f32, y: tile_span_y as f32};
    let high_frequency_bounds = Rect::center_dim(Default::default(), 
                                tiles_in_work_set * state.world.tile_side_meters);

    offset_and_check_frequency_by_area(state, entity_offset_for_frame, high_frequency_bounds);

    //TODO: Needs to be spatialy now!
    let min_p = map_into_world_space(state.world, new_position, &high_frequency_bounds.get_min()); 
    let max_p = map_into_world_space(state.world, new_position, &high_frequency_bounds.get_max());
    for ch in state.world.iter_spatially(min_p, max_p, new_position.chunk_z) {
        for block in ch.first_block.iter() {
            for block_idx in 0..block.e_count as usize {
                let lf_index = block.lf_entities[block_idx];
                let hf_index = state.lf_entities[lf_index as usize].hf_index;
                if hf_index.is_none() {
                    let cameraspace_p = get_camspace_p(state, lf_index);
                    if high_frequency_bounds.p_inside(cameraspace_p) {
                        make_high_frequency_pos(state, lf_index, cameraspace_p);
                    }
                }
            }
        }
    }
}

fn get_lf_entity<'a>(state: &'a mut GameState, index: u32) -> Option<&'a mut LfEntity> {
    if index < state.lf_entity_count {
        Some(&mut state.lf_entities[index as usize])
    } else {
        None
    }
}

fn get_hf_entity(state: &mut GameState, index: u32) -> Option<Entity> {
    
    if index < state.lf_entity_count {
        make_high_frequency(state, index);
        let lf = &mut state.lf_entities[index as usize];
        let hf = &mut state.hf_entities[lf.hf_index.unwrap() as usize];
        Some(Entity {
                lf_index: index,
                lf: lf as *mut LfEntity,
                hf: hf as *mut HfEntity,
            })
    } else {
        None
    }
}

fn add_lf_entity(state: &mut GameState, etype: EntityType, pos: WorldPosition) -> u32 {
    let index = state.lf_entity_count;
    debug_assert!((index as usize) < state.lf_entities.len());

    state.lf_entity_count += 1;

    state.lf_entities[index as usize] = LfEntity {
        etype: etype,
        world_position: pos,
        dim: Default::default(),
        collides: false,

        hf_index: None,
    };

    {
        let &mut GameState{ ref mut world_arena, ref mut world, .. } = state;
        world.change_entity_location(index, None, &pos, world_arena);
    }

    index
}

fn add_wall(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32,
                  abs_tile_z: i32) -> u32 {

    let pos = world_pos_from_tile(state.world, abs_tile_x, 
                                  abs_tile_y, abs_tile_z);
    let e_index = add_lf_entity(state, EntityType::Wall, pos);

    let tile_side_meters = state.world.tile_side_meters;
    
    let lf_entity = get_lf_entity(state, e_index).unwrap();
    lf_entity.dim = V2{ x: tile_side_meters, 
                        y: tile_side_meters, };
    lf_entity.collides = true;

    e_index
}

fn add_player(state: &mut GameState) -> u32 {
    let pos = state.camera_position;
    
    let e_index = add_lf_entity(state, EntityType::Hero, pos);

    if state.camera_follows_entity_index.is_none() {
        state.camera_follows_entity_index = Some(e_index);
    }

    {
        let lf_entity = get_lf_entity(state, e_index).unwrap();
        lf_entity.dim = V2{ x: 1.0, 
                            y: 0.5 };
        lf_entity.collides = true;
    }

    make_high_frequency(state, e_index);

    e_index
}

fn offset_and_check_frequency_by_area(state: &mut GameState, offset: V2<f32>, 
                                      bounds: Rect<f32>) {
    let mut to_remove = [None; MAX_HIGH_ENTITIES];
    for index in 0..state.hf_entity_count as usize {
        let (lf_index, check_position) = {
            let hf = &mut state.hf_entities[index];
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

fn get_camspace_p(state: &GameState, lf_index: u32) -> V2<f32> {
    let lf = &state.lf_entities[lf_index as usize];
    let WorldDifference{ dx, dy, .. } =
        subtract(state.world, &lf.world_position, &state.camera_position); 
    V2{ x: dx, y: dy }
}

fn make_high_frequency_pos(state: &mut GameState, lf_index: u32, camspace_p: V2<f32>) {
    let lf = &mut state.lf_entities[lf_index as usize];
    if lf.hf_index.is_none() {
        if (state.hf_entity_count as usize) < state.hf_entities.len() {
            let hf_index = state.hf_entity_count as usize;
            state.hf_entity_count += 1;

            let hf = &mut state.hf_entities[hf_index];
            hf.position = camspace_p;
            hf.velocity = Default::default();
            hf.chunk_z = lf.world_position.chunk_z;
            hf.face_direction = 0;
            hf.lf_index = lf_index;

            lf.hf_index = Some(hf_index as u32);
        } else {
            debug_assert!(true, "Should not reach this");
        }
    }
}

fn make_high_frequency(state: &mut GameState, lf_index: u32) {
    let camspace_p = get_camspace_p(state, lf_index);
    make_high_frequency_pos(state, lf_index, camspace_p);
}

fn make_low_frequency(state: &mut GameState, lf_index: u32) {
    let hf_index = state.lf_entities[lf_index as usize].hf_index;
    if hf_index.is_some() {
        let index = hf_index.unwrap();
        if index != state.hf_entity_count - 1 {
            state.hf_entities[index as usize] = state.hf_entities[state.hf_entity_count as usize - 1];
            let old_last_lf_index = state.hf_entities[index as usize].lf_index;
            let lf_to_old_last = get_lf_entity(state, old_last_lf_index).unwrap();
            lf_to_old_last.hf_index = hf_index;
        }
        state.lf_entities[lf_index as usize].hf_index = None;
        state.hf_entity_count -= 1;
    }
}

fn move_player(entity: Entity, mut acc: V2<f32>, 
                   state: &mut GameState, delta_t: f32) {

    //Diagonal correction.
    if acc.length_sq() > 1.0 {
        acc = acc.normalize();
    }

    let entity_speed = 15.0; // m/s^2

    acc = acc * entity_speed;

    let hf_entity = entity.get_hf();
    //friction force currently just by rule of thumb;
    acc = acc - hf_entity.velocity * 10.0;


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

        let lf_entity = entity.get_lf();
        for e_index in 0..state.hf_entity_count as usize {
            if e_index == lf_entity.hf_index.unwrap() as usize {
                continue;
            }
            let hf_test_entity = &state.hf_entities[e_index];
            let lf_test_entity = &state.lf_entities[hf_test_entity.lf_index as usize];
            if lf_test_entity.collides {
                //Minkowski Sum
                let diameter = V2{ x: lf_test_entity.dim.x + lf_entity.dim.x, 
                                   y: lf_test_entity.dim.y + lf_entity.dim.y};

                let min_corner = diameter * -0.5;
                let max_corner = diameter * 0.5;
                let rel = hf_entity.position - hf_test_entity.position;

                //check against the 4 entity walls
                if test_wall(max_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2{ x: 1.0, y: 0.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(min_corner.x, min_corner.y, max_corner.y,
                             rel.x, rel.y, entity_delta.x, 
                             entity_delta.y, &mut t_min) {
                    wall_normal = V2{ x: -1.0, y: 0.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(max_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2{ x: 0.0, y: 1.0 };
                    hit_hf_e_index = lf_test_entity.hf_index;
                }
                if test_wall(min_corner.y, min_corner.x, max_corner.x,
                             rel.y, rel.x, entity_delta.y,
                             entity_delta.x, &mut t_min) {
                    wall_normal = V2{ x: 0.0, y: -1.0 };
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

    //map high_entity back to the low entity
    let lf_entity = entity.get_lf();
    let new_pos = map_into_world_space(&state.world, 
                                       &state.camera_position, 
                                       &hf_entity.position);
    {
        let &mut GameState{ ref mut world_arena, ref mut world, .. } = state;
        world.change_entity_location(entity.lf_index, Some(&lf_entity.world_position),
                                     &new_pos, world_arena);
    }
    lf_entity.world_position = new_pos;
    
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

#[derive(Copy, Clone)]
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
    lf_index: u32,
    lf: *mut LfEntity,
    hf: *mut HfEntity,
}

impl Entity {
    fn get_hf(&self) -> &mut HfEntity {
        unsafe { mem::transmute(self.hf) }
    }

    fn get_lf(&self) -> &mut LfEntity {
        unsafe { mem::transmute(self.lf) }
    }
}

#[derive(Default, Copy, Clone)]
struct LfEntity {
    etype: EntityType,
    world_position: WorldPosition,
    dim: V2<f32>,
    collides: bool,

    hf_index: Option<u32>,
}

#[derive(Default, Copy, Clone)]
struct HfEntity {
    position: V2<f32>, //This position is relative to the camera
    velocity: V2<f32>,
    face_direction: u32,
    chunk_z: i32,

    z: f32,
    dz: f32,

    lf_index: u32,
}


const MAX_HIGH_ENTITIES: usize = 256;

struct GameState<'a> {
    world_arena: MemoryArena,
    world: &'a mut World,

    camera_follows_entity_index: Option<u32>,
    camera_position: WorldPosition,

    player_index_for_controller: [Option<u32>; MAX_CONTROLLERS],

    lf_entity_count: u32,
    hf_entity_count: u32,
    lf_entities: [LfEntity; 100000],
    hf_entities: [HfEntity; MAX_HIGH_ENTITIES],

    background_bitmap: graphics::Bitmap<'a>,
    shadow: graphics::Bitmap<'a>,
    tree: graphics::Bitmap<'a>,
    hero_bitmaps: [HeroBitmaps<'a>; 4],
} 

