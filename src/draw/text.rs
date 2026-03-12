use embedded_graphics::{
    image::GetPixel, mono_font::ascii::FONT_7X13, pixelcolor::BinaryColor, prelude::*,
};

const FONT_CHAR_WIDTH: u32 = 7;
const FONT_CHAR_HEIGHT: u32 = 13;
const ASCII_START: u8 = 32;
const ASCII_END: u8 = 126;

pub fn draw_text(buffer: &mut [u32], width: u32, height: u32, text: &str, scale: u8) {
    let font_image = &FONT_7X13.image;
    let img_width = font_image.size().width;
    let chars_per_row = img_width / FONT_CHAR_WIDTH;

    let scale = scale as u32;
    let scaled_char_width = FONT_CHAR_WIDTH * scale;
    let scaled_char_height = FONT_CHAR_HEIGHT * scale;

    let text_len = text.chars().count() as u32;
    let total_width = text_len * scaled_char_width;

    let start_x = (width - total_width) / 2;
    let start_y = (height - scaled_char_height) / 2;

    for (char_idx, c) in text.chars().enumerate() {
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

        let dst_char_x = start_x + char_idx * scaled_char_width;

        for y in 0..scaled_char_height {
            let dst_y = start_y + y;
            if dst_y >= height {
                break;
            }

            let src_y = atlas_char_y + (y / scale);

            for x in 0..scaled_char_width {
                let dst_x = dst_char_x + x;
                if dst_x >= width {
                    break;
                }

                let src_x = atlas_char_x + (x / scale);
                let point = Point::new(src_x as i32, src_y as i32);

                if let Some(color) = font_image.pixel(point) {
                    let color = if color == BinaryColor::On {
                        0xffffffff
                    } else {
                        0x00000000
                    };
                    let index = dst_y * width + dst_x;
                    buffer[index as usize] = color;
                }
            }
        }
    }
}
