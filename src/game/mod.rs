//The public interface of the game
pub struct Buffer<'a> {
    pub memory: &'a mut [u32],
    pub width: uint,
    pub height: uint,
    pub pitch: uint,
}

pub fn game_update_and_render(buffer: &mut Buffer, green_offset: int, blue_offset: int) {
    render_weird_gradient(buffer, green_offset, blue_offset);
}
//End of the public interface



fn render_weird_gradient(buffer: &mut Buffer, green_offset: int, blue_offset: int) {

    //This unsafe variant is only in Debug mode sligthly faster than the safe method
    //therefor we are using the safe method!
    /*let mut row = buffer.memory.as_mut_ptr() as *mut u8;
    for y in range(0, buffer.height) {
        let mut pixel = row as *mut u32;
        for x in range(0, buffer.width) {
            let green_color = (y as int + green_offset) as u8;
            let blue_color = (x as int + blue_offset) as u8;
            unsafe {
                *pixel = (green_color as u32) << 8 | blue_color as u32;
                pixel = pixel.offset(1);
            }
        }
        unsafe { row = row.offset((buffer.pitch * 4) as int); }
    }*/

    for (y, row) in buffer.memory.chunks_mut(buffer.pitch).enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            //if we have padding we don't want to write farther out than 
            //the width of our image
            if x >= buffer.width {
                break;
            }
            let green_color = (y as int + green_offset) as u8;
            let blue_color = (x as int + blue_offset) as u8;
            *pixel = (green_color as u32) << 8 | blue_color as u32;
        }
    }
}


