use color_eyre::Result;
use ratatui::{layout::Rect, widgets::Block, DefaultTerminal};
use ratatui_image::{picker::Picker, StatefulImage};

use crate::comic::Comic;

mod event;

pub struct App {
    image_picker: Picker,
    terminal: DefaultTerminal,
    comic: Comic,
}

//TODO: Currently the app prints one xkcd and exits. Implement a TUI
impl App {
    pub fn run(terminal: DefaultTerminal) -> Result<()> {
        let image_picker = Picker::from_query_stdio()?;
        let comic = Comic::download(None)?;
        let mut app = Self {
            image_picker,
            terminal,
            comic,
        };

        loop {
            app.render()?;
            // Returns true if the user wants to exit
            if app.handle_events()? {
                break;
            }
        }

        Ok(())
    }
    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::new()
                    .title_top(self.comic.title())
                    .title_top(self.comic.date_uploaded())
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

    fn switch_to_comic(&mut self, switch_to_comic: SwitchToComic) -> Result<()> {
        let number = match switch_to_comic {
            SwitchToComic::Next => Some(self.comic.number + 1),
            SwitchToComic::Previous => Some(self.comic.number - 1),
            SwitchToComic::Latest => None,
            SwitchToComic::First => Some(1),
            SwitchToComic::Random => todo!(),
            SwitchToComic::Bookmarked => todo!(),
            SwitchToComic::Specific(num) => Some(num),
        };
        self.comic = Comic::download(number)?;
        Ok(())
    }
}

#[allow(dead_code)]
enum SwitchToComic {
    Next,
    Previous,
    Latest,
    First,
    Random,
    Bookmarked,
    Specific(u32),
}
