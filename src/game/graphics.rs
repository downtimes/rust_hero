use common::VideoBuffer;

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

