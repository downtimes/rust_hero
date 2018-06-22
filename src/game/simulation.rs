use super::world::{World, WorldPosition, map_into_world_space, subtract};
use super::math::{Rect, V2, V3, dot_2};
use super::{GameState, LfEntity, EntityType, Hitpoint, HITPOINTS_ARRAY_MAX};
use super::{MoveSpec, add_collision_rule, should_collide};
use super::memory::MemoryArena;

use std::ptr;

bitflags! {
    pub struct EntityFlags: u32 {
        const COLLIDES      = 0b0001;
        const SIMMING       = 0b1000;
    }
}

// TODO: Get rid of all the raw Pointers once the exploratory Code is done
#[derive(Copy, Clone, PartialEq)]
pub enum EntityReference {
    Ptr(*mut SimEntity),
    Index(usize),
}

// TODO: Rename this struct to Entity instead? and the other one to StorageEntity
#[derive(Copy, Clone, PartialEq)]
pub struct SimEntity {
    //Theese are only for the sim region
    pub storage_index: usize,
    pub can_update: bool,
    

    //Rest of the entity
    pub etype: EntityType,

    pub position: Option<V2<f32>>,
    pub velocity: V2<f32>,
    pub z: f32,
    pub dz: f32,

    pub distance_limit: f32,

    pub chunk_z: i32,

    pub dim: V2<f32>,
    pub flags: EntityFlags,

    pub max_hitpoints: u32,
    pub hitpoints: [Hitpoint; HITPOINTS_ARRAY_MAX],

    pub sword: Option<EntityReference>,

    pub tbob: f32,
    pub face_direction: u32,
}

impl SimEntity {
    pub fn make_non_spatial(&mut self) {
        self.position = None;
    }

    pub fn make_spatial(&mut self, pos: V2<f32>, velocity: V2<f32>) {
        self.position = Some(pos);
        self.velocity = velocity;
    }
}


const HASH_TABLE_LEN: usize = 4096;
pub struct SimRegion<'a> {
    pub world: &'a mut World,
    pub origin: WorldPosition,
    pub bounds: Rect<f32>,
    pub updatable_bounds: Rect<f32>,
    pub max_entity_count: usize,
    pub entity_count: usize,
    pub entities: &'a mut [SimEntity],

    // TODO: Is Hash really the best data structure for this?
    pub hash_table: [Option<SimEntityHash<'a>>; HASH_TABLE_LEN],
}

impl<'a> SimRegion<'a> {
    pub fn get_entity_ref<'b>(&mut self, index: usize) -> &'b mut SimEntity {
        unsafe{ &mut *(&mut self.entities[index] as *mut _) }
    }
}

pub struct SimEntityHash<'a> {
    pub index: usize,
    pub ptr: &'a mut SimEntity,
}

#[derive(Copy, Clone)]
pub struct PairCollisionRule<'a> {
    pub storage_index_a: usize,
    pub storage_index_b: usize,
    pub should_collide: bool,
    pub next_rule: &'a Option<PairCollisionRule<'a>>,
}

fn store_entity_reference(reference: EntityReference) -> EntityReference {
    if let EntityReference::Ptr(ptr) = reference {
        let storage_index = unsafe { (*ptr).storage_index };
        EntityReference::Index(storage_index)
    } else {
        reference
    }
}


#[allow(dead_code)]
fn get_entity_by_index<'a>(sim_region: &'a mut SimRegion,
                           store_index: usize)
                           -> Option<&'a mut SimEntity> {
    match sim_region.get_hash_from_index(store_index) {
        Some(hash) => Some(hash.ptr),
        None => None,
    }
}

fn load_entity_reference(sim_region: &mut SimRegion,
                             state: &mut GameState,
                             reference: EntityReference)
                             -> EntityReference {
    if let EntityReference::Index(idx) = reference {
        if sim_region.get_hash_from_index(idx).is_some() {
            let hash = sim_region.get_hash_from_index(idx).unwrap();
            EntityReference::Ptr(hash.ptr)
        } else {
            // TODO: Remove lifetime hacks here!
            let entity = unsafe { &mut *(state.get_stored_entity(idx).unwrap() as *mut _) };
            EntityReference::Ptr(sim_region.add_entity(state, entity, idx, None))
        }
    } else {
        reference
    }
}


impl<'a> SimRegion<'a> {
    pub fn get_sim_space_p(&self, entity: &LfEntity) -> Option<V2<f32>> {

        entity.world_position.and_then(|pos| {
            let V3{ x, y, .. } = subtract(self.world, &pos, &self.origin);
            Some(V2 { x: x, y: y })
        })
    }

    pub fn end_sim(&mut self, state: &mut GameState<'a>) {

        // TODO: Maybe store entities in the world and don't need GameState here
        for index in 0..self.entity_count {
            let entity = &mut self.entities[index];
            let store_index = entity.storage_index;
            let stored_entity = &mut state.lf_entities[store_index];

            debug_assert!(stored_entity.sim.flags.contains(EntityFlags::SIMMING));
            stored_entity.sim = *entity;
            debug_assert!(!stored_entity.sim.flags.contains(EntityFlags::SIMMING));

            if let Some(sword) = stored_entity.sim.sword.as_mut() {
                *sword = store_entity_reference(*sword);
            }

            // TODO: Safe state back to stored entity once high entities
            // do state decompression, etc.

            let world = &mut state.world;
            let origin = &self.origin;
            let new_pos = entity.position.map(|position| map_into_world_space(world, origin, &position));
            self.world.change_entity_location(store_index,
                                              stored_entity,
                                              new_pos,
                                              &mut state.world_arena);
            if state.camera_follows_entity_index.is_some() &&
               (entity.storage_index == state.camera_follows_entity_index.unwrap()) {
                state.camera_position = stored_entity.world_position.unwrap();
            } 
        }
    }

    fn add_entity(&mut self,
                  state: &mut GameState,
                  source: &mut LfEntity,
                  store_index: usize,
                  sim_p: Option<V2<f32>>)
                  -> &mut SimEntity {
        let sim_space = match sim_p {
            Some(_) => sim_p,
            None => self.get_sim_space_p(source),
        };
        let updatable_bounds = self.updatable_bounds;
        let sim_ent = self.add_entity_raw(state, store_index, source);
        sim_ent.position = sim_space;
        if let Some(position) = sim_space {
            sim_ent.can_update = updatable_bounds.p_inside(position);
        }
        // TODO: convert StoredEntity to a simulation entity
        sim_ent
    }

    fn add_entity_raw(&mut self,
                      state: &mut GameState,
                      store_index: usize,
                      source: &mut LfEntity)
                      -> &mut SimEntity {

        let sim_ent;
        let is_contained = 
            if let Some(hash) = self.get_hash_from_index(store_index) {
                hash.index == store_index
            } else {
                false
            };
            
        if !is_contained {
            if self.entities.len() - 1 >= self.entity_count {
                let ent_count = self.entity_count;
                sim_ent = self.get_entity_ref(ent_count);

                self.add_hash(store_index, sim_ent);

                self.entity_count += 1;

                // TODO: Decompression instead of just a plain copy
                *sim_ent = source.sim;
                if let Some(sword) = sim_ent.sword {
                    sim_ent.sword = Some(load_entity_reference(self, state, sword));
                }
                sim_ent.storage_index = store_index;
                sim_ent.can_update = false;

                debug_assert!(!source.sim.flags.contains(EntityFlags::SIMMING));
                source.sim.flags.insert(EntityFlags::SIMMING);
            } else {
                panic!("Not allowed to instert more entities than that!");
            }
        } else {
            sim_ent = self.get_hash_from_index(store_index).unwrap().ptr;
        }

        sim_ent
    }

    fn get_hash_from_index(&mut self, store_index: usize) -> Option<&mut SimEntityHash<'a>> {
        let hash_value = store_index;

        // Look through the whole table for a slot
        let len = self.hash_table.len();
        self.hash_table[hash_value % len].as_mut()
    }

    fn add_hash(&mut self, store_index: usize, sim_ent: &mut SimEntity) {

        let hash_value = store_index;

        // Look through the whole table for a slot
        let len = self.hash_table.len();

        self.hash_table[hash_value % len] = Some(SimEntityHash {
            index: store_index,
            ptr: unsafe { &mut *(sim_ent as *mut _) },
        });
    }

    pub fn begin_sim(state: &mut GameState,
                     sim_arena: &mut MemoryArena,
                     origin: WorldPosition,
                     bounds: Rect<f32>)
                     -> &'static mut SimRegion<'static> {

        // TODO: If entities were stored in the world we wouldn't need a gamestate here
        // TODO: Notion of inactive sim_entities for the apron around the simulation

        let sim_region: &mut SimRegion = sim_arena.push_struct();
        unsafe {
            ptr::write_bytes(sim_region.hash_table.as_mut_ptr(), 0, HASH_TABLE_LEN);
        }

        // TODO: caclulate eventually from all entities + speed
        let update_safety_margin = 1.0;

        sim_region.world = state.get_world_ref();
        sim_region.origin = origin;
        sim_region.bounds = bounds.add_radius(update_safety_margin, update_safety_margin);
        sim_region.updatable_bounds = bounds;
        // TODO: Needs to be more specific later on?
        sim_region.max_entity_count = 4024;
        sim_region.entity_count = 0;
        sim_region.entities = sim_arena.push_slice(sim_region.max_entity_count);

        let min_p = map_into_world_space(state.world,
                                         &sim_region.origin,
                                         &sim_region.bounds.get_min());
        let max_p = map_into_world_space(state.world,
                                         &sim_region.origin,
                                         &sim_region.bounds.get_max());
        for ch in state.world.iter_spatially(min_p, max_p, sim_region.origin.chunk_z) {
            for block in ch.first_block.iter() {
                for block_idx in 0..block.e_count {
                    let lf_index = block.lf_entities[block_idx];
                    let lf_entity = unsafe { &mut *((&mut state.lf_entities[lf_index]) as *mut _) };

                    let sim_space_p = sim_region.get_sim_space_p(lf_entity);
                    if sim_space_p.is_some() && sim_region.bounds.p_inside(sim_space_p.unwrap()) {
                        sim_region.add_entity(state, lf_entity, lf_index, sim_space_p);
                    }
                }
            }
        }

        sim_region
    }

    pub fn move_entity(&mut self,
                       arena: &mut MemoryArena,
                       table: &mut [Option<PairCollisionRule>],
                       entity: &mut SimEntity,
                       move_spec: &MoveSpec,
                       mut acc: V2<f32>,
                       delta_t: f32) {

        // Diagonal correction.
        if move_spec.unit_max_accel_vector && (acc.length_sq() > 1.0) {
            acc = acc.normalize();
        }

        acc = acc * move_spec.speed;

        // friction force currently just by rule of thumb;
        acc = acc - entity.velocity * move_spec.drag;

        // Gravity and "jumping"
        let gravity = -9.81;
        entity.z += gravity * 0.5 * delta_t.powi(2) + entity.dz * delta_t;
        entity.dz += gravity * delta_t;

        if entity.z < 0.0 {
            entity.z = 0.0;
            entity.dz = 0.0;
        }

        let mut entity_delta = acc * 0.5 * delta_t.powi(2) + entity.velocity * delta_t;
        entity.velocity = acc * delta_t + entity.velocity;

        let mut distance_remaining = 
            if entity.distance_limit == 0.0 {
                // TODO: this number needs to be somehow formalized
                1000.0
            } else {
                entity.distance_limit
            };

        // try the collission detection multiple times to see if we can move with
        // a corrected velocity after a collision
        // TODO: we always loop 4 times. Look for a fast out if we moved enough
        for _ in 0..4 {
            let mut t_min = 1.0;

            let entity_delta_length = entity_delta.length();
            // TODO: what is a good value for espilon here.
            // The check for a valid position is just an optimization to avoid
            // entering the loop if the entity is not spatial. The loop can handle
            // non-spatial entities just fine
            if entity_delta_length <= 0.0001 || entity.position.is_none() {
                break;
            }
            if entity_delta_length > distance_remaining {
                t_min = distance_remaining / entity_delta_length;
            }

            if entity.flags.contains(EntityFlags::COLLIDES) {
                // TODO: do a spatial partition here eventually
                for e_index in 0..self.entity_count {
                    let test_entity = &self.entities[e_index];
                    let test_entity_storage_index = test_entity.storage_index;

            let target_pos = entity.position.unwrap_or_default() + entity_delta;


            //if entity.flags.contains(COLLIDES) {
            // TODO: do a spatial partition here eventually
            for e_index in 0..self.entity_count {
                let test_entity = &mut self.entities[e_index];
                if should_collide(table, test_entity, entity) {
                    if test_entity.flags.contains(COLLIDES) {
                        // Minkowski Sum
                        let diameter = V2 {
                            x: test_entity.dim.x + entity.dim.x,
                            y: test_entity.dim.y + entity.dim.y,
                        };

                        let min_corner = diameter * -0.5;
                        let max_corner = diameter * 0.5;
                        let rel = entity.position.unwrap() - test_entity.position.unwrap();

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
                            hit_entity = Some(test_entity);
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
                            hit_entity = Some(test_entity);
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
                            hit_entity = Some(test_entity);
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
                            hit_entity = Some(test_entity);
                        }
                    }
                }
            }

            entity.position = Some(entity.position.unwrap() + entity_delta * t_min);
            distance_remaining -= entity_delta_length * t_min;
            if let Some(hit_ent_ptr) = hit_entity {
                let hit_ent = unsafe { &mut *hit_ent_ptr };
                entity_delta = target_pos - entity.position.unwrap();
                let stops_on_collision = handle_collision(entity, hit_ent);
                if stops_on_collision {
                    entity_delta = entity_delta - wall_normal * dot_2(entity_delta, wall_normal);
                    entity.velocity = entity.velocity -
                        wall_normal * dot_2(entity.velocity, wall_normal);
                } else {
                    add_collision_rule(arena, table, entity.storage_index, 
                                       hit_ent.storage_index, false);
                }
            }
        }

        if entity.distance_limit != 0.0 {
            entity.distance_limit = distance_remaining;
        }

        // adjust facing direction depending on velocity
        if entity.velocity.x.abs() > entity.velocity.y.abs() {
            if entity.velocity.x > 0.0 {
                entity.face_direction = 0;
            } else {
                entity.face_direction = 2;
            }
        } else if entity.velocity.x != 0.0 && entity.velocity.y != 0.0 {
            if entity.velocity.y > 0.0 {
                entity.face_direction = 1;
            } else {
                entity.face_direction = 3;
            }
        }
    }
}

fn handle_collision(mut a: &mut SimEntity, mut b: &mut SimEntity) -> bool {
    let stops_on_collision = 
        if a.etype == EntityType::Sword {
            false
        } else {
            true
        };


    let first;
    let second;
    if a.etype < b.etype {
        first = a;
        second = b;
    } else {
        first = b;
        second = a;
    }

    if first.etype == EntityType::Monster && second.etype == EntityType::Sword {
        if first.max_hitpoints > 0 {
            first.max_hitpoints -= 1;
        }
    }

    stops_on_collision
}

// TODO: write some documentation for easier understanding how this function works
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
    false
}
