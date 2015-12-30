use std::mem;
use std::ptr;
use num::traits::Float;
use std::default::Default;
use std::f32::consts::PI;

use common::{GameMemory, SoundBuffer, VideoBuffer, Input};
use common::{ThreadContext, MAX_CONTROLLERS};

mod graphics;
mod world;
mod memory;
mod random;
mod math;
mod simulation;

use self::world::{World, subtract, map_into_world_space};
use self::world::{WorldPosition, world_pos_from_tile};
use self::memory::MemoryArena;
use self::graphics::Color;
use self::math::{V2, V3, Rect};

macro_rules! make_array {
    ( $val:expr, $n:expr ) => {{
        let mut arr: [_; $n] = unsafe { mem::uninitialized() };
        for place in arr.iter_mut() {
            unsafe { ptr::write(place, $val); }
        }

        arr
    }}
}

// ============= The public interface ===============
// Has to be very low latency!
#[no_mangle]
pub extern "C" fn get_sound_samples(_context: &ThreadContext,
                                    game_memory: &mut GameMemory,
                                    _sound_buffer: &mut SoundBuffer) {

    let _state: &mut GameState = unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };
}

#[no_mangle]
pub extern "C" fn update_and_render(context: &ThreadContext,
                                    game_memory: &mut GameMemory,
                                    input: &Input,
                                    video_buffer: &mut VideoBuffer) {

    debug_assert!(mem::size_of::<GameState>() <= game_memory.permanent.len());

    let state: &mut GameState = unsafe { mem::transmute(game_memory.permanent.as_mut_ptr()) };

    // random table index 6 start to get a room with staircase on the first
    // screen
    let mut rand_index = 6;

    if !game_memory.initialized {
        state.background_bitmap =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_background.bmp")
                .unwrap();

        state.tree = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                                 context,
                                                 "test2/tree00.bmp")
                         .unwrap();

        state.shadow = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                                   context,
                                                   "test/test_hero_shadow.bmp")
                           .unwrap();

        state.sword = graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                                  context,
                                                  "test2/rock03.bmp")
                          .unwrap();
        state.hero_bitmaps[0].head =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_right_head.bmp")
                .unwrap();
        state.hero_bitmaps[0].torso =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_right_torso.bmp")
                .unwrap();
        state.hero_bitmaps[0].cape =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_right_cape.bmp")
                .unwrap();
        state.hero_bitmaps[0].align = V2 { x: 72, y: 182 };

        state.hero_bitmaps[1].head =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_back_head.bmp")
                .unwrap();
        state.hero_bitmaps[1].torso =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_back_torso.bmp")
                .unwrap();
        state.hero_bitmaps[1].cape =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_back_cape.bmp")
                .unwrap();
        state.hero_bitmaps[1].align = V2 { x: 72, y: 182 };

        state.hero_bitmaps[2].head =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_left_head.bmp")
                .unwrap();
        state.hero_bitmaps[2].torso =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_left_torso.bmp")
                .unwrap();
        state.hero_bitmaps[2].cape =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_left_cape.bmp")
                .unwrap();
        state.hero_bitmaps[2].align = V2 { x: 72, y: 182 };

        state.hero_bitmaps[3].head =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_front_head.bmp")
                .unwrap();
        state.hero_bitmaps[3].torso =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_front_torso.bmp")
                .unwrap();
        state.hero_bitmaps[3].cape =
            graphics::debug_load_bitmap(game_memory.platform_read_entire_file,
                                        context,
                                        "test/test_hero_front_cape.bmp")
                .unwrap();
        state.hero_bitmaps[3].align = V2 { x: 72, y: 182 };


        let game_state_size = mem::size_of::<GameState>();
        state.world_arena = MemoryArena::new(game_memory.permanent.len() - game_state_size,
                                             unsafe {
                                                 game_memory.permanent
                                                            .as_ptr()
                                                            .offset(game_state_size as isize)
                                             });

        state.world = state.world_arena.push_struct();

        state.world.initialize();

        let tile_side_pixels = 60;
        state.meters_to_pixel = tile_side_pixels as f32 / state.world.tile_side_meters;

        // Generating a random maze
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
            } else if random_choice == 1 {
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
                    // vertical walls
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
                    // horizontal walls
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

                    // "Staircases"
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

        let cam_tile_x = screen_base_x * tiles_per_screen_x + 17 / 2;
        let cam_tile_y = screen_base_y * tiles_per_screen_y + 9 / 2;
        let cam_tile_z = screen_base_z;
        let cam_position = world_pos_from_tile(state.world, cam_tile_x, cam_tile_y, cam_tile_z);
        // drop a test monster & familiar
        add_monster(state, cam_tile_x + 2, cam_tile_y + 2, cam_tile_z);
        add_familiar(state, cam_tile_x - 2, cam_tile_y + 2, cam_tile_z);

        set_camera(state, &cam_position);

        game_memory.initialized = true;
    }

    for (c_index, controller) in input.controllers.iter().enumerate() {

        if let Some(e_index) = state.player_index_for_controller[c_index] {
            // in m/s^2
            let mut acc = V2 { x: 0.0, y: 0.0 };

            // Analog movement
            if controller.is_analog() {
                let avg_x = controller.average_x.unwrap_or_default();
                let avg_y = controller.average_y.unwrap_or_default();
                acc = V2 {
                    x: avg_x,
                    y: avg_y,
                };

                // Digital movement
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

            let entity = force_hf_entity(state, e_index).unwrap();
            {
                let hf_entity = entity.get_hf();
                if controller.start.ended_down {
                    hf_entity.dz = 3.0;
                }

            }
            {
                let &mut GameState{ ref mut lf_entities, ref mut hf_entities,
                                    ref mut world_arena, hf_entity_count,
                                    ref mut world, ref camera_position, .. } = state;
                let move_spec = MoveSpec {
                    unit_max_accel_vector: true,
                    drag: 8.0,
                    speed: 50.0,
                };

                move_entity(&mut lf_entities[..],
                            &mut hf_entities[..],
                            hf_entity_count,
                            world,
                            world_arena,
                            camera_position,
                            entity,
                            acc,
                            &move_spec,
                            input.delta_t);
            }

            let mut d_sword = V2 { x: 0.0, y: 0.0 };
            if controller.action_up.ended_down {
                d_sword = V2 { x: 0.0, y: 1.0 };
            }
            if controller.action_down.ended_down {
                d_sword = V2 { x: 0.0, y: -1.0 };
            }
            if controller.action_left.ended_down {
                d_sword = V2 { x: -1.0, y: 0.0 };
            }
            if controller.action_right.ended_down {
                d_sword = V2 { x: 1.0, y: 0.0 };
            }
            if (d_sword.x != 0.0) || (d_sword.y != 0.0) {
                let s_index = state.lf_entities[e_index].sword_lf_index;
                if s_index.is_some() &&
                   state.lf_entities[s_index.unwrap()].world_position.is_none() {
                    {
                        let &mut GameState{ ref mut lf_entities, ref mut world_arena,
                                        ref mut world, .. } = state;
                        let pos = lf_entities[e_index].world_position.unwrap();
                        let sword_lf = &mut lf_entities[s_index.unwrap()];
                        world.change_entity_location(s_index.unwrap(),
                                                     sword_lf,
                                                     None,
                                                     Some(pos),
                                                     world_arena);
                        sword_lf.distance_remaining = 5.0;
                    }
                    let sword = force_hf_entity(state, s_index.unwrap()).unwrap();
                    sword.get_hf().velocity = d_sword * 5.0;
                }
            }
        } else {
            if controller.start.ended_down {
                let e_index = add_player(state);
                state.player_index_for_controller[c_index] = Some(e_index);
            }
        }
    }

    let meters_to_pixel = state.meters_to_pixel;

    // Simulate stuff around the camera
    let tile_span_x = 17 * 3;
    let tile_span_y = 9 * 3;
    let tiles_in_work_set = V2 {
        x: tile_span_x as f32,
        y: tile_span_y as f32,
    };
    let camera_bounds = Rect::center_dim(Default::default(),
                                         tiles_in_work_set * state.world.tile_side_meters);

    let sim_region = SimRegion::begin_sim(sim_arena,
                                          state.world,
                                          state.camera_position,
                                          camera_bounds);

    // Clear the screen to grey and start rendering
    let buffer_dim = V2 {
        x: video_buffer.width as f32,
        y: video_buffer.height as f32,
    };
    graphics::draw_rect(video_buffer, Default::default(), buffer_dim, 0.5, 0.5, 0.5);

    // graphics::draw_bitmap(video_buffer, &state.background_bitmap, 0.0, 0.0);

    let screen_center_x = 0.5 * video_buffer.width as f32;
    let screen_center_y = 0.5 * video_buffer.height as f32;


    for index in 0..sim_region.entity_count {
        let &mut GameState{ref shadow, ref tree, ref sword, ref hero_bitmaps,
                           ref mut hf_entities, ref mut lf_entities,
                           hf_entity_count, ref mut world_arena, ref mut world,
                           ref camera_position, ..} = state;
        let sim_entity = &mut sim_region.entities[index];
        let lf = &mut lf_entities[sim_entity.storage_index];


        // TODO: is the alpha from the previous frame needs to be after
        // updates
        let z_alpha = if hf.z > 1.0 {
            0.0
        } else {
            1.0 - hf.z * 0.8
        };


        let hero_bitmaps = &hero_bitmaps[lf.face_direction as usize];
        let mut piece_group = EntityPieceGroup {
            meters_to_pixel: meters_to_pixel,
            count: 0,
            pieces: make_array!(None, 32),
        };

        match lf.etype {
            EntityType::Hero => {
                piece_group.push_bitmap(shadow,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(&hero_bitmaps.torso,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                piece_group.push_bitmap(&hero_bitmaps.cape,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                piece_group.push_bitmap(&hero_bitmaps.head,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                draw_hitpoints(lf, &mut piece_group);

            }

            EntityType::Sword => {
                update_sword(&mut lf_entities[..],
                             &mut hf_entities[..],
                             hf_entity_count,
                             world,
                             world_arena,
                             camera_position,
                             entity,
                             input.delta_t);
                piece_group.push_bitmap(shadow,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(sword,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        V2 { x: 29, y: 13 },
                                        0.0,
                                        1.0);
            }

            EntityType::Monster => {
                update_monster(&lf_entities[..],
                               &hf_entities[..],
                               hf_entity_count,
                               entity,
                               input.delta_t);
                piece_group.push_bitmap(shadow,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(&hero_bitmaps.torso,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                draw_hitpoints(lf, &mut piece_group);
            }

            EntityType::Wall => {
                piece_group.push_bitmap(tree,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        V2 { x: 40, y: 80 },
                                        1.0,
                                        1.0);
            }

            EntityType::Familiar => {
                update_familiar(&mut lf_entities[..],
                                &mut hf_entities[..],
                                hf_entity_count,
                                world,
                                world_arena,
                                camera_position,
                                entity,
                                input.delta_t);
                hf.tbob += input.delta_t;
                if hf.tbob > 2.0 * PI {
                    hf.tbob -= 2.0 * PI;
                }
                let bob_sign = (hf.tbob * 2.0).sin();
                piece_group.push_bitmap(&shadow,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        (0.5 * z_alpha) + 0.2 * bob_sign);
                piece_group.push_bitmap(&hero_bitmaps.head,
                                        V2 { x: 0.0, y: 0.0 },
                                        0.25 * bob_sign,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
            }
            _ => {
                debug_assert!(true, "We forgot an important case!");
            }
        }

        let gravity = -9.81;
        sim_entity.z += gravity * 0.5 * input.delta_t.powi(2) + sim_entity.dz * input.delta_t;
        sim_entity.dz = gravity * input.delta_t + sim_entity.dz;
        if sim_entity.z < 0.0 {
            sim_entity.z = 0.0;
        }

        let entity_groundpoint = V2 {
            x: screen_center_x + meters_to_pixel * sim_entity.position.x,
            y: screen_center_y - meters_to_pixel * sim_entity.position.y,
        };

        for index in 0..piece_group.count {
            let piece = piece_group.pieces[index].as_ref().unwrap();
            let piece_point = V2 {
                x: entity_groundpoint.x + piece.offset.x,
                y: entity_groundpoint.y + piece.offset.y + piece.offset_z -
                   (meters_to_pixel * hf.z) * piece.entity_zc,
            };

            if piece.bitmap.is_some() {
                graphics::draw_bitmap_alpha(video_buffer,
                                            piece.bitmap.unwrap(),
                                            piece_point,
                                            piece.alpha);
            } else {
                let half_dim = piece.dim * meters_to_pixel * 0.5;
                graphics::draw_rect(video_buffer,
                                    piece_point - half_dim,
                                    piece_point + half_dim,
                                    piece.r,
                                    piece.g,
                                    piece.b);
            }
        }
    }

    sim_region.end_sim(state);
}

// ======== End of the public interface =========

fn draw_hitpoints<'a>(lf: &LfEntity, piece_group: &mut EntityPieceGroup<'a>) {
    if lf.max_hitpoints >= 1 {
        let health_dim = V2 { x: 0.2, y: 0.2 };
        let spacing_x = health_dim.x * 1.5;
        let first_x = (lf.max_hitpoints - 1) as f32 * 0.5 * spacing_x;
        let mut hit_p = V2 {
            x: -first_x,
            y: -0.25,
        };
        let d_hit_p = V2 {
            x: spacing_x,
            y: 0.0,
        };
        for idx in 0..lf.max_hitpoints as usize {
            let hp = &lf.hitpoints[idx];
            let color = if hp.filled != 0 {
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                }
            };
            piece_group.push_rect(hit_p, 0.0, 0.0, health_dim, color);
            hit_p = hit_p + d_hit_p;
        }
    }
}


fn sim_camera_region(state: &mut GameState) {}

pub fn get_lf_entity<'a>(state: &'a mut GameState, index: usize) -> Option<&'a mut LfEntity> {
    if index < state.lf_entity_count {
        Some(&mut state.lf_entities[index])
    } else {
        None
    }
}

fn entity_from_hf(lf_entities: &[LfEntity],
                  hf_entities: &[HfEntity],
                  hf_entity_count: usize,
                  hf_index: usize)
                  -> Option<Entity> {
    if hf_index < hf_entity_count {
        let hf = &hf_entities[hf_index];
        let lf = &lf_entities[hf.lf_index];
        Some(Entity {
            lf_index: hf.lf_index,
            lf: lf as *const LfEntity,
            hf: hf as *const HfEntity,
        })
    } else {
        None
    }
}

fn add_lf_entity<'a>(state: &'a mut GameState,
                     etype: EntityType,
                     pos: Option<WorldPosition>)
                     -> (usize, &'a mut LfEntity) {
    let index = state.lf_entity_count;
    debug_assert!((index) < state.lf_entities.len());

    state.lf_entity_count += 1;

    state.lf_entities[index] = LfEntity {
        etype: etype,
        world_position: pos,
        dim: V2::default(),
        collides: false,

        hf_index: None,

        max_hitpoints: 0,
        hitpoints: [Hitpoint::default(); HITPOINTS_ARRAY_MAX],
        distance_remaining: 0.0,
        sword_lf_index: None,
    };

    if pos.is_some() {
        let &mut GameState{ ref mut world_arena, ref mut world, .. } = state;
        world.change_entity_location_raw(index, None, &pos.unwrap(), world_arena);
    }

    (index, &mut state.lf_entities[index])
}

fn add_wall(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32, abs_tile_z: i32) -> usize {

    let pos = world_pos_from_tile(state.world, abs_tile_x, abs_tile_y, abs_tile_z);
    let tile_side_meters = state.world.tile_side_meters;

    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Wall, Some(pos));

    lf_entity.dim = V2 {
        x: tile_side_meters,
        y: tile_side_meters,
    };
    lf_entity.collides = true;

    e_index
}

fn add_monster(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32, abs_tile_z: i32) -> usize {
    let pos = world_pos_from_tile(state.world, abs_tile_x, abs_tile_y, abs_tile_z);

    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Monster, Some(pos));

    lf_entity.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.collides = true;
    init_hit_points(3, lf_entity);

    e_index
}

fn add_sword(state: &mut GameState) -> usize {
    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Sword, None);

    lf_entity.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.collides = false;

    e_index
}

fn add_familiar(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32, abs_tile_z: i32) -> usize {
    let pos = world_pos_from_tile(state.world, abs_tile_x, abs_tile_y, abs_tile_z);

    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Familiar, Some(pos));

    lf_entity.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.collides = false;

    e_index
}

fn init_hit_points(hit_point_count: u8, lf_entity: &mut LfEntity) {
    debug_assert!(hit_point_count <= HITPOINTS_ARRAY_MAX);
    lf_entity.max_hitpoints = hit_point_count as u32;
    for idx in 0..lf_entity.max_hitpoints as usize {
        lf_entity.hitpoints[idx].filled = HITPOINT_SUB_COUNT;
    }
}

fn add_player(state: &mut GameState) -> usize {

    let e_index = {
        let pos = state.camera_position;
        let (e_index, lf_entity) = add_lf_entity(state, EntityType::Hero, Some(pos));

        lf_entity.dim = V2 { x: 1.0, y: 0.5 };
        lf_entity.collides = true;
        init_hit_points(3, lf_entity);

        e_index
    };

    let s_index = Some(add_sword(state));
    state.lf_entities[e_index].sword_lf_index = s_index;

    if state.camera_follows_entity_index.is_none() {
        state.camera_follows_entity_index = Some(e_index);
    }

    e_index
}

fn get_camspace_p(world: &World,
                  lf_entities: &[LfEntity],
                  lf_index: usize,
                  camera_position: &WorldPosition)
                  -> Option<V2<f32>> {
    let lf = &lf_entities[lf_index];
    if lf.world_position.is_some() {
        let V3{ x, y, .. } = subtract(world, &lf.world_position.unwrap(), camera_position);
        Some(V2 { x: x, y: y })
    } else {
        None
    }
}

fn update_sword(lf_entities: &mut [LfEntity],
                hf_entities: &mut [HfEntity],
                hf_entity_count: usize,
                world: &mut World,
                world_arena: &mut MemoryArena,
                camera_position: &WorldPosition,
                entity: EntityMut,
                dt: f32) {

    let move_spec = MoveSpec {
        unit_max_accel_vector: false,
        speed: 0.0,
        drag: 0.0,
    };
    let old_p = entity.get_hf().position;
    move_entity(lf_entities,
                hf_entities,
                hf_entity_count,
                world,
                world_arena,
                camera_position,
                entity,
                V2 { x: 0.0, y: 0.0 },
                &move_spec,
                dt);
    let travelled = (entity.get_hf().position - old_p).length();
    entity.get_lf().distance_remaining -= travelled;

    if entity.get_lf().distance_remaining < 0.0 {
        world.change_entity_location(entity.lf_index,
                                     entity.get_lf(),
                                     entity.get_lf().world_position,
                                     None,
                                     world_arena);
    }
}

fn update_familiar(lf_entities: &mut [LfEntity],
                   hf_entities: &mut [HfEntity],
                   hf_entity_count: usize,
                   world: &mut World,
                   world_arena: &mut MemoryArena,
                   camera_position: &WorldPosition,
                   entity: EntityMut,
                   dt: f32) {
    let mut closest_hero_d_sq = 10.0.powi(2); //Maximum search range
    let mut closest_hero = None;
    for hf_idx in 0..hf_entity_count {
        let test_entity = entity_from_hf(lf_entities, hf_entities, hf_entity_count, hf_idx)
                              .unwrap();
        // if we try to test ourselves, skip
        if test_entity.lf_index == entity.lf_index {
            continue;
        }

        match test_entity.get_lf().etype {
            EntityType::Hero => {
                let test_d_sq = (test_entity.get_hf().position - entity.get_hf().position)
                                    .length_sq();
                if closest_hero_d_sq > test_d_sq {
                    closest_hero_d_sq = test_d_sq;
                    closest_hero = Some(test_entity);
                }
            }
            _ => {}
        }
    }

    let mut acc = V2 { x: 0.0, y: 0.0 };
    if let Some(hero) = closest_hero {
        if closest_hero_d_sq > 3.0.powi(2) {
            let speed = 0.5;
            let one_over_length = 1.0 / closest_hero_d_sq.sqrt();
            acc = (hero.get_hf().position - entity.get_hf().position) * one_over_length * speed;
        }
    }

    let move_spec = MoveSpec {
        unit_max_accel_vector: true,
        drag: 8.0,
        speed: 50.0,
    };
    move_entity(lf_entities,
                hf_entities,
                hf_entity_count,
                world,
                world_arena,
                camera_position,
                entity,
                acc,
                &move_spec,
                dt);
}

#[allow(unused_variables)]
fn update_monster(lf_entities: &[LfEntity],
                  hf_entities: &[HfEntity],
                  hf_entity_count: usize,
                  entity: EntityMut,
                  dt: f32) {
}

fn move_entity(lf_entities: &mut [LfEntity],
               hf_entities: &mut [HfEntity],
               hf_entity_count: usize,
               world: &mut World,
               world_arena: &mut MemoryArena,
               camera_position: &WorldPosition,
               entity: EntityMut,
               mut acc: V2<f32>,
               move_spec: &MoveSpec,
               delta_t: f32) {

    // Diagonal correction.
    if move_spec.unit_max_accel_vector {
        if acc.length_sq() > 1.0 {
            acc = acc.normalize();
        }
    }

    acc = acc * move_spec.speed;

    let hf_entity = entity.get_hf();
    // friction force currently just by rule of thumb;
    acc = acc - hf_entity.velocity * move_spec.drag;


    // Copy old player Position before we handle input
    let mut entity_delta = acc * 0.5 * delta_t.powi(2) + hf_entity.velocity * delta_t;
    hf_entity.velocity = acc * delta_t + hf_entity.velocity;


    // try the collission detection multiple times to see if we can move with
    // a corrected velocity after a collision
    // TODO: we always loop 4 times. Look for a fast out if we moved enough
    for _ in 0..4 {
        let mut t_min = 1.0;
        let mut wall_normal = Default::default();
        let mut hit_hf_e_index = None;

        let target_pos = hf_entity.position + entity_delta;

        let lf_entity = entity.get_lf();

        if lf_entity.collides {
            for e_index in 0..hf_entity_count {
                if e_index == lf_entity.hf_index.unwrap() {
                    continue;
                }
                let hf_test_entity = &hf_entities[e_index];
                let lf_test_entity = &lf_entities[hf_test_entity.lf_index];
                if lf_test_entity.collides {
                    // Minkowski Sum
                    let diameter = V2 {
                        x: lf_test_entity.dim.x + lf_entity.dim.x,
                        y: lf_test_entity.dim.y + lf_entity.dim.y,
                    };

                    let min_corner = diameter * -0.5;
                    let max_corner = diameter * 0.5;
                    let rel = hf_entity.position - hf_test_entity.position;

                    // check against the 4 entity walls
                    if test_wall(max_corner.x,
                                 min_corner.y,
                                 max_corner.y,
                                 rel.x,
                                 rel.y,
                                 entity_delta.x,
                                 entity_delta.y,
                                 &mut t_min) {
                        wall_normal = V2 { x: 1.0, y: 0.0 };
                        hit_hf_e_index = lf_test_entity.hf_index;
                    }
                    if test_wall(min_corner.x,
                                 min_corner.y,
                                 max_corner.y,
                                 rel.x,
                                 rel.y,
                                 entity_delta.x,
                                 entity_delta.y,
                                 &mut t_min) {
                        wall_normal = V2 { x: -1.0, y: 0.0 };
                        hit_hf_e_index = lf_test_entity.hf_index;
                    }
                    if test_wall(max_corner.y,
                                 min_corner.x,
                                 max_corner.x,
                                 rel.y,
                                 rel.x,
                                 entity_delta.y,
                                 entity_delta.x,
                                 &mut t_min) {
                        wall_normal = V2 { x: 0.0, y: 1.0 };
                        hit_hf_e_index = lf_test_entity.hf_index;
                    }
                    if test_wall(min_corner.y,
                                 min_corner.x,
                                 max_corner.x,
                                 rel.y,
                                 rel.x,
                                 entity_delta.y,
                                 entity_delta.x,
                                 &mut t_min) {
                        wall_normal = V2 { x: 0.0, y: -1.0 };
                        hit_hf_e_index = lf_test_entity.hf_index;
                    }
                }
            }
        }

        hf_entity.position = hf_entity.position + entity_delta * t_min;
        if hit_hf_e_index.is_some() {
            hf_entity.velocity = hf_entity.velocity -
                                 wall_normal * math::dot_2(hf_entity.velocity, wall_normal);
            entity_delta = target_pos - hf_entity.position;
            entity_delta = entity_delta - wall_normal * math::dot_2(entity_delta, wall_normal);
        }
    }

    // adjust facing direction depending on velocity
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

    // map high_entity back to the low entity
    let lf_entity = entity.get_lf();
    let new_pos = map_into_world_space(world, camera_position, &hf_entity.position);
    let old_pos = lf_entity.world_position;
    world.change_entity_location(entity.lf_index,
                                 lf_entity,
                                 old_pos,
                                 Some(new_pos),
                                 world_arena);
}

fn test_wall(wall_value: f32,
             min_ortho: f32,
             max_ortho: f32,
             rel_x: f32,
             rel_y: f32,
             delta_x: f32,
             delta_y: f32,
             t_min: &mut f32)
             -> bool {
    let t_epsilon = 0.001;
    if delta_x != 0.0 {
        let t_res = (wall_value - rel_x) / delta_x;
        if t_res >= 0.0 && t_res < *t_min {
            let y = rel_y + t_res * delta_y;
            if min_ortho <= y && y <= max_ortho {
                if t_res - t_epsilon < 0.0 {
                    *t_min = 0.0;
                    return true;
                } else {
                    *t_min = t_res - t_epsilon;
                    return true;
                }
            }
        }
    }
    return false;
}

struct HeroBitmaps<'a> {
    head: graphics::Bitmap<'a>,
    torso: graphics::Bitmap<'a>,
    cape: graphics::Bitmap<'a>,

    align: V2<i32>,
}

#[derive(Copy, Clone)]
enum EntityType {
    None,
    Hero,
    Wall,
    Monster,
    Familiar,
    Sword,
}

impl Default for EntityType {
    fn default() -> EntityType {
        EntityType::None
    }
}

struct EntityPiece<'a> {
    bitmap: Option<&'a graphics::Bitmap<'a>>,
    offset: V2<f32>,
    offset_z: f32,
    alpha: f32,
    entity_zc: f32,

    r: f32,
    g: f32,
    b: f32,

    dim: V2<f32>,
}

struct EntityPieceGroup<'a> {
    meters_to_pixel: f32,
    count: usize,
    pieces: [Option<EntityPiece<'a>>; 32],
}

impl<'a> EntityPieceGroup<'a> {
    fn push_piece(&mut self,
                  bitmap: Option<&'a graphics::Bitmap<'a>>,
                  offset: V2<f32>,
                  offset_z: f32,
                  dim: V2<f32>,
                  color: Color,
                  align: V2<f32>,
                  entity_zc: f32) {
        debug_assert!((self.count) < self.pieces.len());
        let piece = &mut self.pieces[self.count];
        self.count += 1;
        *piece = Some(EntityPiece {
            bitmap: bitmap,
            offset: V2 {
                x: offset.x,
                y: -offset.y,
            } * self.meters_to_pixel - align,
            offset_z: offset_z * self.meters_to_pixel,
            alpha: color.a,
            entity_zc: entity_zc,

            r: color.r,
            g: color.g,
            b: color.b,

            dim: dim,
        });
    }

    fn push_rect(&mut self,
                 offset: V2<f32>,
                 offset_z: f32,
                 entity_zc: f32,
                 dim: V2<f32>,
                 color: Color) {
        self.push_piece(None, offset, offset_z, dim, color, V2::default(), entity_zc);
    }

    fn push_bitmap(&mut self,
                   bitmap: &'a graphics::Bitmap<'a>,
                   offset: V2<f32>,
                   offset_z: f32,
                   align: V2<i32>,
                   entity_zc: f32,
                   alpha: f32) {

        let color = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: alpha,
        };
        let align_float = V2 {
            x: align.x as f32,
            y: align.y as f32,
        };
        self.push_piece(Some(bitmap),
                        offset,
                        offset_z,
                        V2::default(),
                        color,
                        align_float,
                        entity_zc);
    }
}


const HITPOINTS_ARRAY_MAX: usize = 16;
const HITPOINT_SUB_COUNT: u8 = 4;

#[derive(Copy, Clone)]
struct MoveSpec {
    unit_max_accel_vector: bool,
    drag: f32,
    speed: f32,
}

impl Default for MoveSpec {
    fn default() -> MoveSpec {
        MoveSpec {
            unit_max_accel_vector: false,
            drag: 0.0,
            speed: 1.0,
        }
    }
}

#[derive(Default, Copy, Clone)]
struct Hitpoint {
    flags: u8,
    filled: u8,
}

#[derive(Copy, Clone)]
pub struct LfEntity {
    pub etype: EntityType,
    pub world_position: Option<WorldPosition>,
    pub dim: V2<f32>,
    pub collides: bool,

    pub max_hitpoints: u32,
    pub hitpoints: [Hitpoint; HITPOINTS_ARRAY_MAX],

    pub sword_lf_index: Option<usize>,
    pub distance_remaining: f32,
    pub velocity: V2<f32>,

    pub tbob: f32,
    pub face_direction: u32,
}


pub struct GameState<'a> {
    pub world_arena: MemoryArena,
    pub world: &'a mut World,

    pub meters_to_pixel: f32,

    pub camera_follows_entity_index: Option<usize>,
    pub camera_position: WorldPosition,

    pub player_index_for_controller: [Option<usize>; MAX_CONTROLLERS],

    // TODO: rename all lf stuff to stored!
    pub lf_entity_count: usize,
    pub lf_entities: [LfEntity; 100000],

    pub background_bitmap: graphics::Bitmap<'a>,
    pub shadow: graphics::Bitmap<'a>,
    pub tree: graphics::Bitmap<'a>,
    pub sword: graphics::Bitmap<'a>,
    pub hero_bitmaps: [HeroBitmaps<'a>; 4],
}
