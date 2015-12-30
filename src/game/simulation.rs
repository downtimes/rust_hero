use super::world::{World, WorldPosition, map_into_world_space, subtract};
use super::math::{Rect, V2, V3};
use super::{GameState, LfEntity, get_lf_entity};
use super::memory::MemoryArena;

pub struct SimEntity {
    position: V2<f32>,
    chunk_z: i32,

    z: f32,
    dz: f32,

    storage_index: usize,
}

pub struct SimRegion<'a> {
    // TODO: need a hashtable here to map stored entitys to sim_entities
    world: &World,
    origin: WorldPosition,
    bounds: Rect<f32>,
    max_entity_count: usize,
    entity_count: usize,
    entities: &'a mut [SimEntity],
}



impl SimRegion {
    pub fn get_sim_space_p(&self, entity: &LfEntity) -> V2<f32> {

        let V3{ x, y, .. } = subtract(self.world, &entity.world_position.unwrap(), self.origin);
        V2 { x: x, y: y }
    }

    pub fn end_sim(&mut self, state: &mut GameState) {

        // TODO: Maybe store entities in the world and don't need GameState here
        for index in 0..self.entity_count {
            let entity = self.entitis[index];
            let store_index = entity.storage_index;
            let stored_entity = &mut state.lf_entities[store_index];

            // TODO: Safe state back to stored entity once high entities
            // do state decompression, etc.
            let new_pos = map_into_world_space(state.world, self.origin, entity.position);
            self.world.change_entity_location(store_index,
                                              stored_entity,
                                              stored_entity.world_position,
                                              Some(new_pos),
                                              state.world_arena);
        }

        if state.camera_follows_entity_index.is_some() {
            let new_cam_pos;
            {
                let entity_idx = state.camera_follows_entity_index.unwrap();
                new_cam_pos = get_lf_entity(state, entity_idx).unwrap().world_position.unwrap();
            }
        }
    }

    fn add_entity(&mut self, source: LfEntity, sim_p: Option<V2<f32>>) {
        let sim_ent = self.add_entity_raw();
        if sim_p.is_some() {
            sim_ent.position = sim_p.unwrap();
        } else {
            sim_ent.position = self.get_sim_space_p(source);
        }
        // TODO: convert StoredEntity to a simulation entity
    }

    fn add_entity_raw(&mut self) -> &mut SimEntity {
        let sim_ent;
        if self.entites.len() < self.max_entity_count {
            self.entity_count += 1;
            sim_ent = &mut self.entities[self.entity_count];
            // TODO: clear the entity before we return it to the caller?
        } else {
            panic!("Not allowed to instert more entities than that!");
        }
        sim_ent
    }

    pub fn begin_sim(state: &GameState,
                     origin: WorldPosition,
                     bounds: Rect<f32>)
                     -> &'a mut SimRegion<'a> {
        // TODO: If entities were stored in the world we wouldn't need a gamestate here

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
                        sim_region.add_entity(lf_entity, Some(sim_space_p));
                    }
                }
            }
        }

        sim_region
    }
}
