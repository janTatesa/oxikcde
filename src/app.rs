use cli_log::info;
use color_eyre::Result;

use comic::ComicDownloader;
use ratatui::DefaultTerminal;
use ui::Ui;
mod comic;
mod ui;
use crate::{
    CommandToApp::{self, *},
    SwitchToComic,
};

pub struct App {
    comic_downloader: ComicDownloader,
    ui: Ui,
}

impl App {
    pub fn new(terminal: DefaultTerminal) -> Result<Self> {
        let mut comic_downloader = ComicDownloader::default();
        let mut ui = Ui::new(terminal)?;
        ui.render(comic_downloader.switch(SwitchToComic::Latest)?)?;
        Ok(Self {
            comic_downloader,
            ui,
        })
    }
    pub fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        info!("Performing {:?}", command);
        match command {
            SwitchToComic(action) => self.ui.render(self.comic_downloader.switch(action)?),
            HandleResize => self.ui.handle_resize(),
            Quit => Ok(()),
        }
    }
}
