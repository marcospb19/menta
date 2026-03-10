use embedded_graphics::{
    image::GetPixel, mono_font::ascii::FONT_7X13, pixelcolor::BinaryColor, prelude::*,
};

pub fn draw_font_atlas(buffer: &mut [u32], width: u32, height: u32, scale: u8) {
    let font_image = &FONT_7X13.image;
    let img_width = font_image.size().width;
    let img_height = font_image.size().height;

    let scaled_width = img_width * scale as u32;
    let scaled_height = img_height * scale as u32;

    let start_x = (width - scaled_width) / 2;
    let start_y = (height - scaled_height) / 2;

    for y in 0..scaled_height {
        let draw_y = start_y + y;
        if draw_y >= height {
            panic!();
        }

        let src_y = (y / scale as u32) as i32;

        for x in 0..scaled_width {
            let draw_x = start_x + x;
            assert!(draw_x < width);

            let src_x = (x / scale as u32) as i32;

            let point = Point::new(src_x, src_y);
            if let Some(color) = font_image.pixel(point) {
                let color = if color == BinaryColor::On {
                    0xffffffff
                } else {
                    0x00000000
                };
                let index = draw_y * width + draw_x;
                buffer[index as usize] = color;
            }
        }
    }
}
