use common::{VideoBuffer, ThreadContext};
use common::PlatformReadEntireFileT;
use std::mem;

pub fn draw_rect(buffer: &mut VideoBuffer, real_min_x: f32, real_min_y: f32, 
                 real_max_x: f32, real_max_y: f32, 
                 r: f32, g: f32, b: f32) {
    let mut min_x = real_min_x.round() as isize;
    let mut max_x = real_max_x.round() as isize;
    let mut min_y = real_min_y.round() as isize;
    let mut max_y = real_max_y.round() as isize;

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

    //Bit pattern: AA RR GG BB
    let color: u32 = (((r * 255.0).round() as u32) << 16) |
                     (((g * 255.0).round() as u32) << 8) | 
                     (b * 255.0).round() as u32;

    for row in buffer.memory.chunks_mut(buffer.pitch).skip(min_y as usize).take(height) {
        for pixel in row.iter_mut().skip(min_x as usize).take(width) {
           *pixel = color; 
        }
    }
}

#[repr(C, packed)]
struct BitmapHeader {
    file_type: u16,
    file_size: u32,
    _reserved1: u16,
    _reserved2: u16,
    bitmap_offset: u32,
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bytes_per_pixel: u16,
}

pub struct Bitmap;

#[cfg(not(ndebug))]
pub fn debug_load_bitmap(read_func: PlatformReadEntireFileT, context: &ThreadContext,
                   file_name: &str) -> Option<Bitmap> {
    let file = read_func(context, file_name);
    if let Ok(result) = file {
        let header: &BitmapHeader = unsafe { mem::transmute(result.contents) };
        let pixels = unsafe { result.contents.offset(header.bitmap_offset as isize) };
    }

    None
}

