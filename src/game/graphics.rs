use common::{VideoBuffer, ThreadContext};
use common::PlatformReadEntireFileT;
use std::mem;
use std::slice;

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

//TODO: see how to do this crazy blit with iterators to be more idiomatic rust!
//Something with iter.zip!
pub fn draw_bitmap(buffer: &mut VideoBuffer, bitmap: &Bitmap, x: f32, y: f32) {
    let mut min_y = y.round() as isize;
    let mut min_x = x.round() as isize;
    let mut max_x = (x + bitmap.width as f32).round() as isize;
    let mut max_y = (y + bitmap.height as f32).round() as isize;

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


    let bitmap_offset = bitmap.width as isize * (bitmap.height as isize - 1);
    let buffer_offset = min_y * buffer.pitch as isize + min_x; 

    //TODO: Sourcerow is wrong for clipping!
    let mut source_row = unsafe { bitmap.memory.as_ptr().offset(bitmap_offset) };
    let mut dest_row = unsafe { buffer.memory.as_mut_ptr().offset(buffer_offset) };
    for _ in min_y..max_y {
        let mut dest = dest_row;
        let mut source = source_row;

        for _ in min_x..max_x {
            unsafe { 
                *dest = *source;
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

#[cfg(not(ndebug))]
//Note: This function only loads a specific fileformat and is not generic.
//only bottom up AABBGGRR Bitmaps
pub fn debug_load_bitmap(read_func: PlatformReadEntireFileT, context: &ThreadContext,
                   file_name: &str) -> Option<Bitmap<'static>> {
    
    //Note: Bitmap byteorder is AA BB GG RR first row is bottom line
    let file = read_func(context, file_name);
    if let Ok(result) = file {
        let bytes_per_pixel = 4;
        let header: &BitmapHeader = unsafe { mem::transmute(result.contents) };

        let pixels = 
            unsafe { slice::from_raw_parts_mut(
                        result.contents.offset(header.bitmap_offset as isize) as *mut u8, 
                        (header.width * header.height * bytes_per_pixel) as usize) 
                    };

        //Shift alpha from bottom byte to top
        for pixel in pixels.chunks_mut(bytes_per_pixel as usize) {
            let c0 = pixel[0] as u32;
            let c1 = pixel[1] as u32;
            let c2 = pixel[2] as u32;
            let c3 = pixel[3] as u32;

            let pixel_ptr = pixel.as_ptr() as *mut u32;
            unsafe { *pixel_ptr = (c0 << 24) | (c3 << 16) | (c2 << 8) | c1; }
        }

        //Reinterpret the memory as u32
        let result = unsafe { slice::from_raw_parts_mut(
                                    pixels.as_mut_ptr() as *mut u32, 
                                    pixels.len() / bytes_per_pixel as usize) 
                            };

        Some(Bitmap{
            width: header.width as u32,
            height: header.height as u32,
            memory: result,
        })
    } else {
        None
    }
}

