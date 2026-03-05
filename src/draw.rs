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

fn cell_exists(grid: &ContributionGrid, row: u32, col: u32) -> bool {
    if !(0..7).contains(&row) {
        return false;
    }
    let row = row as usize;
    let col = col as usize;
    if col >= grid.rows[row].len() {
        return false;
    }
    grid.rows[row][col].is_some()
}

fn cell_value(grid: &ContributionGrid, row: u32, col: u32) -> Option<u8> {
    if !(0..7).contains(&row) {
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
    buffer: &mut [u32],
    width: u32,
    height: u32,
    grid: &ContributionGrid,
) {
    let row_count: u32 = 7;
    let column_count = grid.column_count() as u32;

    let square_cell_size: u32 = 21;
    // Dimensions of the entire
    let total_rect_width = column_count * square_cell_size + 6;
    let total_rect_height = row_count * square_cell_size + 6;

    let start_x = width.checked_sub(total_rect_width).unwrap() / 2;
    let start_y = height.checked_sub(total_rect_height).unwrap() / 2;

    let opacity = OPACITY_PERCENT / 100.0;
    let surrounding_color = apply_transparency(0, 0, 0, opacity);
    let level_colors: &[u32] = &[
        apply_transparency(20, 20, 20, opacity),
        apply_transparency(2 * 5, 14 * 5, 5 * 5, opacity),
        apply_transparency(2 * 8, 14 * 8, 5 * 8, opacity),
        apply_transparency(2 * 10, 14 * 10, 5 * 10, opacity),
        apply_transparency(2 * 12, 14 * 12, 5 * 12, opacity),
    ];

    for y in 0..total_rect_height {
        let draw_y = start_y + y;
        assert!(draw_y < height);

        for x in 0..total_rect_width {
            let draw_x = start_x + x;
            assert!(draw_x < width);

            let col = x / square_cell_size;
            let row = y / square_cell_size;
            let in_sep_x = x % square_cell_size < 6;
            let in_sep_y = y % square_cell_size < 6;

            let color = if in_sep_x && in_sep_y {
                // Corner: draw if any of the 4 adjacent cells exist
                let draw = cell_exists(grid, row, col)
                    || cell_exists(grid, row.saturating_sub(1), col)
                    || cell_exists(grid, row, col.saturating_sub(1))
                    || cell_exists(grid, row.saturating_sub(1), col.saturating_sub(1));
                if draw {
                    surrounding_color
                } else {
                    continue;
                }
            } else if in_sep_x {
                // Vertical separator: between col-1 and col, inside row
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row, col - 1);
                if draw {
                    surrounding_color
                } else {
                    continue;
                }
            } else if in_sep_y {
                // Horizontal separator: between row-1 and row, inside col
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row - 1, col);
                if draw {
                    surrounding_color
                } else {
                    continue;
                }
            } else {
                // Cell content
                if let Some(level) = cell_value(grid, row, col) {
                    *level_colors
                        .get(level as usize)
                        .unwrap_or_else(|| level_colors.last().unwrap())
                } else {
                    continue;
                }
            };

            let index = draw_y * width + draw_x;
            buffer[index as usize] = color;
        }
    }

    // anchor at right side, with some padding
    let rotation_anchor_right = 3440 / 2 - column_count as usize * 10 - 7;
    let right_padding = 30;
    buffer.rotate_right(rotation_anchor_right - right_padding);
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
