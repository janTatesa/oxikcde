use eyre::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};
use std::array;

const RESIZE: Resize = Resize::Scale(None);
pub fn image_widget() -> StatefulImage {
    StatefulImage::default().resize(RESIZE)
}

type Color = [u8; 3];
const WHITE: Color = [255, 255, 255];
const BLACK: Color = [0, 0, 0];

pub struct ImageProtocols {
    original_image_protocol: StatefulProtocol,
    processed_image_protocol: StatefulProtocol,
}

impl ImageProtocols {
    pub fn get(&mut self, process_image: bool) -> &mut StatefulProtocol {
        if process_image {
            &mut self.processed_image_protocol
        } else {
            &mut self.original_image_protocol
        }
    }
}

pub struct ImageProcessor {
    foreground_color: Color,
    background_color: Color,
    picker: Picker,
    keep_colors: bool,
}

impl ImageProcessor {
    pub fn new(
        foreground_color: Rgb<u8>,
        background_color: Rgb<u8>,
        keep_colors: bool,
    ) -> Result<Self> {
        let mut picker = Picker::from_query_stdio()?;
        picker.set_background_color(background_color.to_rgba().0);
        Ok(Self {
            foreground_color: foreground_color.0,
            background_color: background_color.0,
            keep_colors,
            picker,
        })
    }

    pub fn image_protocols(&self, image: DynamicImage) -> ImageProtocols {
        ImageProtocols {
            processed_image_protocol: self.picker.new_resize_protocol(self.process_image(&image)),
            original_image_protocol: self.picker.new_resize_protocol(image),
        }
    }

    fn process_image(&self, image: &DynamicImage) -> DynamicImage {
        let (width, height) = image.dimensions();
        let mut out = ImageBuffer::new(width, height);

        image.pixels().for_each(|(x, y, pixel)| {
            out.put_pixel(x, y, Rgb::from(self.pixel_color(pixel.to_rgb().0)))
        });

        out.into()
    }

    fn pixel_color(&self, original_color: Color) -> Color {
        match original_color {
            WHITE => self.background_color,
            BLACK => self.foreground_color,
            color if is_grayscale(color) || !self.keep_colors => self.blend_color(color),
            color => color,
        }
    }

    fn blend_color(&self, original_color: Color) -> Color {
        let ratio = Rgb::from(original_color).to_luma().0[0] as f64 / 255.0;
        array::from_fn(|i| {
            let fg = self.foreground_color[i] as f64;
            let bg = self.background_color[i] as f64;

            (bg * ratio + fg * (1.0 - ratio)) as u8
        })
    }
}

const fn is_grayscale([r, g, b]: Color) -> bool {
    r == g && r == b
}
