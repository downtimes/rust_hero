use std::num::wrapping::OverflowingOps;
use std::mem;
use std::i32;
use std::default::Default;

use super::memory::MemoryArena;
use super::math::V2f;

//TODO: Think about this number
const WORLD_BORDER_CHUNKS: i32 = (i32::MAX/64);
const TILES_PER_CHUNK: i32 = 16;

pub fn world_pos_from_tile(world: &World, tile_x: i32, tile_y: i32, tile_z: i32) -> WorldPosition {
    //TODO: move to 3D
    let chunk_x = tile_x / TILES_PER_CHUNK;
    let chunk_y = tile_y / TILES_PER_CHUNK;
    WorldPosition {
        chunk_x: chunk_x,
        chunk_y: chunk_y,
        chunk_z: tile_z,
        offset: V2f{x: (tile_x - (chunk_x * TILES_PER_CHUNK)) as f32 * world.tile_side_meters,
                    y: (tile_y - (chunk_y * TILES_PER_CHUNK)) as f32 * world.tile_side_meters},
    }
}

//TODO: Better hash function for our use case
fn get_hash(tile_chunk_x: i32, tile_chunk_y: i32, tile_chunk_z: i32) -> u32 {
    ((tile_chunk_x.overflowing_mul(19).0)
    .overflowing_add((tile_chunk_y.overflowing_mul(7)).0).0)
    .overflowing_add((tile_chunk_z.overflowing_mul(3)).0).0 as u32
}

pub fn map_into_world_space(world: &World, world_pos: &WorldPosition, rel_pos: &V2f) -> WorldPosition {

    let mut pos = *world_pos;
    pos.offset = pos.offset + *rel_pos;
    canonicalize_coord(world, &mut pos.chunk_x, &mut pos.offset.x);
    canonicalize_coord(world, &mut pos.chunk_y, &mut pos.offset.y);
    pos
}

pub struct World {
    pub tile_side_meters: f32,
    pub chunk_side_meters: f32,

    //Size must be a power of two at the moment
    pub chunk_hash: [Option<Chunk>; 4096],

    pub first_free: Option<*mut EntityBlock>,
}


impl World {
    pub fn initialize(&mut self) {
        self.tile_side_meters = 1.4;
        self.chunk_side_meters = TILES_PER_CHUNK as f32 * 1.4;
    }

    pub fn change_entity_location(&mut self, lf_index: u32,
                                  old_pos: Option<&WorldPosition>, new_pos: &WorldPosition,
                                  arena: &mut MemoryArena) {
        if old_pos.is_some() && are_in_same_chunk(self, old_pos.unwrap(), new_pos) {
            //Do nothing because we are already in the right spot 
        } else {
            if old_pos.is_some() {
                let pos = old_pos.unwrap();
                let chunk = self.get_chunk(pos.chunk_x, pos.chunk_y, pos.chunk_z, None);
                debug_assert!(chunk.is_some());
                
                let mut move_e_index = 0;
                let mut block_lf_index = 0;
                let mut block_number: i32 = 0;
                if chunk.is_some() {
                    let ch = chunk.unwrap();

                    //We acutally copy every EntityBlock in this algorithm so that
                    //we don't get aliasing. Look for better alternatives to implement this
                    //more efficiently!
                    //pull entity out of the old slot
                    let mut read_block = ch.first_block;
                    let first_block = &mut ch.first_block;
                    'find: loop {

                        for index in 0..read_block.e_count as usize {
                            if read_block.lf_entities[index] == lf_index {
                                debug_assert!(first_block.e_count > 0);
                                block_lf_index = index;
                                first_block.e_count -= 1;
                                move_e_index = first_block.lf_entities[first_block.e_count as usize];
                                if first_block.e_count == 0 {
                                    if first_block.next.is_some() {
                                        block_number -= 1;
                                        let next_block: &mut EntityBlock 
                                            = unsafe { mem::transmute(first_block.next.unwrap()) };
                                        *first_block = *next_block;
                                        //put the block in the freelist
                                        next_block.next = self.first_free;
                                        self.first_free = Some(next_block as *mut EntityBlock);
                                    }
                                }

                                break 'find;
                            }
                        }

                        if read_block.next.is_none() {
                            break 'find;
                        } else {
                            block_number += 1;
                            read_block = *unsafe { mem::transmute::<*mut EntityBlock,
                                                                    &mut EntityBlock>
                                                    (read_block.next.unwrap()) };
                        }
                    }

                    //After the fact walk again and write the copied value to the now empty slot
                    if block_number >= 0 {
                        let mut block = first_block;
                        for _ in 0..block_number {
                            block = unsafe { mem::transmute(block.next.unwrap()) };
                        }
                        block.lf_entities[block_lf_index] = move_e_index;
                    }
                }
            }

            //Now start inserting the entity in the new Block
            let chunk = self.get_chunk(new_pos.chunk_x, new_pos.chunk_y, new_pos.chunk_z,
                                       Some(arena)).unwrap();
            let block = &mut chunk.first_block;
            if block.e_count as usize == block.lf_entities.len() {
                //Make new block to store it
                let old_block = 
                    if self.first_free.is_some() {
                        let ptr = self.first_free.unwrap();
                        self.first_free = unsafe { (*ptr).next };
                        unsafe { mem::transmute(ptr) }
                    } else {
                        arena.push_struct::<EntityBlock>()
                    };
                *old_block = *block;
                block.next = Some(old_block as *mut EntityBlock);
                block.e_count = 0;
            }
            debug_assert!((block.e_count as usize) < block.lf_entities.len());
            block.lf_entities[block.e_count as usize] = lf_index;
            block.e_count += 1;
        }
    }

    //NOTE: THIS FUNCTION DECOUPLES THE LIFETIME OF THE CHUNK FROM THE GAMESTATE!
    //Be carefull that you don't get the same chunk two times and modifie them
    //it's asumed that you will not alias.
    pub fn get_chunk(&mut self, chunk_x: i32, chunk_y: i32, 
                     chunk_z: i32, arena: Option<&mut MemoryArena>) -> Option<&'static mut Chunk> {

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
                    result = Some(chunk_val.decouple());
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
                        result = Some(new_chunk.decouple());
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
            result = Some(first_chunk.as_mut().unwrap().decouple());
        }
        
        result
    }
}

#[derive(Copy)]
pub struct EntityBlock {
    pub e_count: u32,
    pub lf_entities: [u32; 16],

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

impl Chunk {
    pub fn decouple(&mut self) -> &'static mut Chunk {
        unsafe { 
            mem::transmute(self as *mut Chunk) 
        }
    }
}

#[derive(Copy, Default)]
//TODO: add a reference to the world here so we don't have to pass it for 
//move calculations?
pub struct WorldPosition {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_z: i32,

    //chunk relative
    pub offset: V2f,
}

pub fn are_in_same_chunk(world: &World, pos1: &WorldPosition, pos2: &WorldPosition) -> bool {
    debug_assert!(is_canonical_v(world, pos1.offset) && is_canonical_v(world, pos2.offset));
    pos1.chunk_x == pos2.chunk_x &&
    pos1.chunk_y == pos2.chunk_y &&
    pos1.chunk_z == pos2.chunk_z
}

impl WorldPosition {
    pub fn centered_chunk_pos(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> WorldPosition {
        WorldPosition {
            chunk_x: chunk_x,
            chunk_y: chunk_y,
            chunk_z: chunk_z,

            offset: V2f{ x: 0.0, y: 0.0 },
        }
    }
}

fn is_canonical(world: &World, rel: f32) -> bool {
    (rel >= -0.5*world.chunk_side_meters) &&
    (rel <= 0.5*world.chunk_side_meters)
}

fn is_canonical_v(world: &World, offset: V2f) -> bool {
    is_canonical(world, offset.x) &&
    is_canonical(world, offset.y) 
}

pub fn canonicalize_coord(world: &World, tile: &mut i32, tile_offset: &mut f32) {

        let offset = (*tile_offset / world.chunk_side_meters).round();

        let new_tile = *tile + offset as i32;
        *tile = new_tile;

        *tile_offset -= offset * world.chunk_side_meters;
        debug_assert!(is_canonical(world, *tile_offset));
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
    let d_tile_x = world.chunk_side_meters * (a.chunk_x as i32 - b.chunk_x as i32) as f32;
    let d_tile_y = world.chunk_side_meters * (a.chunk_y as i32 - b.chunk_y as i32) as f32;
    let d_tile_z = world.chunk_side_meters * (a.chunk_z as i32 - b.chunk_z as i32) as f32;

    WorldDifference {
        dx: d_tile_x + a.offset.x - b.offset.x, 
        dy: d_tile_y + a.offset.y - b.offset.y,
        dz: d_tile_z,
    }
}
