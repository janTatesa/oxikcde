mod image;
pub mod terminal;

use super::{comic::Comic, config::StylingConfig, config::TerminalConfig, OpenInBrowser};
use ::image::DynamicImage;
use color_eyre::Result;
use image::*;
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Styled,
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use ratatui_image::{protocol::StatefulProtocol, Resize};
use terminal::*;

pub struct Ui {
    terminal: DefaultTerminal,
    image_protocols: ImageProtocols,
    image_processor: ImageProcessor,
    process_image: bool,
    styling_config: StylingConfig,
}

pub enum RenderOption {
    ToggleProcessing,
    ShowBookmarkComicMessage,
    ShowError(String),
    ShowOpenInBrowserMessage(OpenInBrowser),
    NewComic(DynamicImage),
    None,
}

impl Ui {
    pub fn new(
        original_image: DynamicImage,
        styling_config: StylingConfig,
        terminal_config: TerminalConfig,
        keep_colors: bool,
    ) -> Result<Self> {
        let terminal = initialise_terminal()?;
        let image_processor = ImageProcessor::new(
            terminal_config
                .foreground_color
                .map(Ok)
                .unwrap_or_else(|| get_color(FOREGROUND_COLOR))?,
            terminal_config
                .background_color
                .map(Ok)
                .unwrap_or_else(|| get_color(BACKGROUND_COLOR))?,
            keep_colors,
        )?;
        let image_protocols = image_processor.image_protocols(original_image);
        Ok(Self {
            terminal,
            process_image: true,
            styling_config,
            image_protocols,
            image_processor,
        })
    }

    pub fn update(&mut self, comic: &Comic, option: RenderOption) -> Result<()> {
        let message = match option {
            RenderOption::ToggleProcessing => {
                self.process_image = !self.process_image;
                Some(
                    format!(
                        "Image processing is now {}!",
                        if self.process_image { "on" } else { "off" }
                    )
                    .set_style(self.styling_config.messages_style),
                )
            }
            RenderOption::ShowBookmarkComicMessage => {
                Some("Bookmarked comic!".set_style(self.styling_config.messages_style))
            }
            RenderOption::ShowOpenInBrowserMessage(open_in_browser) => Some(
                format!("Opened {open_in_browser} in your web browser!",)
                    .set_style(self.styling_config.messages_style),
            ),
            RenderOption::ShowError(error) => {
                Some(error.set_style(self.styling_config.errors_style))
            }
            RenderOption::NewComic(image) => {
                self.image_protocols = self.image_processor.image_protocols(image);
                None
            }
            _ => None,
        };

        let mut title_block = Block::new()
            .title_top(
                comic
                    .date_uploaded()
                    .set_style(self.styling_config.date_style),
            )
            .title_top(
                Line::styled(format!("{comic}"), self.styling_config.title_style).centered(),
            );

        if let Some(message) = message {
            title_block = title_block.title_top(message.into_right_aligned_line())
        }

        let alt_text = Paragraph::new(comic.alt_text())
            .centered()
            .wrap(Wrap::default())
            .set_style(self.styling_config.alt_text_style);

        self.terminal.draw(|frame| {
            render(
                title_block,
                alt_text,
                self.image_protocols.get(self.process_image),
                frame,
            )
        })?;
        Ok(())
    }
}

fn render(
    title_block: Block,
    alt_text: Paragraph,
    image: &mut StatefulProtocol,
    frame: &mut Frame,
) {
    let alt_text_height = alt_text.line_count(frame.area().width) as u16;
    let layout = layout(alt_text_height).split(frame.area());
    let image_area = image.size_for(&Resize::Scale(None), layout[1]);
    let centered_image_area = center_area(
        layout[1],
        Constraint::Length(image_area.width),
        Constraint::Length(image_area.height),
    );
    frame.render_widget(title_block, layout[0]);
    frame.render_widget(alt_text, layout[2]);
    frame.render_stateful_widget(IMAGE_WIDGET, centered_image_area, image);
}

fn center_area(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn layout(alt_text_height: u16) -> Layout {
    Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(alt_text_height),
        ],
    )
}
