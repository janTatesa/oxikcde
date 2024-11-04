use color_eyre::Result;
use ratatui::{layout::Rect, widgets::Block, DefaultTerminal};
use ratatui_image::{picker::Picker, StatefulImage};

use crate::comic::Comic;

pub struct App {
    image_picker: Picker,
    terminal: DefaultTerminal,
    comic: Comic,
}

//TODO: Currently the app prints one xkcd and exits. Implement a TUI
impl App {
    pub fn new(terminal: DefaultTerminal) -> Result<Self> {
        let image_picker = Picker::from_query_stdio()?;
        let comic = Comic::download(None)?;
        Ok(Self {
            image_picker,
            terminal,
            comic,
        })
    }

    pub fn run(mut self) -> Result<()> {
        loop {
            self.render()?;
            if self.handle_events()? {
                break;
            }
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::new()
                    .title_top(self.comic.title())
                    .title_bottom(self.comic.alt_text()),
                area,
            );
            let image_area = Rect {
                y: area.y + 1,
                height: area.height - 2,
                ..area
            };

            //TODO: handle the unwrap
            let mut image = self.comic.image(&mut self.image_picker).unwrap();

            // The image widget.
            //TODO: resize the image
            let image_widget = StatefulImage::new(Some(image::Rgb([30, 30, 46])))
                .resize(ratatui_image::Resize::Fit(None));
            // Render with the protocol state.
            f.render_stateful_widget(image_widget, image_area, &mut image)
        })?;
        Ok(())
    }

    //TODO
    pub fn handle_events(&mut self) -> Result<bool> {
        let _ = crossterm::event::read()?;
        Ok(true)
    }
}
