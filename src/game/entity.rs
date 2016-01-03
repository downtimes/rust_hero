use super::simulation::{EntityReference, SimRegion, SimEntity};
use super::math::{V2};
use super::{EntityType, MoveSpec, ControlledHero};

use num::traits::Float;

pub fn update_sword(sim_region: &mut SimRegion,
                    entity: &mut SimEntity,
                    dt: f32) {

    let move_spec = MoveSpec {
        unit_max_accel_vector: false,
        speed: 0.0,
        drag: 0.0,
    };
    let old_p = entity.position;
    sim_region.move_entity(entity,
                           V2 { x: 0.0, y: 0.0 },
                           &move_spec,
                           dt);
    let travelled = (entity.position - old_p).length();
    entity.distance_remaining -= travelled;

    if entity.distance_remaining < 0.0 {
        debug_assert!(true, "Need to be able to 'remove' entitites from active set");
    }
}

pub fn update_player(sim_region: &mut SimRegion,
                     entity: &mut SimEntity,
                     dt: f32,
                     req: &ControlledHero) {

    let move_spec = MoveSpec {
        unit_max_accel_vector: true,
        drag: 8.0,
        speed: 50.0,
    };

    entity.dz = req.d_z;

    sim_region.move_entity(entity,
                           req.acc,
                           &move_spec,
                           dt);

    if (req.d_sword.x != 0.0) || (req.d_sword.y != 0.0) {
        if let Some(sword) = entity.sword {
            if let EntityReference::Ptr(ptr) = sword {
                let sword_refe = unsafe { &mut *ptr };

                sword_refe.position = entity.position;

                sword_refe.distance_remaining = 5.0;
                sword_refe.velocity = req.d_sword * 5.0;
            }
        }
    }
}

pub fn update_familiar(sim_region: &mut SimRegion,
                       entity: &mut SimEntity,
                       dt: f32) {
    let mut closest_hero_d_sq = 10.0.powi(2); //Maximum search range
    let mut closest_hero = None;


    //TODO: make spatial querys easy for things
    for test_idx in 0..sim_region.entity_count {
        let test_entity = sim_region.entities[test_idx];

        match test_entity.etype {
            EntityType::Hero => {
                let test_d_sq = (test_entity.position - entity.position).length_sq(); 
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
            let acceleration = 0.5;
            let one_over_length = acceleration / closest_hero_d_sq.sqrt();
            acc = (hero.position - entity.position) * one_over_length;
        }
    }

    let move_spec = MoveSpec {
        unit_max_accel_vector: true,
        drag: 8.0,
        speed: 50.0,
    };
    sim_region.move_entity(entity,
                           acc,
                           &move_spec,
                           dt);
}

#[allow(unused_variables)]
pub fn update_monster(sim_region: &mut SimRegion,
                      entity: &SimEntity,
                      dt: f32) {
}

