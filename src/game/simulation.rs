use super::world::{World, WorldPosition, map_into_world_space, subtract};
use super::math::{Rect, V2, V3, dot_2};
use super::{GameState, LfEntity, get_lf_entity, EntityType, Hitpoint, HITPOINTS_ARRAY_MAX};
use super::MoveSpec;
use super::memory::MemoryArena;

use std::ptr;

pub enum EntityReference {
    Ptr(&SimEntity),
    Index(usize),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SimEntity {
    pub position: V2<f32>,
    pub chunk_z: i32,

    pub z: f32,
    pub dz: f32,

    pub storage_index: usize,

    pub etype: EntityType,
    pub dim: V2<f32>,
    pub collides: bool,

    pub max_hitpoints: u32,
    pub hitpoints: [Hitpoint; HITPOINTS_ARRAY_MAX],

    pub sword: Option<EntityReference>,
    pub distance_remaining: f32,
    pub velocity: V2<f32>,

    pub tbob: f32,
    pub face_direction: u32,
}

pub struct SimRegion<'a> {
    // TODO: need a hashtable here to map stored entitys to sim_entities
    world: &World,
    origin: WorldPosition,
    bounds: Rect<f32>,
    max_entity_count: usize,
    entity_count: usize,
    entities: &'a mut [SimEntity],

    // TODO: Is Hash really the best data structure for this?
    hash_table: [Option<SimEntityHash>; 4096],
}

struct SimEntityHash {
    index: usize,
    ptr: &mut SimEntity,
}

fn store_entity_reference(reference: &mut EntityReference) {
    reference = if let EntityReference::Ptr(ptr) = reference {
        EntityReference::Index(ptr.storage_index)
    } else {
        reference
    };
}


fn get_entity_by_index(sim_region: &SimRegion, store_index: usize) -> Option<&mut SimEntity> {
    sim_region.get_hash_from_index(store_index).map(|hash| hash.ptr)
}

fn load_entity_reference(sim_region: &mut SimRegion,
                         state: &mut GameState,
                         reference: &mut EntityReference) {
    reference = if let EntityReference::Index(idx) = reference {
        let hash = sim_region.get_hash_from_index(idx);
        if hash.is_some() {
            EntityReference::Ptr(hash.unwrap().ptr)
        } else {
            EntityReference::Ptr(sim_region.add_entity(state.lf_entities[idx], idx, None))
        }
    } else {
        reference
    };
}


impl SimRegion {
    pub fn get_sim_space_p(&self, entity: &LfEntity) -> V2<f32> {

        let V3{ x, y, .. } = subtract(self.world, &entity.world_position.unwrap(), self.origin);
        V2 { x: x, y: y }
    }

    pub fn end_sim(&mut self, state: &mut GameState) {

        // TODO: Maybe store entities in the world and don't need GameState here
        for index in 0..self.entity_count {
            let entity = &mut self.entitis[index];
            let store_index = entity.storage_index;
            let stored_entity = &mut state.lf_entities[store_index];

            stored_entity.sim = *entity;
            store_entity_reference(&mut stored_entity.sim.sword);

            // TODO: Safe state back to stored entity once high entities
            // do state decompression, etc.
            let new_pos = map_into_world_space(state.world, self.origin, entity.position);
            self.world.change_entity_location(store_index,
                                              stored_entity,
                                              stored_entity.world_position,
                                              Some(new_pos),
                                              state.world_arena);
            if state.camera_follows_entity_index.is_some() &&
               (entity.storage_index == state.camera_follows_entity_index.unwrap()) {
                let new_cam_p = stored_entity.world_position;
            }
        }
    }

    fn add_entity(&mut self,
                  state: &mut GameState,
                  source: &LfEntity,
                  store_index: usize,
                  sim_p: Option<V2<f32>>)
                  -> &mut SimEntity {
        let sim_ent = self.add_entity_raw(state, store_index, source);
        if sim_p.is_some() {
            sim_ent.position = sim_p.unwrap();
        } else {
            sim_ent.position = self.get_sim_space_p(source);
        }
        // TODO: convert StoredEntity to a simulation entity
        sim_ent
    }

    fn add_entity_raw(&mut self,
                      state: &mut GameState,
                      store_index: usize,
                      source: &LfEntity)
                      -> &mut SimEntity {
        let sim_ent;
        if self.entites.len() < self.max_entity_count {
            self.entity_count += 1;
            sim_ent = &mut self.entities[self.entity_count];
            self.map_index_to_sim_entity(store_index, sim_ent);

            // TODO: Decompression instead of just a plain copy
            *sim_ent = source.sim;
            load_entity_reference(self, state, sim_ent.sword);
            sim_ent.storage_index = store_index;
        } else {
            panic!("Not allowed to instert more entities than that!");
        }

        sim_ent
    }

    fn map_index_to_sim_entity(&mut self, store_index: usize, entity: &mut SimEntity) {
        let hash = self.get_hash_from_index(store_index);
        debug_assert!(hash.is_none() || (hash.unwrap().index == store_index));

        if hash.is_some() {
            let hash = hash.unwrap();
            hash.ptr = entity;
        }
    }

    fn get_hash_from_index(&mut self, store_index: usize) -> Option<&mut SimEntityHash> {
        let hash_value = store_index as u32;

        // Look through the whole table for a slot
        for idx in 0..self.hash_table.len() {
            let entry = self.hash_table[(store_index + idx) % self.hash_table.len()];
            if entry.is_none() {
                return None;
            } else if entry.unwrap().index == store_index {
                return Some(entry);
            }
        }
    }

    pub fn begin_sim(state: &mut GameState,
                     origin: WorldPosition,
                     bounds: Rect<f32>)
                     -> &'a mut SimRegion<'a> {
        // TODO: If entities were stored in the world we wouldn't need a gamestate here
        // TODO: clear the hashtable with ptr::write_bytes
        // TODO: Notion of inactive sim_entities for the apron around the simulation

        let sim_region: &mut SimRegion = state.world_arena.push_struct();
        sim_region.world = state.world;
        sim_region.origin = origin;
        sim_region.bounds = bounds;
        // TODO: Needs to be more specific later on?
        sim_region.max_entity_count = 4024;
        sim_region.entity_count = 0;
        sim_region.entities = state.world_arena.push_slice(sim_region.max_entity_count);

        let min_p = map_into_world_space(state.world,
                                         sim_region.center,
                                         &sim_region.bounds.get_min());
        let max_p = map_into_world_space(state.world,
                                         sim_region.center,
                                         &sim_region.bounds.get_max());
        for ch in state.world.iter_spatially(min_p, max_p, sim_region.center.chunk_z) {
            for block in ch.first_block.iter() {
                for block_idx in 0..block.e_count {
                    let lf_index = block.lf_entities[block_idx];
                    let lf_entity = state.lf_entities[lf_index];

                    let &mut GameState{ ref mut lf_entities, ref mut hf_entities,
                                        ref mut hf_entity_count, ref world,
                                        ref camera_position, .. } = state;
                    let sim_space_p = sim_region.get_sim_space_p(lf_entity);
                    if sim_region.bounds.p_inside(sim_space_p) {
                        // TODO: check a second rectangle to set the entity to be movable
                        // or not
                        sim_region.add_entity(state, lf_entity, lf_index, Some(sim_space_p));
                    }
                }
            }
        }

        sim_region
    }

    pub fn move_entity(&mut self,
                       lf_entities: &mut [LfEntity],
                       world_arena: &mut MemoryArena,
                       camera_position: &WorldPosition,
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

        let hf_entity = entity.get_hf();
        // friction force currently just by rule of thumb;
        acc = acc - entity.velocity * move_spec.drag;


        // Copy old player Position before we handle input
        let mut entity_delta = acc * 0.5 * delta_t.powi(2) + entity.velocity * delta_t;
        entity.velocity = acc * delta_t + entity.velocity;


        // try the collission detection multiple times to see if we can move with
        // a corrected velocity after a collision
        // TODO: we always loop 4 times. Look for a fast out if we moved enough
        for _ in 0..4 {
            let mut t_min = 1.0;
            let mut wall_normal = Default::default();
            let mut hit_hf_e_index = None;

            let target_pos = entity.position + entity_delta;

            let mut hit_entity = None;
            if entity.collides {
                // TODO: do a spatial partition here eventually
                for e_index in 0..self.entity_count {
                    let test_entity = &self.entities[e_index];
                    let test_entity_storage_index = test_entity.storage_index;

                    if test_entity_storage_index == entity.storage_index {
                        continue;
                    }


                    if test_entity.collides {
                        // Minkowski Sum
                        let diameter = V2 {
                            x: test_entity.dim.x + entity.dim.x,
                            y: test_entity.dim.y + entity.dim.y,
                        };

                        let min_corner = diameter * -0.5;
                        let max_corner = diameter * 0.5;
                        let rel = hf_entity.position - test_entity.position;

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

            entity.position = entity.position + entity_delta * t_min;
            if hit_entity.is_some() {
                entity.velocity = entity.velocity -
                                  wall_normal * dot_2(entity.velocity, wall_normal);
                entity_delta = target_pos - entity.position;
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
