use std::{num::NonZeroU32, rc::Rc};

use winit::window::Window;

pub fn draw_background(
    surface: &mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
    width: u32,
    height: u32,
) {
    if width == 0 || height == 0 {
        return;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(_) => return,
    };

    buffer.fill(0x00000000);

    let rect_width = 373;
    let rect_height = 150;

    let start_x = (width as i32 - rect_width) / 2;
    let start_y = (height as i32 - rect_height) / 2;

    for y in 0..rect_height {
        let draw_y = start_y + y;
        if draw_y < 0 || draw_y >= height as i32 {
            continue;
        }

        for x in 0..rect_width {
            let draw_x = start_x + x;
            if draw_x < 0 || draw_x >= width as i32 {
                continue;
            }

            let color = if x % 7 < 2 {
                0x7FFF0000 // red separator (50% opacity)
            } else {
                0x7F00FF00 // green week (50% opacity)
            };

            buffer[(draw_y as u32 * width + draw_x as u32) as usize] = color;
        }
    }

    let _ = buffer.present();
}

pub fn resize_surface(
    surface: &mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
    width: u32,
    height: u32,
) {
    if width == 0 || height == 0 {
        return;
    }
    if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
        let _ = surface.resize(w, h);
    }
}
