use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};

use crate::comic::Comic;

pub struct App {
    terminal: DefaultTerminal,
    comic: Comic,
}

//TODO: Currently the app prints one xkcd and exits. Implement a TUI
impl App {
    pub fn new(terminal: DefaultTerminal) -> Result<Self> {
        let comic = Comic::download(None)?;
        Ok(Self { terminal, comic })
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

    //TODO print comic alt text and title
    pub fn render(&mut self) -> Result<()> {
        let mut image = self.comic.image()?;
        self.terminal.draw(|f| {
            // The image widget.
            let image_widget = StatefulImage::new(Some(image::Rgb([30, 30, 46])));
            // Render with the protocol state.
            f.render_stateful_widget(image_widget, f.area(), &mut image);
        })?;
        Ok(())
    }

    //TODO
    pub fn handle_events(&mut self) -> Result<bool> {
        let _ = crossterm::event::read()?;
        Ok(true)
    }
}
