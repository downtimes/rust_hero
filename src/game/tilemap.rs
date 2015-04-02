use std::num::wrapping::OverflowingOps;
use std::mem;
use std::i32;

use super::memory::MemoryArena;
use super::math::V2f;

pub struct TileMap {
    pub tile_side_meters: f32,

    pub chunk_shift: u32,
    pub chunk_mask: i32,
    pub chunk_dim: usize,
    //Size must be a power of two at the moment
    pub tilechunk_hash: [Option<TileChunk>; 4096],
}

//TODO: Better hash function for our use case
fn get_hash(tile_chunk_x: i32, tile_chunk_y: i32, tile_chunk_z: i32) -> u32 {
    ((tile_chunk_x.overflowing_mul(19).0)
    .overflowing_add((tile_chunk_y.overflowing_mul(7)).0).0)
    .overflowing_add((tile_chunk_z.overflowing_mul(3)).0).0 as u32
}

//TODO: Think about this number
const WORLD_BORDER_CHUNKS: i32 = (i32::MAX/64);

impl TileMap {
    pub fn get_tilechunk(&mut self, tilechunk_x: i32, tilechunk_y: i32, 
                         tilechunk_z: i32, arena: Option<&mut MemoryArena>) -> Option<&mut TileChunk> {

        debug_assert!(tilechunk_x > -WORLD_BORDER_CHUNKS);
        debug_assert!(tilechunk_y > -WORLD_BORDER_CHUNKS);
        debug_assert!(tilechunk_z > -WORLD_BORDER_CHUNKS);
        debug_assert!(tilechunk_x < WORLD_BORDER_CHUNKS);
        debug_assert!(tilechunk_y < WORLD_BORDER_CHUNKS);
        debug_assert!(tilechunk_z < WORLD_BORDER_CHUNKS);

        let hash_value = get_hash(tilechunk_x, tilechunk_y, tilechunk_z);
        let hash_slot = hash_value & (self.tilechunk_hash.len() - 1) as u32;

        debug_assert!(hash_slot < self.tilechunk_hash.len() as u32);

        let first_chunk = &mut self.tilechunk_hash[hash_slot as usize];
        let mut result = None;

        //We have entries in the hashtable so now we need to walk them
        if first_chunk.is_some() {
            let mut chunk_val: &mut TileChunk = first_chunk.as_mut().unwrap();
            loop {
                if tilechunk_x == chunk_val.chunk_x &&
                   tilechunk_y == chunk_val.chunk_y &&
                   tilechunk_z == chunk_val.chunk_z {

                    //found it so we can return it!
                    result = Some(chunk_val);
                    break;
                }

                //No more entries in the list
                if chunk_val.next.is_none() {
                    if arena.is_some() {
                        let new_chunk = arena.unwrap().push_struct::<TileChunk>();
                        new_chunk.chunk_x = tilechunk_x;
                        new_chunk.chunk_y = tilechunk_y;
                        new_chunk.chunk_z = tilechunk_z;
                        new_chunk.next = None;
                        result = Some(new_chunk);
                        chunk_val.next = Some(new_chunk as *mut TileChunk);
                    }
                    break;
                } else {
                    chunk_val = unsafe { mem::transmute(chunk_val.next.unwrap()) };
                }
            }
        } else if arena.is_some() {
            *first_chunk = Some(TileChunk {
                    tiles: arena.unwrap().push_slice(0),
                    chunk_x: tilechunk_x,
                    chunk_y: tilechunk_y,
                    chunk_z: tilechunk_z,
                    next: None,
            });
            result = Some(first_chunk.as_mut().unwrap());
        }
        
        result
    }

    pub fn set_tile_value(&mut self, memory: &mut MemoryArena,
                          tile_x: i32, tile_y: i32, tile_z: i32,
                          value: u32)  {

        let chunk_pos = get_chunk_position(self, tile_x, tile_y, tile_z);
        
        let chunk_dim = self.chunk_dim;
        let tilechunk = self.get_tilechunk(chunk_pos.tilechunk_x, chunk_pos.tilechunk_y,
                                        chunk_pos.tilechunk_z, Some(memory));
        
        let chunk = tilechunk.unwrap();
        if chunk.tiles.len() == 0 {
            chunk.tiles = memory.push_slice(chunk_dim * chunk_dim);
        }

        chunk.set_tile_value(chunk_pos.tile_x, chunk_pos.tile_y, value, chunk_dim);
    }

    pub fn get_tile_value(&mut self, tile_x: i32, tile_y: i32, 
                          tile_z: i32) -> Option<u32> {

        let chunk_pos = get_chunk_position(self, tile_x, tile_y, tile_z);
        let chunk_dim = self.chunk_dim;
        let tilechunk = self.get_tilechunk(chunk_pos.tilechunk_x,
                                           chunk_pos.tilechunk_y,
                                           chunk_pos.tilechunk_z, None);

        match tilechunk {
            Some(_) => tilechunk.unwrap().get_tile_value(chunk_pos.tile_x, 
                                                         chunk_pos.tile_y,
                                                         chunk_dim),
            None => None,
        }
    }

    fn get_tile_value_pos(&mut self, pos: &TilemapPosition) -> Option<u32> {
        self.get_tile_value(pos.tile_x, pos.tile_y, pos.tile_z)
    }
}

struct TileChunkPosition {
    tilechunk_x: i32,
    tilechunk_y: i32,
    tilechunk_z: i32,
    
    tile_x: i32,
    tile_y: i32,
}

pub struct TileChunk {
    pub tiles: &'static mut [u32],

    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_z: i32,

    pub next: Option<*mut TileChunk>,
}

#[derive(Copy, Default)]
//TODO: add a reference to a Tilemap here so we don't have to pass it for 
//move calculations?
pub struct TilemapPosition {
    pub tile_x: i32,
    pub tile_y: i32,
    pub tile_z: i32,

    //Tile relative
    pub offset: V2f,
}

impl TilemapPosition {
    pub fn centered_pos(tile_x: i32, tile_y: i32, tile_z: i32) -> TilemapPosition {
        TilemapPosition {
            tile_x: tile_x,
            tile_y: tile_y,
            tile_z: tile_z,

            offset: V2f{ x: 0.0, y: 0.0 },
        }
    }
}

pub fn canonicalize_coord(tilemap: &TileMap, tile: &mut i32, tile_offset: &mut f32) {

        let offset = (*tile_offset / tilemap.tile_side_meters).round();

        let new_tile = *tile + offset as i32;
        *tile = new_tile;

        *tile_offset -= offset * tilemap.tile_side_meters;
        //TODO: the rounding makes problems for us here. we might round back
        //to the tile we came from so the assertion fires
        //subtract epsilon when tile_offset > tilemap.tile_side_meters?
        //TODO: reinstate the macro!
        //debug_assert!(*tile_offset >= -0.5 * tilemap.tile_side_meters, "tile_offset {}", *tile_offset);
//        debug_assert!(*tile_offset <= 0.5 * tilemap.tile_side_meters, 
//                      "tile_offset {:.10}\n checked against {:.10}",
//                      *tile_offset, 0.5 * tilemap.tile_side_meters);
}

impl TilemapPosition {
    pub fn recanonicalize(&mut self, tilemap: &TileMap) {

        canonicalize_coord(tilemap, &mut self.tile_x, &mut self.offset.x);
        canonicalize_coord(tilemap, &mut self.tile_y, &mut self.offset.y);
    }

    pub fn offset(self, d: V2f, tilemap: &TileMap) -> TilemapPosition {
        let mut res = self; 
        res.offset = res.offset + d;
        res.recanonicalize(tilemap);
        res
    }
}

//TODO: Caution when wrapping around the difference gets too large to represent
//exactly can be 31 bits difference and we can only represent 24 (float)
pub struct TilemapDifference {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

//TODO: This function does not cope well if we are in the middle of the tilemap
//because of arithmetic underflow. Needs revision!!
pub fn subtract(tilemap: &TileMap, a: &TilemapPosition, b: &TilemapPosition) -> TilemapDifference {
    let d_tile_x = tilemap.tile_side_meters * (a.tile_x as i32 - b.tile_x as i32) as f32;
    let d_tile_y = tilemap.tile_side_meters * (a.tile_y as i32 - b.tile_y as i32) as f32;
    let d_tile_z = tilemap.tile_side_meters * (a.tile_z as i32 - b.tile_z as i32) as f32;

    TilemapDifference {
        dx: d_tile_x + a.offset.x - b.offset.x, 
        dy: d_tile_y + a.offset.y - b.offset.y,
        dz: d_tile_z,
    }
}

pub fn is_tile_value_empty(value: u32) -> bool {
    value != 1
}

#[allow(dead_code)]
pub fn is_tilemap_point_empty(tilemap: &mut TileMap, position: &TilemapPosition) -> bool {
    match tilemap.get_tile_value(position.tile_x, position.tile_y, position.tile_z) {
        Some(value) => is_tile_value_empty(value),
        None => false,
    }
}


impl TileChunk {
    fn get_tile_value(&self, tile_x: i32, tile_y: i32, chunk_dim: usize) -> Option<u32> {
        let index = tile_y as usize * chunk_dim + tile_x as usize;
        if index >= self.tiles.len() {
            None
        } else {
            Some(self.tiles[index])
        }
    }

    fn set_tile_value(&mut self, tile_x: i32, tile_y: i32,
                      value: u32, chunk_dim: usize) {
        let index = tile_y as usize * chunk_dim + tile_x as usize;
        self.tiles[index] = value;
    }
}


fn get_chunk_position(tile_map: &TileMap, tile_x: i32, tile_y: i32,
                      tile_z: i32) -> TileChunkPosition {
    TileChunkPosition {
        tilechunk_x: tile_x >> tile_map.chunk_shift,
        tilechunk_y: tile_y >> tile_map.chunk_shift,
        tilechunk_z: tile_z,
        tile_x: tile_x & tile_map.chunk_mask,
        tile_y: tile_y & tile_map.chunk_mask,
    }
}

pub fn on_same_tile(position1: &TilemapPosition, position2: &TilemapPosition) -> bool {
    position1.tile_x == position2.tile_x && position1.tile_y == position2.tile_y
    && position1.tile_z == position2.tile_z
}
