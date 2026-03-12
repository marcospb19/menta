use std::{num::NonZeroU32, rc::Rc};

use winit::window::Window;

use crate::OPACITY_PERCENT;

pub mod graph;
pub mod text;

pub fn apply_transparency(r: u8, g: u8, b: u8) -> u32 {
    let opacity = OPACITY_PERCENT / 100.0;

    let alpha = (opacity * 255.0).round() as u8;
    let r = ((r as f32) * opacity).round() as u8;
    let g = ((g as f32) * opacity).round() as u8;
    let b = ((b as f32) * opacity).round() as u8;

    ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
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
