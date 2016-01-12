use super::world::{World, WorldPosition, map_into_world_space, subtract};
use super::math::{Rect, V2, V3, dot_2};
use super::{GameState, LfEntity, EntityType, Hitpoint, HITPOINTS_ARRAY_MAX};
use super::MoveSpec;
use super::memory::MemoryArena;

use std::ptr;

bitflags! {
    flags EntityFlags: u32 {
        const COLLIDES      = 0b0001,
        const SIMMING       = 0b1000,
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
    pub etype: EntityType,

    pub position: Option<V2<f32>>,
    pub velocity: V2<f32>,
    pub z: f32,
    pub dz: f32,

    pub chunk_z: i32,

    pub storage_index: usize,

    pub dim: V2<f32>,
    pub flags: EntityFlags,

    pub max_hitpoints: u32,
    pub hitpoints: [Hitpoint; HITPOINTS_ARRAY_MAX],

    pub sword: Option<EntityReference>,
    pub distance_remaining: f32,

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
    pub max_entity_count: usize,
    pub entity_count: usize,
    pub entities: &'a mut [SimEntity],

    // TODO: Is Hash really the best data structure for this?
    pub hash_table: [Option<SimEntityHash<'a>>; HASH_TABLE_LEN],
}

pub struct SimEntityHash<'a> {
    pub index: usize,
    pub ptr: &'a mut SimEntity,
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

fn load_entity_reference<'a>(sim_region: &mut SimRegion,
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

            debug_assert!(stored_entity.sim.flags.contains(SIMMING));
            stored_entity.sim = *entity;
            debug_assert!(!stored_entity.sim.flags.contains(SIMMING));

            if let Some(sword) = stored_entity.sim.sword.as_mut() {
                *sword = store_entity_reference(*sword);
            }

            // TODO: Safe state back to stored entity once high entities
            // do state decompression, etc.

            let new_pos = if entity.position.is_some() {
                Some(map_into_world_space(state.world, &self.origin, &entity.position.unwrap()))
            } else {
                None
            };
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
        let sim_ent = self.add_entity_raw(state, store_index, source);
        sim_ent.position = sim_space;
        // TODO: convert StoredEntity to a simulation entity
        sim_ent
    }

    fn add_entity_raw(&mut self,
                      state: &mut GameState,
                      store_index: usize,
                      source: &mut LfEntity)
                      -> &mut SimEntity {

        let sim_ent;
        if self.get_hash_from_index(store_index).is_none() {
            if self.entities.len() >= self.max_entity_count - 1 {
                // TODO: unsafe not good!
                sim_ent = unsafe { &mut *(&mut self.entities[self.entity_count] as *mut _) };

                self.add_hash(store_index, sim_ent);

                self.entity_count += 1;

                // TODO: Decompression instead of just a plain copy
                *sim_ent = source.sim;
                if let Some(sword) = sim_ent.sword {
                    sim_ent.sword = Some(load_entity_reference(self, state, sword));
                }
                sim_ent.storage_index = store_index;

                debug_assert!(!source.sim.flags.contains(SIMMING));
                source.sim.flags.insert(SIMMING);
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
        // TODO: get rid of unsafe here
        sim_region.world = unsafe { &mut *(state.world as *mut _) };
        sim_region.origin = origin;
        sim_region.bounds = bounds;
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
                        // TODO: check a second rectangle to set the entity to be movable
                        // or not
                        sim_region.add_entity(state, lf_entity, lf_index, sim_space_p);
                    }
                }
            }
        }

        sim_region
    }

    pub fn move_entity(&mut self,
                       entity: &mut SimEntity,
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

        // friction force currently just by rule of thumb;
        acc = acc - entity.velocity * move_spec.drag;

        // Gravity and "jumping"
        let gravity = -9.81;
        entity.z += gravity * 0.5 * delta_t.powi(2) + entity.dz * delta_t;
        entity.dz = gravity * delta_t + entity.dz;

        if entity.z < 0.0 {
            entity.z = 0.0;
        }


        // Copy old player Position before we handle input
        let mut entity_delta = acc * 0.5 * delta_t.powi(2) + entity.velocity * delta_t;
        entity.velocity = acc * delta_t + entity.velocity;


        // try the collission detection multiple times to see if we can move with
        // a corrected velocity after a collision
        // TODO: we always loop 4 times. Look for a fast out if we moved enough
        for _ in 0..4 {
            let mut t_min = 1.0;
            let mut wall_normal = Default::default();
            let mut hit_entity = None;

            let target_pos = entity.position.unwrap() + entity_delta;

            if entity.flags.contains(COLLIDES) {
                // TODO: do a spatial partition here eventually
                for e_index in 0..self.entity_count {
                    let test_entity = &self.entities[e_index];
                    let test_entity_storage_index = test_entity.storage_index;

                    if test_entity_storage_index == entity.storage_index {
                        continue;
                    }


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
            if hit_entity.is_some() {
                entity.velocity = entity.velocity -
                                  wall_normal * dot_2(entity.velocity, wall_normal);
                entity_delta = target_pos - entity.position.unwrap();
                entity_delta = entity_delta - wall_normal * dot_2(entity_delta, wall_normal);
            }
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
