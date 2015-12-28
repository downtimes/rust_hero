use common::{VideoBuffer, ThreadContext};
use common::PlatformReadEntireFileT;
use std::mem;
use std::slice;

use super::math::V2;

#[derive(Copy, Clone, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn draw_rect(buffer: &mut VideoBuffer,
                 real_min: V2<f32>,
                 real_max: V2<f32>,
                 r: f32,
                 g: f32,
                 b: f32) {

    let mut min_x = real_min.x.round() as isize;
    let mut max_x = real_max.x.round() as isize;
    let mut min_y = real_min.y.round() as isize;
    let mut max_y = real_max.y.round() as isize;

    if min_x < 0 {
        min_x = 0;
    }
    if min_y < 0 {
        min_y = 0;
    }
    if max_x > buffer.width as isize {
        max_x = buffer.width as isize;
    }
    if max_y > buffer.height as isize {
        max_y = buffer.height as isize;
    }

    let width = if min_x < max_x {
        (max_x - min_x) as usize
    } else {
        0
    };
    let height = if min_y < max_y {
        (max_y - min_y) as usize
    } else {
        0
    };

    // Bit pattern: AA RR GG BB
    let color: u32 = (((r * 255.0).round() as u32) << 16) | (((g * 255.0).round() as u32) << 8) |
                     (b * 255.0).round() as u32;

    for row in buffer.memory.chunks_mut(buffer.pitch).skip(min_y as usize).take(height) {
        for pixel in row.iter_mut().skip(min_x as usize).take(width) {
            *pixel = color;
        }
    }
}

// TODO: see how to do this crazy blit with iterators to be more idiomatic rust!
// Something with iter.zip!
pub fn draw_bitmap_alpha(buffer: &mut VideoBuffer,
                         bitmap: &Bitmap,
                         top_left: V2<f32>,
                         alpha: f32) {

    let mut min_y = top_left.y.round() as isize;
    let mut min_x = top_left.x.round() as isize;
    let mut max_x = min_x + bitmap.width as isize;
    let mut max_y = min_y + bitmap.height as isize;

    let mut source_offset_x = 0;
    if min_x < 0 {
        source_offset_x -= min_x;
        min_x = 0;
    }
    let mut source_offset_y = 0;
    if min_y < 0 {
        source_offset_y -= min_y;
        min_y = 0;
    }
    if max_x > buffer.width as isize {
        max_x = buffer.width as isize;
    }
    if max_y > buffer.height as isize {
        max_y = buffer.height as isize;
    }

    let bitmap_offset = bitmap.width as isize * (bitmap.height as isize - 1) -
                        bitmap.width as isize * source_offset_y +
                        source_offset_x;
    let buffer_offset = min_y * buffer.pitch as isize + min_x;

    let mut source_row = unsafe { bitmap.memory.as_ptr().offset(bitmap_offset) };
    let mut dest_row = unsafe { buffer.memory.as_mut_ptr().offset(buffer_offset) };
    for _ in min_y..max_y {
        let mut dest = dest_row;
        let mut source = source_row;

        for _ in min_x..max_x {
            unsafe {
                let a = ((*source >> 24) as f32 / 255.0) * alpha;
                let sr = (*source >> 16) & 0xFF;
                let sg = (*source >> 8) & 0xFF;
                let sb = *source & 0xFF;

                let dr = (*dest >> 16) & 0xFF;
                let dg = (*dest >> 8) & 0xFF;
                let db = *dest & 0xFF;

                // Lerp between source and dest
                let r = ((1.0 - a) * dr as f32 + a * sr as f32).round() as u32;
                let g = ((1.0 - a) * dg as f32 + a * sg as f32).round() as u32;
                let b = ((1.0 - a) * db as f32 + a * sb as f32).round() as u32;

                *dest = (r << 16) | (g << 8) | b;

                source = source.offset(1);
                dest = dest.offset(1);
            }
        }

        unsafe {
            dest_row = dest_row.offset(buffer.pitch as isize);
            source_row = source_row.offset(-(bitmap.width as isize));
        }
    }
}

#[allow(dead_code)]
pub fn draw_bitmap(buffer: &mut VideoBuffer, bitmap: &Bitmap, x: f32, y: f32) {
    draw_bitmap_alpha(buffer, bitmap, V2 { x: x, y: y }, 1.0);
}

#[repr(C, packed)]
struct BitmapHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    bitmap_offset: u32,
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    size_of_bitmap: u32,
    horz_resolution: i32,
    vert_resolution: i32,
    colors_used: u32,
    colors_important: u32,

    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

pub struct Bitmap<'a> {
    width: u32,
    height: u32,
    memory: &'a [u32],
}

fn rotate_left(value: u32, mut amount: i32) -> u32 {
    if amount < 0 {
        amount += 32;
    }

    debug_assert!(amount < 25 && amount >= 0);
    value.rotate_left(amount as u32)
}

#[cfg(not(ndebug))]
// Note: This function only loads a specific fileformat and is not generic.
// only bottom up AABBGGRR Bitmaps
pub fn debug_load_bitmap(read_func: PlatformReadEntireFileT,
                         context: &ThreadContext,
                         file_name: &str)
                         -> Option<Bitmap<'static>> {

    // Note: Bitmap byteorder is determined by the header. bottom up
    let file = read_func(context, file_name);
    if let Ok(result) = file {
        let header: &BitmapHeader = unsafe { mem::transmute(result.contents) };

        debug_assert!(header.compression == 3);

        let pixels = unsafe {
            slice::from_raw_parts_mut(
                        result.contents.offset(header.bitmap_offset as isize) as *mut u32,
                        (header.width * header.height) as usize)
        };
        let red_mask = header.red_mask;
        let blue_mask = header.blue_mask;
        let green_mask = header.green_mask;
        let alpha_mask = !(red_mask | green_mask | blue_mask);

        let alpha_shift = 24 - alpha_mask.trailing_zeros() as i32;
        let red_shift = 16 - red_mask.trailing_zeros() as i32;
        let green_shift = 8 - green_mask.trailing_zeros() as i32;
        let blue_shift = 0 - blue_mask.trailing_zeros() as i32;

        // Shift bits according to masks
        for pixel in pixels.iter_mut() {
            *pixel = rotate_left(*pixel & red_mask, red_shift) |
                     rotate_left(*pixel & green_mask, green_shift) |
                     rotate_left(*pixel & blue_mask, blue_shift) |
                     rotate_left(*pixel & alpha_mask, alpha_shift)
        }

        Some(Bitmap {
            width: header.width as u32,
            height: header.height as u32,
            memory: pixels,
        })
    } else {
        None
    }
}
