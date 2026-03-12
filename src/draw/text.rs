use embedded_graphics::{
    image::GetPixel, mono_font::ascii::FONT_7X13, pixelcolor::BinaryColor, prelude::*,
};

const FONT_CHAR_WIDTH: u32 = 7;
const FONT_CHAR_HEIGHT: u32 = 13;
const ASCII_START: u8 = 32;
const ASCII_END: u8 = 126;

/// Calculates the default wrap width for text: (15 + sqrt(text_length) * aspect_ratio) * 0.7
fn default_wrap_width(text_len: usize, width: u32, height: u32) -> usize {
    let aspect_ratio = width as f32 / height.max(1) as f32;
    ((15.0 + (text_len as f32).sqrt() * aspect_ratio) * 0.85) as usize
}

/// Wraps text into lines based on a maximum line length.
/// Respects word boundaries where possible.
fn wrap_text(text: &str, max_line_len: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let line_len = current_line.chars().count();

        if line_len == 0 {
            // Start new line with this word
            if word_len > max_line_len {
                // Word is too long, we need to split it
                let chars = word.chars();
                let mut chunk = String::new();
                for c in chars {
                    if chunk.chars().count() >= max_line_len {
                        lines.push(chunk);
                        chunk = String::new();
                    }
                    chunk.push(c);
                }
                if !chunk.is_empty() {
                    current_line = chunk;
                }
            } else {
                current_line.push_str(word);
            }
        } else if line_len + 1 + word_len <= max_line_len {
            // Add word to current line
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Start new line
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

pub fn draw_text(buffer: &mut [u32], width: u32, height: u32, text: &str, scale: u8) {
    draw_text_wrapped(buffer, width, height, text, scale, None);
}

pub fn draw_text_wrapped(
    buffer: &mut [u32],
    width: u32,
    height: u32,
    text: &str,
    scale: u8,
    wrap_width: Option<usize>,
) {
    let font_image = &FONT_7X13.image;
    let img_width = font_image.size().width;
    let chars_per_row = img_width / FONT_CHAR_WIDTH;

    let scale = scale as u32;
    let scaled_char_width = FONT_CHAR_WIDTH * scale;
    let scaled_char_height = FONT_CHAR_HEIGHT * scale;

    // Wrap text into lines
    let text_len = text.chars().count();
    let wrap_width = wrap_width.unwrap_or_else(|| default_wrap_width(text_len, width, height));
    let lines = wrap_text(text, wrap_width.max(1));

    let max_line_len = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u32;

    let total_width = max_line_len * scaled_char_width;

    let start_x = (width - total_width) / 2;
    let start_y = (height - scaled_char_height) / 2;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_idx = line_idx as u32;
        let line_start_y = start_y + line_idx * scaled_char_height;

        let line_len = line.chars().count() as u32;
        let line_width = line_len * scaled_char_width;
        let line_start_x = start_x + (total_width - line_width) / 2;

        for (char_idx, c) in line.chars().enumerate() {
            let char_idx = char_idx as u32;
            let ascii_val = c as u8;

            if !(ASCII_START..=ASCII_END).contains(&ascii_val) {
                continue;
            }

            let atlas_idx = (ascii_val - ASCII_START) as u32;
            let atlas_col = atlas_idx % chars_per_row;
            let atlas_row = atlas_idx / chars_per_row;
            let atlas_char_x = atlas_col * FONT_CHAR_WIDTH;
            let atlas_char_y = atlas_row * FONT_CHAR_HEIGHT;

            let dst_char_x = line_start_x + char_idx * scaled_char_width;

            for y in 0..scaled_char_height {
                let dst_y = line_start_y + y;

                let src_y = atlas_char_y + (y / scale);

                for x in 0..scaled_char_width {
                    let dst_x = dst_char_x + x;

                    let src_x = atlas_char_x + (x / scale);
                    let point = Point::new(src_x as i32, src_y as i32);

                    if let Some(color) = font_image.pixel(point) {
                        let color = if color == BinaryColor::On {
                            0xffff_ffff
                        } else {
                            0x0000_0000
                        };
                        let index = dst_y * width + dst_x;
                        buffer[index as usize] = color;
                    }
                }
            }
        }
    }
}
