use std::{num::NonZeroU32, rc::Rc};

use winit::window::Window;

use crate::{OPACITY_PERCENT, contributions::ContributionGrid};

fn apply_transparency(r: u8, g: u8, b: u8, opacity: f32) -> u32 {
    let alpha = (opacity * 255.0).round() as u8;
    let r = ((r as f32) * opacity).round() as u8;
    let g = ((g as f32) * opacity).round() as u8;
    let b = ((b as f32) * opacity).round() as u8;

    ((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn cell_exists(grid: &ContributionGrid, row: i32, col: i32) -> bool {
    if row < 0 || row >= 7 || col < 0 {
        return false;
    }
    let row = row as usize;
    let col = col as usize;
    if col >= grid.rows[row].len() {
        return false;
    }
    grid.rows[row][col].is_some()
}

fn cell_value(grid: &ContributionGrid, row: i32, col: i32) -> Option<u8> {
    if row < 0 || row >= 7 || col < 0 {
        return None;
    }
    let row = row as usize;
    let col = col as usize;
    if col >= grid.rows[row].len() {
        return None;
    }
    grid.rows[row][col]
}

pub fn draw_contribution_graph(
    surface: &mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
    width: u32,
    height: u32,
    grid: &ContributionGrid,
) {
    if width == 0 || height == 0 {
        return;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(_) => return,
    };

    buffer.fill(0x00000000);

    let num_rows = 7i32;
    let num_cols = grid.rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let cell_size = 21i32;
    let rect_width = num_cols * cell_size + 6;
    let rect_height = num_rows * cell_size + 6;

    let start_x = (width as i32 - rect_width) / 2;
    let start_y = (height as i32 - rect_height) / 2;

    let opacity = OPACITY_PERCENT / 100.0;
    let sep_color = apply_transparency(100, 100, 100, opacity);
    let level_colors: [u32; 5] = [
        apply_transparency(255, 0, 0, opacity), // level 0: red (no contributions)
        apply_transparency(0, 100, 0, opacity), // level 1: dim green
        apply_transparency(0, 160, 0, opacity), // level 2: medium green
        apply_transparency(0, 210, 0, opacity), // level 3: bright green
        apply_transparency(0, 255, 0, opacity), // level 4: brightest green
    ];

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

            let col = x / cell_size;
            let row = y / cell_size;
            let in_sep_x = x % cell_size < 6;
            let in_sep_y = y % cell_size < 6;

            let color = if in_sep_x && in_sep_y {
                // Corner: draw if any of the 4 adjacent cells exist
                let draw = cell_exists(grid, row, col)
                    || cell_exists(grid, row - 1, col)
                    || cell_exists(grid, row, col - 1)
                    || cell_exists(grid, row - 1, col - 1);
                if draw {
                    sep_color
                } else {
                    continue;
                }
            } else if in_sep_x {
                // Vertical separator: between col-1 and col, inside row
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row, col - 1);
                if draw {
                    sep_color
                } else {
                    continue;
                }
            } else if in_sep_y {
                // Horizontal separator: between row-1 and row, inside col
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row - 1, col);
                if draw {
                    sep_color
                } else {
                    continue;
                }
            } else {
                // Cell content
                if let Some(level) = cell_value(grid, row, col) {
                    level_colors[level as usize]
                } else {
                    continue;
                }
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
