
use super::MemoryArena;

pub struct TileMap<'a> {
    pub tile_side_pixels: u32,
    pub tile_side_meters: f32,

    pub tilechunk_count_x: usize,
    pub tilechunk_count_y: usize,

    pub chunk_shift: u32,
    pub chunk_mask: u32,
    pub tilechunks: &'a mut [TileChunk<'a>],
}

impl<'a> TileMap<'a> {
    pub fn get_tilechunk(&'a self, chunk_pos: &TileChunkPosition) -> Option<&'a TileChunk<'a>> {
        let index = (chunk_pos.tilechunk_y 
                          * self.tilechunk_count_x as u32
                          + chunk_pos.tilechunk_x) as usize;
        if index < self.tilechunks.len() {
            Some(&self.tilechunks[index])
        } else {
            None
        }
    }

    pub fn meters_to_pixels(&self, meters: f32) -> f32 {
        meters * self.tile_side_pixels as f32 / self.tile_side_meters
    }

    pub fn set_tile_value(&mut self, _memory: &mut MemoryArena,
                          tile_x: u32, tile_y: u32, value: u32)  {

        let chunk_pos = get_chunk_position(self, tile_x, tile_y);
        let tilechunk = &mut self.tilechunks[(chunk_pos.tilechunk_y 
                                              * self.tilechunk_count_x as u32
                                              + chunk_pos.tilechunk_x) as usize];

        tilechunk.set_tile_value(chunk_pos.tile_x, chunk_pos.tile_y, value);

    }

    pub fn get_tile_value(&'a self, position: &TilemapPosition) -> u32 {

        let chunk_pos = get_chunk_position(self, position.tile_x, position.tile_y);
        let tilechunk = self.get_tilechunk(&chunk_pos);

        match tilechunk {
            Some(_) => tilechunk.unwrap().get_tile_value(chunk_pos.tile_x, chunk_pos.tile_y),
            None => 0,
        }
    }
}

struct TileChunkPosition {
    tilechunk_x: u32,
    tilechunk_y: u32,
    
    tile_x: u32,
    tile_y: u32,
}

pub struct TileChunk<'a> {
    pub chunk_dim: usize,
    pub tiles: &'a mut [u32]
}

#[derive(Copy)]
pub struct TilemapPosition {
    pub tile_x: u32,
    pub tile_y: u32,

    //Tile relative
    pub offset_x: f32,
    pub offset_y: f32,
}

pub fn canonicalize_coord(tilemap: &TileMap, tile: &mut u32, tile_offset: &mut f32) {

        let offset = (*tile_offset / tilemap.tile_side_meters as f32).round();

        let new_tile = *tile as i32 + offset as i32;
        *tile = new_tile as u32;

        *tile_offset -= offset * tilemap.tile_side_meters;
        //TODO: the rounding makes problems for us here. we might round back
        //to the tile we came from so the assertion fires
        debug_assert!(*tile_offset >= -0.5 * tilemap.tile_side_meters);
        debug_assert!(*tile_offset <= 0.5 * tilemap.tile_side_meters);
}

impl TilemapPosition {
    pub fn recanonicalize(&mut self, tilemap: &TileMap) {

        canonicalize_coord(tilemap, &mut self.tile_x, &mut self.offset_x);
        canonicalize_coord(tilemap, &mut self.tile_y, &mut self.offset_y);
    }
}

pub fn is_tilemap_point_empty<'a>(tilemap: &'a TileMap<'a>, position: &TilemapPosition) -> bool {
    tilemap.get_tile_value(position) == 0
}


impl<'a> TileChunk<'a> {
    fn get_tile_value(&self, tile_x: u32, tile_y: u32) -> u32 {
        let index = tile_y as usize * self.chunk_dim + tile_x as usize;
        if index > self.tiles.len() {
            0
        } else {
            self.tiles[index]
        }
    }

    fn set_tile_value(&mut self, tile_x: u32, tile_y: u32, value: u32) {
        let index = tile_y as usize * self.chunk_dim + tile_x as usize;
        self.tiles[index] = value;
    }
}


fn get_chunk_position(tile_map: &TileMap, tile_x: u32, tile_y: u32) -> TileChunkPosition {
    TileChunkPosition {
        tilechunk_x: tile_x >> tile_map.chunk_shift,
        tilechunk_y: tile_y >> tile_map.chunk_shift,
        tile_x: tile_x & tile_map.chunk_mask,
        tile_y: tile_y & tile_map.chunk_mask,
    }
}
