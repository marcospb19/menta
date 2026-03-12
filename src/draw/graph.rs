use crate::{contributions::ContributionGrid, draw::apply_transparency};

const DAYS_IN_WEEK: u32 = 7;
const SQUARE_SIZE: u32 = 21;
const GAP_SIZE: u32 = 6;

fn cell_exists(grid: &ContributionGrid, row: u32, col: u32) -> bool {
    if !(0..DAYS_IN_WEEK).contains(&row) {
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
    if !(0..DAYS_IN_WEEK).contains(&row) {
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
    let column_count = grid.column_count() as u32;

    let total_rect_width = column_count * SQUARE_SIZE + GAP_SIZE;
    let total_rect_height = DAYS_IN_WEEK * SQUARE_SIZE + GAP_SIZE;

    let start_x = width.checked_sub(total_rect_width).unwrap() / 2;
    let start_y = height.checked_sub(total_rect_height).unwrap() / 2;

    let surrounding_color = apply_transparency(0, 0, 0);
    let level_colors: &[u32] = &[
        apply_transparency(20, 20, 20),
        apply_transparency(2 * 5, 14 * 5, 5 * 5),
        apply_transparency(2 * 8, 14 * 8, 5 * 8),
        apply_transparency(2 * 10, 14 * 10, 5 * 10),
        apply_transparency(2 * 12, 14 * 12, 5 * 12),
    ];

    for y in 0..total_rect_height {
        let draw_y = start_y + y;
        assert!(draw_y < height);

        for x in 0..total_rect_width {
            let draw_x = start_x + x;
            assert!(draw_x < width);

            let col = x / SQUARE_SIZE;
            let row = y / SQUARE_SIZE;
            let in_sep_x = x % SQUARE_SIZE < GAP_SIZE;
            let in_sep_y = y % SQUARE_SIZE < GAP_SIZE;

            let color = if in_sep_x && in_sep_y {
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
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row, col - 1);
                if draw {
                    surrounding_color
                } else {
                    continue;
                }
            } else if in_sep_y {
                let draw = cell_exists(grid, row, col) || cell_exists(grid, row - 1, col);
                if draw {
                    surrounding_color
                } else {
                    continue;
                }
            } else if let Some(level) = cell_value(grid, row, col) {
                *level_colors
                    .get(level as usize)
                    .unwrap_or_else(|| level_colors.last().unwrap())
            } else {
                continue;
            };

            let index = draw_y * width + draw_x;
            buffer[index as usize] = color;
        }
    }

    let rotation_anchor_right = 3440 / 2 - column_count as usize * 10 - 7;
    let right_padding = 30;
    buffer.rotate_right(rotation_anchor_right - right_padding);
}
