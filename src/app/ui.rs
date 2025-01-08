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
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};

use super::{comic::Comic, OpenInBrowser};
pub struct Ui {
    terminal: DefaultTerminal,
    picker: Picker,
    original_image_protocol: StatefulProtocol,
    inverted_image_protocol: StatefulProtocol,
    invert_image: bool,
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
        let terminal = ratatui::init();
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )?;
        let picker = Picker::from_query_stdio()?;
        let inverted_image_protocol = picker.new_resize_protocol(invert_image(&original_image));
        let original_image_protocol = picker.new_resize_protocol(original_image);
        Ok(Self {
            terminal,
            picker,
            invert_image: true,
            original_image_protocol,
            inverted_image_protocol,
        })
    }

    pub fn update(&mut self, comic: &Comic, option: RenderOption) -> Result<()> {
        let message = match option {
            RenderOption::ToggleInvert => {
                self.invert_image = !self.invert_image;
                Some(
                    format!(
                        "Image inversion is now {}!",
                        if self.invert_image { "on" } else { "off" }
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
                self.inverted_image_protocol =
                    self.picker.new_resize_protocol(invert_image(&image));
                self.original_image_protocol = self.picker.new_resize_protocol(image);
                None
            }
            _ => None,
        };
        let title_block = title_block(comic, message);

        let image = if self.invert_image {
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

fn invert_image(image: &DynamicImage) -> DynamicImage {
    let mut grayscale = grayscale(image);
    invert(&mut grayscale);
    grayscale.into()
}
