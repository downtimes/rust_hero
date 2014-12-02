
pub struct Buffer<'a> {
    pub memory: &'a mut [u8],
    pub width: uint,
    pub height: uint,
    pub pitch: uint,
}

fn render_weird_gradient(buffer: &mut Buffer, green_offset: int, blue_offset: int) {

    let mut row = buffer.memory.as_mut_ptr();
    for y in range(0, buffer.height) {
        let mut pixel = row as *mut u32;
        for x in range(0, buffer.width) {
            let green_color = (x as int + green_offset) as u8;
            let blue_color = (y as int + blue_offset) as u8;
            unsafe {
                *pixel = (green_color as u32) << 8 | blue_color as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe { row = row.offset(buffer.pitch as int); }
    }

    /*for (y, chunk) in buffer.memory.chunks_mut(buffer.pitch).enumerate() {
        for (x, color) in chunk.iter_mut().enumerate() {
            //ist extrem langsam gerade
            if x >= buffer.width * 4 {
                break;
            }

            match x % 4 {
                0 => *color = ((x/4) as int + blue_offset) as u8,  
                1 => *color = (y as int + green_offset) as u8,
                _ => (),
            }
        }
    }*/
}


pub fn game_update_and_render(buffer: &mut Buffer, green_offset: int, blue_offset: int) {
    render_weird_gradient(buffer, green_offset, blue_offset);
}
