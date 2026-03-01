use std::{num::NonZeroU32, rc::Rc};

use winit::window::Window;

pub fn draw_gradient(
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

    for y in 0..height {
        for x in 0..width {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = 128u8;
            let b = ((y as f32 / height as f32) * 255.0) as u8;
            let color = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
            let index = (y * width + x) as usize;
            if index < buffer.len() {
                buffer[index] = color;
            }
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
