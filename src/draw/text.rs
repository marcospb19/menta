use embedded_graphics::{
    image::GetPixel, mono_font::ascii::FONT_7X13, pixelcolor::BinaryColor, prelude::*,
};

pub fn draw_font_atlas(buffer: &mut [u32], width: u32, height: u32) {
    let font_image = &FONT_7X13.image;
    let img_width = font_image.size().width;
    let img_height = font_image.size().height;

    let start_x = (width - img_width) / 2;
    let start_y = (height - img_height) / 2;

    for y in 0..img_height {
        let draw_y = start_y + y;
        if draw_y >= height {
            panic!();
        }

        for x in 0..img_width {
            let draw_x = start_x + x;
            assert!(draw_x < width);

            let point = Point::new(x as i32, y as i32);
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
