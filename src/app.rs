mod cli;
mod comic;
mod event;
mod ui;

use self::{CommandToApp::*, OpenInBrowser::*};
use clap::{builder::OsStr, ArgMatches, ValueEnum};
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
        let cli = cli();
        let mut comic_downloader = ComicDownloader::new()?;
        let (comic, image) = comic_downloader.switch(initial_switch_to_comic(&cli))?;
        let mut ui = Ui::new(image)?;
        ui.update(&comic, RenderOption::None)?;
        Self {
            comic_downloader,
            ui,
            comic,
        }
        .main_loop()
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
                let (comic, image) = match self.comic_downloader.switch(action) {
                    Ok((comic, image)) => (comic, image),
                    Err(error) => {
                        return self
                            .ui
                            .update(&self.comic, RenderOption::Error(error.to_string()))
                    }
                };

                self.comic = comic;
                return self.ui.update(&self.comic, RenderOption::NewComic(image));
            }
            BookmarkComic => self.comic_downloader.bookmark_comic(),
            OpenInBrowser(open_in_browser) => self.open_in_browser(open_in_browser)?,
            _ => {}
        };

        self.ui.update(&self.comic, command.into())?;
        Ok(())
    }

    fn open_in_browser(&self, open_in_browser: OpenInBrowser) -> Result<()> {
        open::that(match open_in_browser {
            Comic => format!("https://xkcd.com/{}", self.comic.number),
            Explanation => format!("https://explainxkcd.com/{}", self.comic.number),
        })?;
        Ok(())
    }
}

fn initial_switch_to_comic(cli: &ArgMatches) -> SwitchToComic {
    cli.get_one::<u64>("number")
        .map(|num| SwitchToComic::Specific(num.to_owned()))
        .unwrap_or_else(|| {
            cli.get_one::<SwitchToComic>("initial_comic")
                .unwrap()
                .to_owned()
        })
}

#[derive(Debug, Copy, Clone)]
enum CommandToApp {
    Quit,
    SwitchToComic(SwitchToComic),
    ToggleInvert,
    BookmarkComic,
    OpenInBrowser(OpenInBrowser),
}

#[derive(Debug, Copy, Clone)]
enum OpenInBrowser {
    Comic,
    Explanation,
}

impl From<CommandToApp> for RenderOption {
    fn from(val: CommandToApp) -> Self {
        match val {
            ToggleInvert => RenderOption::ToggleInvert,
            BookmarkComic => RenderOption::BookmarkComic,
            OpenInBrowser(open_in_browser) => RenderOption::OpenInBrowser(open_in_browser),
            _ => RenderOption::None,
        }
    }
}

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

impl From<SwitchToComic> for OsStr {
    // Required for having a default argument, we only need to implement it for latest
    fn from(value: SwitchToComic) -> Self {
        if let SwitchToComic::Latest = value {
            return "latest".into();
        }
        unreachable!()
    }
}
