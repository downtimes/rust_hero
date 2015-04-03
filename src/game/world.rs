use std::num::wrapping::OverflowingOps;
use std::mem;
use std::i32;
use std::default::Default;

use super::memory::MemoryArena;
use super::math::V2f;

//TODO: Better hash function for our use case
fn get_hash(tile_chunk_x: i32, tile_chunk_y: i32, tile_chunk_z: i32) -> u32 {
    ((tile_chunk_x.overflowing_mul(19).0)
    .overflowing_add((tile_chunk_y.overflowing_mul(7)).0).0)
    .overflowing_add((tile_chunk_z.overflowing_mul(3)).0).0 as u32
}

pub fn map_into_world_space(world: &World, world_pos: &WorldPosition, rel_pos: &V2f) -> WorldPosition {

    let mut pos = *world_pos;
    pos.offset = pos.offset + *rel_pos;
    canonicalize_coord(world, &mut pos.tile_x, &mut pos.offset.x);
    canonicalize_coord(world, &mut pos.tile_y, &mut pos.offset.y);
    pos
}

pub struct World {
    pub tile_side_meters: f32,

    pub chunk_shift: u32,
    pub chunk_mask: i32,
    pub chunk_dim: usize,
    //Size must be a power of two at the moment
    pub chunk_hash: [Option<Chunk>; 4096],
}

//TODO: Think about this number
const WORLD_BORDER_CHUNKS: i32 = (i32::MAX/64);

impl World {
    pub fn get_chunk(&mut self, chunk_x: i32, chunk_y: i32, 
                     chunk_z: i32, arena: Option<&mut MemoryArena>) -> Option<&mut Chunk> {

        debug_assert!(chunk_x > -WORLD_BORDER_CHUNKS);
        debug_assert!(chunk_y > -WORLD_BORDER_CHUNKS);
        debug_assert!(chunk_z > -WORLD_BORDER_CHUNKS);
        debug_assert!(chunk_x < WORLD_BORDER_CHUNKS);
        debug_assert!(chunk_y < WORLD_BORDER_CHUNKS);
        debug_assert!(chunk_z < WORLD_BORDER_CHUNKS);

        let hash_value = get_hash(chunk_x, chunk_y, chunk_z);
        let hash_slot = hash_value & (self.chunk_hash.len() - 1) as u32;

        debug_assert!(hash_slot < self.chunk_hash.len() as u32);

        let first_chunk = &mut self.chunk_hash[hash_slot as usize];
        let mut result = None;

        //We have entries in the hashtable so now we need to walk them
        if first_chunk.is_some() {
            let mut chunk_val: &mut Chunk = first_chunk.as_mut().unwrap();
            loop {
                if chunk_x == chunk_val.chunk_x &&
                   chunk_y == chunk_val.chunk_y &&
                   chunk_z == chunk_val.chunk_z {

                    //found it so we can return it!
                    result = Some(chunk_val);
                    break;
                }

                //No more entries in the list
                if chunk_val.next.is_none() {
                    if arena.is_some() {
                        let new_chunk = arena.unwrap().push_struct::<Chunk>();
                        new_chunk.first_block = Default::default();
                        new_chunk.chunk_x = chunk_x;
                        new_chunk.chunk_y = chunk_y;
                        new_chunk.chunk_z = chunk_z;
                        new_chunk.next = None;
                        result = Some(new_chunk);
                        chunk_val.next = Some(new_chunk as *mut Chunk);
                    }
                    break;
                } else {
                    chunk_val = unsafe { mem::transmute(chunk_val.next.unwrap()) };
                }
            }
        } else if arena.is_some() {
            *first_chunk = Some(Chunk {
                    first_block: Default::default(),
                    chunk_x: chunk_x,
                    chunk_y: chunk_y,
                    chunk_z: chunk_z,
                    next: None,
            });
            result = Some(first_chunk.as_mut().unwrap());
        }
        
        result
    }
}

pub struct EntityBlock {
    pub e_count: u32,
    pub lf_entities: [Option<u32>; 16],

    pub next: Option<*mut EntityBlock>,
}

impl Default for EntityBlock {
    fn default() -> EntityBlock {
        EntityBlock {
            e_count: 0,
            lf_entities: [Default::default(); 16],

            next: None,
        }
    }
}

pub struct Chunk {
    pub first_block: EntityBlock,

    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_z: i32,

    pub next: Option<*mut Chunk>,
}

#[derive(Copy, Default)]
//TODO: add a reference to the world here so we don't have to pass it for 
//move calculations?
pub struct WorldPosition {
    pub tile_x: i32,
    pub tile_y: i32,
    pub tile_z: i32,

    //Tile relative
    pub offset: V2f,
}

impl WorldPosition {
    pub fn centered_pos(tile_x: i32, tile_y: i32, tile_z: i32) -> WorldPosition {
        WorldPosition {
            tile_x: tile_x,
            tile_y: tile_y,
            tile_z: tile_z,

            offset: V2f{ x: 0.0, y: 0.0 },
        }
    }
}

pub fn canonicalize_coord(world: &World, tile: &mut i32, tile_offset: &mut f32) {

        let offset = (*tile_offset / world.tile_side_meters).round();

        let new_tile = *tile + offset as i32;
        *tile = new_tile;

        *tile_offset -= offset * world.tile_side_meters;
        //TODO: the rounding makes problems for us here. we might round back
        //to the tile we came from so the assertion fires
        //subtract epsilon when tile_offset > world.tile_side_meters?
        //TODO: reinstate the macro!
        //debug_assert!(*tile_offset >= -0.5 * world.tile_side_meters, "tile_offset {}", *tile_offset);
//        debug_assert!(*tile_offset <= 0.5 * world.tile_side_meters, 
//                      "tile_offset {:.10}\n checked against {:.10}",
//                      *tile_offset, 0.5 * world.tile_side_meters);
}

//TODO: Caution when wrapping around the difference gets too large to represent
//exactly can be 31 bits difference and we can only represent 24 (float)
pub struct WorldDifference {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

//TODO: This function does not cope well if we are in the middle of the world
//because of arithmetic underflow. Needs revision!!
pub fn subtract(world: &World, a: &WorldPosition, b: &WorldPosition) -> WorldDifference {
    let d_tile_x = world.tile_side_meters * (a.tile_x as i32 - b.tile_x as i32) as f32;
    let d_tile_y = world.tile_side_meters * (a.tile_y as i32 - b.tile_y as i32) as f32;
    let d_tile_z = world.tile_side_meters * (a.tile_z as i32 - b.tile_z as i32) as f32;

    WorldDifference {
        dx: d_tile_x + a.offset.x - b.offset.x, 
        dy: d_tile_y + a.offset.y - b.offset.y,
        dz: d_tile_z,
    }
}
