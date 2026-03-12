use std::{mem, sync::LazyLock};

use embedded_graphics::{
    geometry::OriginDimensions,
    image::{GetPixel, ImageRaw},
    mono_font,
    pixelcolor::BinaryColor,
    prelude::*,
};

const FONT: mono_font::MonoFont = mono_font::ascii::FONT_7X13;
static FONT_CHAR_WIDTH: LazyLock<u32> = LazyLock::new(|| FONT.image.size().width / 16);
static FONT_CHAR_HEIGHT: LazyLock<u32> = LazyLock::new(|| FONT.image.size().height / 6);
const ASCII_START: u8 = 32;

/// Samples a pixel from the font atlas for a given character at local coordinates.
fn sample_atlas(
    font_image: &ImageRaw<'_, BinaryColor>,
    ascii_val: u8,
    x: u32,
    y: u32,
    scale: u32,
    chars_per_row: u32,
) -> BinaryColor {
    let atlas_idx = (ascii_val - ASCII_START) as u32;
    let atlas_col = atlas_idx % chars_per_row;
    let atlas_row = atlas_idx / chars_per_row;
    let atlas_char_x = atlas_col * *FONT_CHAR_WIDTH;
    let atlas_char_y = atlas_row * *FONT_CHAR_HEIGHT;

    let src_x = atlas_char_x + (x / scale);
    let src_y = atlas_char_y + (y / scale);
    let point = Point::new(src_x as i32, src_y as i32);

    font_image.pixel(point).unwrap()
}

/// Calculates the default wrap width for text: (15 + sqrt(text_length) * aspect_ratio) * 0.7
fn default_wrap_width(text_len: usize, width: u32, height: u32) -> usize {
    let aspect_ratio = width as f32 / height.max(1) as f32;
    15 + ((text_len as f32).sqrt() * aspect_ratio * 0.85) as usize
}

/// Helper struct for managing text wrapping with max line length constraints
struct TextWrapper {
    previous_lines: Vec<String>,
    current: String,
    current_chars_len: usize,
    max_len: usize,
    // was_current_line_wrapped: bool,
}

impl TextWrapper {
    pub fn new(max_len: usize) -> Self {
        Self {
            previous_lines: Vec::new(),
            current: String::new(),
            current_chars_len: 0,
            max_len,
        }
    }

    pub fn push_text(&mut self, text: &str) {
        let word_len = text.chars().count();

        if self.current_chars_len + word_len > self.max_len {
            assert!(!self.current.is_empty(), "we break on huge lines lol");

            if text == " " {
                return; // ignore
            } else {
                self.start_new_line(); // break and keep going
            }
        }

        self.current.push_str(text);
        self.current_chars_len += word_len;
    }

    pub fn start_new_line(&mut self) {
        let previous = mem::take(&mut self.current);
        self.previous_lines.push(previous);
        self.current_chars_len = 0;
    }

    fn finalize(mut self) -> Vec<String> {
        if !self.current.is_empty() {
            self.previous_lines.push(self.current);
        }
        self.previous_lines
    }
}

/// Wraps text into lines based on a maximum line length.
/// Respects word boundaries where possible and preserves multiple spaces.
fn wrap_text(text: &str, max_line_len: usize) -> Vec<String> {
    let mut wrapper = TextWrapper::new(max_line_len);

    for line in text.lines() {
        for piece in line.split(' ').intersperse(" ") {
            wrapper.push_text(piece);
        }
        wrapper.start_new_line();
    }

    wrapper.finalize()
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
    let font_image = &FONT.image;
    let img_width = font_image.size().width;
    let chars_per_row = img_width / *FONT_CHAR_WIDTH;

    let scale = scale as u32;
    let scaled_char_width = *FONT_CHAR_WIDTH * scale;
    let scaled_char_height = *FONT_CHAR_HEIGHT * scale;

    // Wrap text into lines
    let text_len = text.chars().count();
    let wrap_width = wrap_width.unwrap_or_else(|| default_wrap_width(text_len, width, height));

    let mut lines = wrap_text(text, wrap_width);
    if lines.is_empty() {
        lines = vec!["<empty>".to_string()];
    }

    let max_line_len = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u32;

    let rect_width = max_line_len * scaled_char_width;
    let rect_height = lines.len() as u32 * scaled_char_height;

    // top-left corner position (clamp to 0 to prevent overflow)
    let rect_origin_x = width.saturating_sub(rect_width) / 2;
    let rect_origin_y = height.saturating_sub(rect_height) / 2;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_origin_y = rect_origin_y + line_idx as u32 * scaled_char_height;

        for (char_idx, c) in line.chars().enumerate() {
            let ascii = if c.is_ascii() { c as u8 } else { b'?' };
            let char_origin_x = rect_origin_x + char_idx as u32 * scaled_char_width;

            for char_pixel_y in 0..scaled_char_height {
                let screen_pixel_y = line_origin_y + char_pixel_y;

                for char_pixel_x in 0..scaled_char_width {
                    let screen_pixel_x = char_origin_x + char_pixel_x;

                    let color = sample_atlas(
                        font_image,
                        ascii,
                        char_pixel_x,
                        char_pixel_y,
                        scale,
                        chars_per_row,
                    );
                    let color = if color == BinaryColor::On {
                        0xffff_ffff
                    } else {
                        0x0000_0000
                    };

                    // Bounds check before writing to buffer
                    if screen_pixel_x < width && screen_pixel_y < height {
                        let index = screen_pixel_y * width + screen_pixel_x;
                        buffer[index as usize] = color;
                    }
                }
            }
        }
    }
}
