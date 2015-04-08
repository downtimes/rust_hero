use std::num::Wrapping as w;
use std::mem;
use std::i32;
use std::default::Default;

use super::memory::MemoryArena;
use super::math::V2;

//TODO: Think about this number
const WORLD_BORDER_CHUNKS: i32 = (i32::MAX/64);
const TILES_PER_CHUNK: i32 = 16;

pub fn world_pos_from_tile(world: &World, tile_x: i32, tile_y: i32, tile_z: i32) -> WorldPosition {
    //TODO: move to 3D
    let mut chunk_x = tile_x / TILES_PER_CHUNK;
    let mut chunk_y = tile_y / TILES_PER_CHUNK;

    if tile_x < 0 {
        chunk_x -= 1;
    }

    if tile_y < 0 {
        chunk_y -= 1;
    }

    WorldPosition {
        chunk_x: chunk_x,
        chunk_y: chunk_y,
        chunk_z: tile_z,
        offset: V2{x: (tile_x % TILES_PER_CHUNK) as f32 
                        * world.tile_side_meters,
                   y: (tile_y % TILES_PER_CHUNK) as f32 
                        * world.tile_side_meters},
    }
}

//TODO: Better hash function for our use case
fn get_hash(tile_chunk_x: i32, tile_chunk_y: i32, tile_chunk_z: i32) -> u32 {
    let x = w(tile_chunk_x);
    let y = w(tile_chunk_y);
    let z = w(tile_chunk_z);


    let res = x * w(19) + y * w(7) + z * w(3);
    res.0 as u32
}

pub fn map_into_world_space(world: &World, world_pos: &WorldPosition, rel_pos: &V2<f32>) -> WorldPosition {

    let mut pos = *world_pos;
    pos.offset = pos.offset + *rel_pos;
    canonicalize_coord(world, &mut pos.chunk_x, &mut pos.offset.x);
    canonicalize_coord(world, &mut pos.chunk_y, &mut pos.offset.y);
    pos
}


//TODO: Implement a way to iterate over chunks spatialy
//with a simple iter(mincorner, maxcorner, z) or something
pub struct World {
    pub tile_side_meters: f32,
    pub chunk_side_meters: f32,

    //Size must be a power of two at the moment
    pub chunk_hash: [Option<Chunk>; 4096],

    pub first_free: Option<*mut EntityBlock>,
}

pub struct IterChunk {
    world: &'static mut World,
    min_x: i32,
    curr_x: i32,
    curr_y: i32,
    max_x: i32,
    max_y: i32,
    z: i32,
}

impl Iterator for IterChunk {
    type Item = &'static Chunk;

    fn next(&mut self) -> Option<&'static Chunk> {
        if self.curr_x == self.max_x && self.curr_y == self.max_y {
            None
        } else {
            let ch = self.world.get_chunk(self.curr_x, self.curr_y, self.z, None);
            self.curr_x += 1;
            if self.curr_x == self.max_x && self.curr_y != self.max_y {
                self.curr_y += 1;
                self.curr_x = self.min_x;
            }

            //if on this position there is no chunk iterate to the next position
            //recursively
            match ch {
                None => self.next(),
                Some(chunk) => Some(chunk),
            }
        }
    }
}


impl World {
    pub fn iter_spatially(&mut self, min_p: WorldPosition, max_p: WorldPosition, z: i32) -> IterChunk {
        IterChunk {
            world: unsafe { mem::transmute(self as *mut World) },
            min_x: min_p.chunk_x,
            curr_x: min_p.chunk_x,
            curr_y: min_p.chunk_y,
            max_x: max_p.chunk_x + 1,
            max_y: max_p.chunk_y + 1,
            z: z,
        }
    }

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
                
                //pull entity out of the old slot
                if chunk.is_some() {
                    let ch = chunk.unwrap();

                    let first_block = &mut ch.first_block;

                    //in case the entity is in the first_block
                    //NOTE: This is done seperately to avoid aliasing in the 
                    //following loop because we could alias with our first_block
                    //variable
                    for index in 0..first_block.e_count as usize {
                        if first_block.lf_entities[index] == lf_index {
                            debug_assert!(first_block.e_count > 0);
                            first_block.e_count -= 1;
                            first_block.lf_entities[index] 
                                = first_block.lf_entities[first_block.e_count as usize];
                            maybe_remove_block(self, first_block);
                        }
                    }

                    //in case it is in some of the consecutive blocks
                    'find: for block in first_block.iter_mut().skip(1) {
                        for index in 0..block.e_count as usize {
                            if block.lf_entities[index] == lf_index {
                                debug_assert!(first_block.e_count > 0);
                                first_block.e_count -= 1;
                                block.lf_entities[index] 
                                    = first_block.lf_entities[first_block.e_count as usize];
                                maybe_remove_block(self, first_block);

                                //we have done our work no need to iterate over
                                //any more of the blocks
                                break 'find;
                            }
                        }
                    }
                }

                fn maybe_remove_block(world: &mut World, first_block: &mut EntityBlock) {
                    if first_block.e_count == 0 && first_block.next.is_some() {
                        let next_block: &mut EntityBlock 
                            = unsafe { mem::transmute(first_block.next.unwrap()) };
                        *first_block = *next_block;
                        //put the block in the freelist
                        next_block.next = world.first_free;
                        world.first_free = Some(next_block as *mut EntityBlock);
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
    //Be carefull that you don't get the same chunk two times and modify them.
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

pub struct Iter(Option<*const EntityBlock>);
pub struct IterMut(Option<*mut EntityBlock>);

impl Iterator for Iter {
    type Item = &'static EntityBlock;

    fn next(&mut self) -> Option<&'static EntityBlock> {
        match *self {
            Iter(None) => None,
            Iter(Some(ptr)) => unsafe {
                match (*ptr).next {
                    None => *self = Iter(None),
                    Some(next_ptr) => *self = Iter(Some(next_ptr as *const EntityBlock)),
                }
                Some(mem::transmute(ptr))
            },
        }
    }
}

impl Iterator for IterMut {
    type Item = &'static mut EntityBlock;

    fn next(&mut self) -> Option<&'static mut EntityBlock> {
        match *self {
            IterMut(None) => None,
            IterMut(Some(ptr)) => unsafe {
                *self = IterMut((*ptr).next); 
                Some(mem::transmute(ptr))
            }
        }
    }
}

#[derive(Copy, Clone)]
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

impl EntityBlock {
    pub fn iter(&self) -> Iter {
        Iter(Some(self as *const EntityBlock))
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut(Some(self as *mut EntityBlock))
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

#[derive(Copy, Clone, Default)]
//TODO: add a reference to the world here so we don't have to pass it for 
//move calculations?
pub struct WorldPosition {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_z: i32,

    //chunk relative
    pub offset: V2<f32>,
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

            offset: V2{ x: 0.0, y: 0.0 },
        }
    }
}

fn is_canonical(world: &World, rel: f32) -> bool {
    (rel >= -0.5*world.chunk_side_meters) &&
    (rel <= 0.5*world.chunk_side_meters)
}

fn is_canonical_v(world: &World, offset: V2<f32>) -> bool {
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
