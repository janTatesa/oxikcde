mod cli;
mod comic;
mod event;
mod ui;

use self::CommandToApp::*;
use clap::ValueEnum;
use cli::cli;
use cli_log::info;
use comic::{Comic, ComicDownloader};
use event::get_command;
use eyre::Result;
use ui::{RenderOption, Ui};

pub struct App {
    comic_downloader: ComicDownloader,
    ui: Ui,
    comic: Comic,
}

impl App {
    pub fn run() -> Result<()> {
        let mut ui = Ui::new()?;
        let mut comic_downloader = ComicDownloader::new()?;
        let comic = comic_downloader.switch(Self::initial_switch_to_comic())?;
        ui.update(&comic, RenderOption::None)?;
        Self {
            comic_downloader,
            ui,
            comic,
        }
        .main_loop()
    }

    fn initial_switch_to_comic() -> SwitchToComic {
        let cli = cli();
        cli.get_one::<u64>("number")
            .map(|num| SwitchToComic::Specific(num.to_owned()))
            .unwrap_or_else(|| {
                cli.get_one::<SwitchToComic>("initial_comic")
                    .unwrap_or(&SwitchToComic::Latest)
                    .to_owned()
            })
    }

    fn main_loop(mut self) -> Result<()> {
        loop {
            let command = get_command()?;
            info!("Performing {:?}", command);
            if let Quit = command {
                return self.comic_downloader.save_data();
            }
            self.handle_command(command)?
        }
    }

    fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        match command {
            SwitchToComic(action) => {
                self.comic = match self.comic_downloader.switch(action) {
                    Ok(comic) => comic,
                    Err(error) => {
                        return self
                            .ui
                            .update(&self.comic, RenderOption::Error(error.to_string()))
                    }
                }
            }
            BookmarkComic => self.comic_downloader.bookmark_comic(),
            _ => {}
        };

        self.ui.update(&self.comic, command.into())?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum CommandToApp {
    Quit,
    SwitchToComic(SwitchToComic),
    Resize,
    ToggleInvert,
    BookmarkComic,
}

impl From<CommandToApp> for RenderOption {
    fn from(val: CommandToApp) -> Self {
        match val {
            Resize => RenderOption::Resize,
            ToggleInvert => RenderOption::ToggleInvert,
            BookmarkComic => RenderOption::BookmarkComic,
            _ => RenderOption::None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum SwitchToComic {
    #[clap(skip)]
    Next,
    #[clap(skip)]
    Previous,
    Latest,
    First,
    Random,
    Bookmarked,
    #[clap(skip)]
    Specific(u64),
    LastSeen,
}
