use color_eyre::Result;
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
    comic: Comic,
}

impl Ui {
    pub fn new(terminal: DefaultTerminal) -> Result<Self> {
        let picker = Picker::from_query_stdio()?;
        Ok(Self {
            terminal,
            picker,
            comic: Comic::default(),
        })
    }
    pub fn handle_resize(&mut self) -> Result<()> {
        self.picker = Picker::from_query_stdio()?;
        self.render_internal()
    }

    pub fn render(&mut self, comic: Comic) -> Result<()> {
        self.comic = comic;
        self.render_internal()
    }

    fn render_internal(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::new()
                    .title_top(self.comic.date_uploaded.as_str().blue())
                    .title_top(Line::styled(
                        &self.comic.title,
                        Style::new().yellow().bold(),
                    )),
                area,
            );
            let alt_text = Paragraph::new(self.comic.alt_text.as_str())
                .centered()
                .wrap(Wrap::default());
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

            //TODO: handle the unwrap
            let mut image = self.picker.new_resize_protocol(self.comic.image.clone());

            // The image widget.
            //TODO: resize the image
            let image_widget = StatefulImage::new(Some(image::Rgb([30, 30, 46])))
                .resize(ratatui_image::Resize::Fit(None));
            // Render with the protocol state.
            f.render_stateful_widget(image_widget, image_area, &mut image)
        })?;
        Ok(())
    }
}
