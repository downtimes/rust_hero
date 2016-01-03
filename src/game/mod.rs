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
mod entity;

use self::world::{World, subtract};
use self::world::{WorldPosition, world_pos_from_tile};
use self::memory::MemoryArena;
use self::graphics::Color;
use self::math::{V2, V3, Rect};
use self::simulation::{SimEntity, SimRegion, EntityReference};
use self::entity::{update_player, update_sword, update_familiar, update_monster};


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

        // drop a test monster & familiar
        add_monster(state, cam_tile_x + 2, cam_tile_y + 2, cam_tile_z);
        add_familiar(state, cam_tile_x - 2, cam_tile_y + 2, cam_tile_z);

        game_memory.initialized = true;
    }

    let mut player_to_add = None;
    for (c_index, controller) in input.controllers.iter().enumerate() {

        if let Some(controlled_hero) = state.controlled_heroes[c_index].as_mut() {

            //Zero it out so we get only movement from the last frame!
            let saved_index = controlled_hero.entity_index;
            *controlled_hero = Default::default();
            controlled_hero.entity_index = saved_index;
            

            // Analog movement
            if controller.is_analog() {
                controlled_hero.acc = V2 {
                    x: controller.average_x.unwrap_or_default(),
                    y: controller.average_y.unwrap_or_default(),
                };

                // Digital movement
            } else {
                if controller.move_up.ended_down {
                    controlled_hero.acc.y = 1.0;
                }
                if controller.move_down.ended_down {
                    controlled_hero.acc.y = -1.0;
                }
                if controller.move_left.ended_down {
                    controlled_hero.acc.x = -1.0;
                }
                if controller.move_right.ended_down {
                    controlled_hero.acc.x = 1.0;
                }
            }

            if controller.action_up.ended_down {
                controlled_hero.d_sword = V2 { x: 0.0, y: 1.0 };
            }
            if controller.action_down.ended_down {
                controlled_hero.d_sword = V2 { x: 0.0, y: -1.0 };
            }
            if controller.action_left.ended_down {
                controlled_hero.d_sword = V2 { x: -1.0, y: 0.0 };
            }
            if controller.action_right.ended_down {
                controlled_hero.d_sword = V2 { x: 1.0, y: 0.0 };
            }
            if controller.start.ended_down {
                controlled_hero.d_z = 3.0;
            }

        } else {
            if controller.start.ended_down {
                player_to_add = Some(c_index);
            }
        }
    }

    if let Some(idx) = player_to_add {
        let e_index = add_player(state);
        let con_h = ControlledHero {
            entity_index: e_index,
            acc: V2::default() ,
            d_sword: V2::default(),
            d_z: 0.0,
        };
        state.controlled_heroes[idx] = Some(con_h);
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

    let mut sim_arena = MemoryArena::new(game_memory.transient.len(), game_memory.transient.as_ptr());
    let camera_pos = state.camera_position;
    let sim_region = SimRegion::begin_sim(state,
                                          &mut sim_arena,
                                          camera_pos,
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
        let &mut GameState { ref hero_bitmaps, ref controlled_heroes,
                             ref shadow, ref sword, ref tree, .. } = state;

        //TODO: get rid of unsafe sharing of data
        let sim_entity: &mut SimEntity = unsafe {&mut *((&mut sim_region.entities[index]) as *mut _) };

        // TODO: is the alpha from the previous frame needs to be after
        // updates
        let z_alpha = if sim_entity.z > 1.0 {
            0.0
        } else {
            1.0 - sim_entity.z * 0.8
        };


        let hero_bitmaps = &hero_bitmaps[sim_entity.face_direction as usize];
        let mut piece_group = EntityPieceGroup {
            meters_to_pixel: meters_to_pixel,
            count: 0,
            pieces: make_array!(None, 32),
        };

        match sim_entity.etype {
            EntityType::Hero => {
                for controlled_hero in controlled_heroes {
                    if let Some(con_hero) = controlled_hero.as_ref() {
                        if con_hero.entity_index == sim_entity.storage_index {
                            update_player(sim_region,
                                          sim_entity, input.delta_t,
                                          con_hero);
                        }
                    }
                }
                piece_group.push_bitmap(shadow,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(&hero_bitmaps.torso,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                piece_group.push_bitmap(&hero_bitmaps.cape,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                piece_group.push_bitmap(&hero_bitmaps.head,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                draw_hitpoints(sim_entity, &mut piece_group);

            }

            EntityType::Sword => {
                update_sword(sim_region,
                             sim_entity, input.delta_t);
                piece_group.push_bitmap(shadow,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(sword,
                                        V2::default(),
                                        0.0,
                                        V2 { x: 29, y: 13 },
                                        0.0,
                                        1.0);
            }

            EntityType::Monster => {
                update_monster(sim_region,
                               sim_entity,
                               input.delta_t);
                piece_group.push_bitmap(shadow,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        z_alpha);
                piece_group.push_bitmap(&hero_bitmaps.torso,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
                draw_hitpoints(sim_entity, &mut piece_group);
            }

            EntityType::Wall => {
                piece_group.push_bitmap(tree,
                                        V2::default(),
                                        0.0,
                                        V2 { x: 40, y: 80 },
                                        1.0,
                                        1.0);
            }

            EntityType::Familiar => {
                update_familiar(sim_region,
                                sim_entity,
                                input.delta_t);
                sim_entity.tbob += input.delta_t;
                if sim_entity.tbob > 2.0 * PI {
                    sim_entity.tbob -= 2.0 * PI;
                }
                let bob_sign = (sim_entity.tbob * 2.0).sin();
                piece_group.push_bitmap(&shadow,
                                        V2::default(),
                                        0.0,
                                        hero_bitmaps.align,
                                        0.0,
                                        (0.5 * z_alpha) + 0.2 * bob_sign);
                piece_group.push_bitmap(&hero_bitmaps.head,
                                        V2::default(),
                                        0.25 * bob_sign,
                                        hero_bitmaps.align,
                                        1.0,
                                        1.0);
            }
            _ => {
                debug_assert!(true, "We forgot an important case!");
            }
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
                   (meters_to_pixel * sim_entity.z) * piece.entity_zc,
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

    //TODO: Figure out origin and fix the non-following camera
    //TODO: Add logic for handleing unplace entities
    let origin: WorldPosition = Default::default();
    let V3 {x, y, ..} = subtract(sim_region.world, &origin, &sim_region.origin);
    let diff = V2 { x: x, y: y};
    graphics::draw_rect(video_buffer, diff, V2 {x: 10.0, y: 10.0}, 1.0, 1.0, 0.0);


    sim_region.end_sim(state);
}

// ======== End of the public interface =========

fn draw_hitpoints<'a>(sim_entity: &SimEntity, piece_group: &mut EntityPieceGroup<'a>) {
    if sim_entity.max_hitpoints >= 1 {
        let health_dim = V2 { x: 0.2, y: 0.2 };
        let spacing_x = health_dim.x * 1.5;
        let first_x = (sim_entity.max_hitpoints - 1) as f32 * 0.5 * spacing_x;
        let mut hit_p = V2 {
            x: -first_x,
            y: -0.25,
        };
        let d_hit_p = V2 {
            x: spacing_x,
            y: 0.0,
        };
        for idx in 0..sim_entity.max_hitpoints as usize {
            let hp = &sim_entity.hitpoints[idx];
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


fn add_lf_entity<'a>(state: &'a mut GameState,
                     etype: EntityType,
                     pos: Option<WorldPosition>)
                     -> (usize, &'a mut LfEntity) {
    let index = state.lf_entity_count;
    debug_assert!((index) < state.lf_entities.len());

    state.lf_entity_count += 1;

    let sim_ent = SimEntity {
        position: V2::default(),
        chunk_z: 0,

        z: 0.0,
        dz: 0.0,

        storage_index: 0,

        etype: etype,
        dim: V2::default(),
        collides: false,

        max_hitpoints: 0,
        hitpoints: [Hitpoint::default(); HITPOINTS_ARRAY_MAX],

        sword: None,
        distance_remaining: 0.0,

        velocity: V2::default(),
        tbob: 0.0,
        face_direction: 0,
    };

    state.lf_entities[index] = LfEntity {
        sim: sim_ent,
        world_position: pos.unwrap_or_default(),
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

    lf_entity.sim.dim = V2 {
        x: tile_side_meters,
        y: tile_side_meters,
    };
    lf_entity.sim.collides = true;

    e_index
}

fn add_monster(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32, abs_tile_z: i32) -> usize {
    let pos = world_pos_from_tile(state.world, abs_tile_x, abs_tile_y, abs_tile_z);

    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Monster, Some(pos));

    lf_entity.sim.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.sim.collides = true;
    init_hit_points(3, lf_entity);

    e_index
}

fn add_sword(state: &mut GameState) -> usize {
    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Sword, None);

    lf_entity.sim.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.sim.collides = false;

    e_index
}

fn add_familiar(state: &mut GameState, abs_tile_x: i32, abs_tile_y: i32, abs_tile_z: i32) -> usize {
    let pos = world_pos_from_tile(state.world, abs_tile_x, abs_tile_y, abs_tile_z);

    let (e_index, lf_entity) = add_lf_entity(state, EntityType::Familiar, Some(pos));

    lf_entity.sim.dim = V2 { x: 1.0, y: 0.5 };
    lf_entity.sim.collides = false;

    e_index
}

fn init_hit_points(hit_point_count: u8, lf_entity: &mut LfEntity) {
    debug_assert!(hit_point_count as usize <= HITPOINTS_ARRAY_MAX);
    lf_entity.sim.max_hitpoints = hit_point_count as u32;
    for idx in 0..lf_entity.sim.max_hitpoints as usize {
        lf_entity.sim.hitpoints[idx].filled = HITPOINT_SUB_COUNT;
    }
}

fn add_player(state: &mut GameState) -> usize {

    let e_index = {
        let mut pos = state.camera_position;
        pos.offset.x = 5.0;
        pos.offset.y = 5.0;

        let (e_index, lf_entity) = add_lf_entity(state, EntityType::Hero, Some(pos));

        lf_entity.sim.dim = V2 { x: 1.0, y: 0.5 };
        lf_entity.sim.collides = true;
        init_hit_points(3, lf_entity);

        e_index
    };

    let s_index = add_sword(state);
    state.lf_entities[e_index].sim.sword = Some(EntityReference::Index(s_index));

    if state.camera_follows_entity_index.is_none() {
        state.camera_follows_entity_index = Some(e_index);
    }

    e_index
}

pub struct HeroBitmaps<'a> {
    pub head: graphics::Bitmap<'a>,
    pub torso: graphics::Bitmap<'a>,
    pub cape: graphics::Bitmap<'a>,

    pub align: V2<i32>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
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


pub const HITPOINTS_ARRAY_MAX: usize = 16;
const HITPOINT_SUB_COUNT: u8 = 4;

#[derive(Copy, Clone)]
pub struct MoveSpec {
    pub unit_max_accel_vector: bool,
    pub drag: f32,
    pub speed: f32,
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

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Hitpoint {
    pub flags: u8,
    pub filled: u8,
}

#[derive(Copy, Clone)]
pub struct LfEntity {
    pub sim: SimEntity,
    pub world_position: WorldPosition,
}

#[derive(Copy, Clone, Default)]
pub struct ControlledHero{
    entity_index: usize,
    acc: V2<f32>,
    d_sword: V2<f32>,
    d_z: f32,
}

pub struct GameState<'a> {
    pub world_arena: MemoryArena,
    pub world: &'a mut World,

    pub meters_to_pixel: f32,

    pub camera_follows_entity_index: Option<usize>,
    pub camera_position: WorldPosition,

    pub controlled_heroes: [Option<ControlledHero>; MAX_CONTROLLERS],

    // TODO: rename all lf stuff to stored!
    pub lf_entity_count: usize,
    pub lf_entities: [LfEntity; 100000],

    pub background_bitmap: graphics::Bitmap<'a>,
    pub shadow: graphics::Bitmap<'a>,
    pub tree: graphics::Bitmap<'a>,
    pub sword: graphics::Bitmap<'a>,
    pub hero_bitmaps: [HeroBitmaps<'a>; 4],
}

impl<'a> GameState<'a> {
    pub fn get_stored_entity(&mut self, index: usize) -> Option<&mut LfEntity> {
        if index < self.lf_entity_count {
            Some(&mut self.lf_entities[index])
        } else {
            None
        }
    }
}
