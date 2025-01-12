use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use std::array;

use super::Color;

const WHITE: Color = [255, 255, 255];
const BLACK: Color = [0, 0, 0];
pub fn process_image(
    foreground_color: Color,
    background_color: Color,
    image: &DynamicImage,
) -> DynamicImage {
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(width, height);

    image.pixels().for_each(|(x, y, pixel)| {
        out.put_pixel(
            x,
            y,
            Rgb::from(pixel_color(
                pixel.to_rgb().0,
                foreground_color,
                background_color,
            )),
        )
    });

    out.into()
}

fn pixel_color(original_color: Color, foreground_color: Color, background_color: Color) -> Color {
    match original_color {
        WHITE => background_color,
        BLACK => foreground_color,
        color if is_grayscale(color) => blend_color(
            Rgb::from(color).to_luma().0[0] as f64 / 255.0,
            foreground_color,
            background_color,
        ),
        color => color,
    }
}

const fn is_grayscale([r, g, b]: [u8; 3]) -> bool {
    r == g && r == b
}

fn blend_color(ratio: f64, foreground_color: Color, background_color: Color) -> Color {
    array::from_fn(|i| {
        let fg = foreground_color[i] as f64;
        let bg = background_color[i] as f64;

        (bg * ratio + fg * (1.0 - ratio)) as u8
    })
}
