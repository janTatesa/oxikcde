use super::{comic::Comic, OpenInBrowser};
use color_eyre::owo_colors::Rgb;
use colors_transform::Color;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use eyre::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};
use std::array;
pub struct Ui {
    terminal: DefaultTerminal,
    picker: Picker,
    original_image_protocol: StatefulProtocol,
    inverted_image_protocol: StatefulProtocol,
    process_image: bool,
    text_color: [u8; 3],
    background_color: [u8; 3],
}

pub enum RenderOption {
    ToggleInvert,
    BookmarkComic,
    Error(String),
    OpenInBrowser(OpenInBrowser),
    NewComic(DynamicImage),
    None,
}

impl Ui {
    pub fn new(original_image: DynamicImage) -> Result<Self> {
        let terminal = initialise_terminal()?;
        let (text_color, background_color) =
            (get_color(FOREGROUND_COLOR)?, get_color(BACKGROUND_COLOR)?);

        let picker = Picker::from_query_stdio()?;
        let inverted_image_protocol = picker.new_resize_protocol(process_image(
            text_color,
            background_color,
            &original_image,
        ));
        let original_image_protocol = picker.new_resize_protocol(original_image);
        Ok(Self {
            terminal,
            picker,
            process_image: true,
            original_image_protocol,
            inverted_image_protocol,
            text_color,
            background_color,
        })
    }

    pub fn update(&mut self, comic: &Comic, option: RenderOption) -> Result<()> {
        let message = match option {
            RenderOption::ToggleInvert => {
                self.process_image = !self.process_image;
                Some(
                    format!(
                        "Image processing is now {}!",
                        if self.process_image { "on" } else { "off" }
                    )
                    .magenta(),
                )
            }
            RenderOption::BookmarkComic => Some("Bookmarked comic!".cyan()),
            RenderOption::OpenInBrowser(open_in_browser) => Some(
                format!(
                    "Opened {} in your web browser!",
                    match open_in_browser {
                        OpenInBrowser::Comic => "comic",
                        OpenInBrowser::Explanation => "explanation",
                    },
                )
                .green(),
            ),
            RenderOption::Error(error) => Some(error.red()),
            RenderOption::NewComic(image) => {
                self.inverted_image_protocol = self.picker.new_resize_protocol(process_image(
                    self.text_color,
                    self.background_color,
                    &image,
                ));
                self.original_image_protocol = self.picker.new_resize_protocol(image);
                None
            }
            _ => None,
        };
        let title_block = title_block(comic, message);

        let image = if self.process_image {
            &mut self.inverted_image_protocol
        } else {
            &mut self.original_image_protocol
        };

        let alt_text = Paragraph::new(comic.alt_text.as_str())
            .centered()
            .wrap(Wrap::default())
            .dark_gray();
        //TODO: Center the image
        let image_widget = StatefulImage::default().resize(Resize::Scale(None));
        self.terminal.draw(|f| {
            let layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(alt_text.line_count(f.area().width) as u16),
                ],
            )
            .split(f.area());

            f.render_widget(title_block, layout[0]);
            f.render_widget(alt_text, layout[2]);
            f.render_stateful_widget(image_widget, layout[1], image);
        })?;
        Ok(())
    }
}

fn process_image(
    foreground_color: [u8; 3],
    background_color: [u8; 3],
    image: &DynamicImage,
) -> DynamicImage {
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(width, height);

    for (x, y, pixel) in image.pixels() {
        let grayscale = pixel.to_luma().0[0];
        let ratio = grayscale as f64 / 255.0;

        let mut new_pixel_iter = foreground_color.into_iter().zip(background_color).map(
            |(foreground_color, background_color)| {
                (background_color as f64 * ratio + foreground_color as f64 * (1.0 - ratio)) as u8
            },
        );

        let new_pixel: [_; 3] = array::from_fn(|_| new_pixel_iter.next().unwrap());
        let new_pixel = image::Rgb::from(new_pixel);

        out.put_pixel(x, y, new_pixel);
    }

    out.into()
}
fn initialise_terminal() -> Result<DefaultTerminal> {
    let terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    Ok(terminal)
}

const FOREGROUND_COLOR: u8 = 10;
const BACKGROUND_COLOR: u8 = 11;
const DELAY_MS: u64 = 20;
fn get_color(code: u8) -> Result<[u8; 3]> {
    let string = xterm_query::query_osc(format!("\x1b]{code};?\x1b\\").as_str(), DELAY_MS)?;
    let mut hex = String::new();
    for i in (8..19).step_by(5) {
        hex.push_str(&string[i..(i + 2)])
    }
    let rgb = colors_transform::Rgb::from_hex_str(&hex)
        .unwrap()
        .as_tuple();
    Ok([rgb.0 as u8, rgb.1 as u8, rgb.2 as u8])
}

fn title_block<'a>(comic: &'a Comic, message: Option<Span<'a>>) -> Block<'a> {
    let mut block = Block::new()
        .title_top(comic.date_uploaded.as_str().blue())
        .title_top(
            Line::styled(
                format!("{}: {}", comic.number, comic.name),
                Style::new().yellow().bold(),
            )
            .centered(),
        );

    if let Some(message) = message {
        block = block.title_top(message.into_right_aligned_line())
    }
    block
}
