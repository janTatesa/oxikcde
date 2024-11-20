use cli_log::debug;
use color_eyre::Result;
use image::{
    imageops::{grayscale, invert},
    DynamicImage, ImageBuffer,
};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal,
};
use ratatui_image::{picker::Picker, StatefulImage};

use super::comic::Comic;
pub struct Ui {
    terminal: DefaultTerminal,
    picker: Picker,
    invert_image: bool,
    comic: std::result::Result<Comic, String>,
    comic_number: u64,
}

impl Ui {
    pub fn new(terminal: DefaultTerminal, comic: Result<Comic>, comic_number: u64) -> Result<Self> {
        let picker = Picker::from_query_stdio()?;
        let comic = match comic {
            Ok(comic) => Ok(comic),
            Err(e) => Err(e.to_string()),
        };
        let mut ui = Self {
            terminal,
            comic_number,
            picker,
            invert_image: true,
            comic,
        };
        ui.render()?;
        Ok(ui)
    }
    pub fn handle_resize(&mut self) -> Result<()> {
        self.picker = Picker::from_query_stdio()?;
        self.render()
    }

    pub fn render_new_comic(&mut self, comic: Result<Comic>, comic_number: u64) -> Result<()> {
        self.comic = match comic {
            Ok(comic) => Ok(comic),
            Err(e) => Err(e.to_string()),
        };
        self.comic_number = comic_number;
        self.invert_image = true;
        self.render()
    }

    pub fn toggle_invert(&mut self) -> Result<()> {
        self.invert_image = !self.invert_image;
        self.render()
    }

    fn render(&mut self) -> Result<()> {
        match self.comic.clone() {
            Ok(comic) => self.render_sucess(comic),
            Err(e) => self.render_failure(e),
        }
    }
    fn render_sucess(&mut self, comic: Comic) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::new()
                    .title_top(comic.date_uploaded.as_str().blue())
                    .title_top(
                        Line::styled(
                            format!("{}: {}", self.comic_number, comic.name),
                            Style::new().yellow().bold(),
                        )
                        .centered(),
                    ),
                area,
            );
            let alt_text = Paragraph::new(comic.alt_text.as_str())
                .centered()
                .wrap(Wrap::default())
                .dark_gray();
            let alt_text_height = alt_text.line_count(area.width) as u16;
            let alt_text_area = Rect {
                y: area.height - alt_text_height,
                height: alt_text_height,
                ..area
            };
            f.render_widget(alt_text, alt_text_area);
            let image_area = Rect {
                y: area.y + 1,
                height: area.height - 1 - alt_text_height,
                ..area
            };

            let mut image = self.picker.new_resize_protocol(if self.invert_image {
                invert_image(&comic.image)
            } else {
                comic.image
            });

            // The image widget.
            //TODO: resize the image
            let image_widget = StatefulImage::new(None);
            f.render_stateful_widget(image_widget, image_area, &mut image)
        })?;
        Ok(())
    }

    fn render_failure(&mut self, message: String) -> Result<()> {
        self.terminal.draw(|f| {
            f.render_widget(
                Paragraph::new(format!(
                    "Failed to download comic{}, error: {}, press d to download again",
                    if self.comic_number == 0 {
                        String::new()
                    } else {
                        format!(" number {}", self.comic_number)
                    },
                    message
                ))
                .wrap(Wrap::default()),
                f.area(),
            )
        })?;
        Ok(())
    }
}

fn invert_image(image: &DynamicImage) -> DynamicImage {
    let mut grayscale = grayscale(image);
    invert(&mut grayscale);
    grayscale.into()
}
