use std::{num::NonZeroU32, rc::Rc};

use winit::window::Window;

use crate::OPACITY_PERCENT;

fn apply_transparency(r: u8, g: u8, b: u8, opacity: f32) -> u32 {
    let alpha = (opacity * 255.0).round() as u8;
    let r = ((r as f32) * opacity).round() as u8;
    let g = ((g as f32) * opacity).round() as u8;
    let b = ((b as f32) * opacity).round() as u8;

    ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

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

    let rect_width = 1119;
    let rect_height = 153;

    let start_x = (width as i32 - rect_width) / 2;
    let start_y = (height as i32 - rect_height) / 2;

    let opacity = OPACITY_PERCENT / 100.0;
    let gray_sep = apply_transparency(100, 100, 100, opacity);
    let day_done = apply_transparency(0, 255, 0, opacity);
    let day_pending = apply_transparency(255, 0, 0, opacity);

    let mut shff = 12390123;

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

            shff ^= shff << 13;
            shff ^= shff >> 17;
            shff ^= shff << 5;
            let is_day_done = shff % 2 == 0;

            let color = if x % 21 < 6 || y % 21 < 6 {
                gray_sep
            } else {
                if is_day_done { day_done } else { day_pending }
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
