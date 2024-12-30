use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use eyre::Result;
use image::{
    imageops::{grayscale, invert},
    DynamicImage,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};

use super::comic::Comic;
pub struct Ui {
    terminal: DefaultTerminal,
    picker: Picker,
    invert_image: bool,
}

pub enum RenderOption {
    ToggleInvert,
    Resize,
    BookmarkComic,
    Error(String),
    None,
}

impl Ui {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::init();
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )?;
        Ok(Self {
            terminal,
            picker: Picker::from_query_stdio()?,
            invert_image: true,
        })
    }

    pub fn update(&mut self, comic: &Comic, option: RenderOption) -> Result<()> {
        match option {
            RenderOption::ToggleInvert => self.invert_image = !self.invert_image,
            RenderOption::Resize => self.picker = Picker::from_query_stdio()?,
            _ => {}
        };

        let mut image = self.image_protocol(comic);
        let title_block = Self::title_block(comic, option, self.invert_image);
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
            f.render_stateful_widget(image_widget, layout[1], &mut image);
        })?;
        Ok(())
    }

    fn title_block(comic: &Comic, option: RenderOption, invert_image: bool) -> Block {
        let mut block = Block::new()
            .title_top(comic.date_uploaded.as_str().blue())
            .title_top(
                Line::styled(
                    format!("{}: {}", comic.number, comic.name),
                    Style::new().yellow().bold(),
                )
                .centered(),
            );
        let message = match option {
            RenderOption::ToggleInvert => Some(
                format!(
                    "Image inversion is now {}",
                    if invert_image { "on" } else { "off" }
                )
                .magenta(),
            ),
            RenderOption::BookmarkComic => Some("Bookmarked comic!".cyan()),
            RenderOption::Error(error) => Some(error.clone().red()),
            _ => None,
        };

        if let Some(message) = message {
            block = block.title_top(message.into_right_aligned_line())
        }
        block
    }

    fn image_protocol(&mut self, comic: &Comic) -> StatefulProtocol {
        self.picker.new_resize_protocol(if self.invert_image {
            invert_image(&comic.image)
        } else {
            comic.image.clone()
        })
    }
}

fn invert_image(image: &DynamicImage) -> DynamicImage {
    let mut grayscale = grayscale(image);
    invert(&mut grayscale);
    grayscale.into()
}
