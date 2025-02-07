mod comic;
mod config;
pub use config::print_default_config;
mod event;
mod ui;

use clap::{ArgMatches, ValueEnum};
use cli_log::info;
use color_eyre::{eyre::Context, Result};
use colors_transform::Color;
use comic::*;
use config::Config;
use event::EventHandler;
use image::Rgb;
use std::path::PathBuf;
use strum::{Display, EnumString};
use ui::*;

pub struct App {
    comic_downloader: ComicDownloader,
    event_handler: EventHandler,
    xkcd_url: String,
    explanation_url: String,
    ui: Ui,
    comic: Comic,
}

impl App {
    pub fn run(cli: ArgMatches) -> Result<()> {
        let config = Config::new(
            cli.get_one::<PathBuf>("config_path")
                .expect("Option has default value"),
        )
        .wrap_err("Failed to parse config")?;
        let mut comic_downloader = ComicDownloader::new();
        let (comic, image) =
            comic_downloader.switch(initial_switch_to_comic(&config.initial_comic, &cli))?;
        let mut ui = Ui::new(image, config.styling, config.terminal, config.keep_colors)
            .wrap_err("Failed to initialise ui")?;
        ui.update(&comic, RenderOption::None)?;
        Self {
            comic_downloader,
            ui,
            comic,
            event_handler: EventHandler::new(config.keybindings),

            xkcd_url: config.url,
            explanation_url: config.explanation_url,
        }
        .main_loop()
    }

    fn main_loop(mut self) -> Result<()> {
        loop {
            let command = self.event_handler.get_command()?;
            info!("Performing {:?}", command);
            if let CommandToApp::Quit = command {
                return self.comic_downloader.save_data();
            }
            self.handle_command(command)?
        }
    }

    fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        match command {
            CommandToApp::SwitchToComic(switch_to_comic) => {
                return self.switch_to_comic(switch_to_comic)
            }
            CommandToApp::BookmarkComic => self.comic_downloader.bookmark_comic(),
            CommandToApp::OpenInBrowser(open_in_browser) => {
                self.open_in_browser(open_in_browser)?
            }
            CommandToApp::None => return Ok(()),
            _ => {}
        };

        self.update_ui(command.into())?;
        Ok(())
    }

    fn switch_to_comic(&mut self, switch_to_comic: SwitchToComic) -> Result<()> {
        let option = match self.comic_downloader.switch(switch_to_comic) {
            Ok((comic, image)) => {
                self.comic = comic;
                RenderOption::NewComic(image)
            }
            Err(error) => RenderOption::ShowError(error.to_string()),
        };

        self.update_ui(option)
    }

    fn update_ui(&mut self, option: RenderOption) -> Result<()> {
        self.ui.update(&self.comic, option)
    }

    fn open_in_browser(&self, open_in_browser: OpenInBrowser) -> Result<()> {
        open::that(format!(
            "{}{}",
            match open_in_browser {
                OpenInBrowser::Comic => &self.xkcd_url,
                OpenInBrowser::Explanation => &self.explanation_url,
            },
            self.comic.number(),
        ))?;
        Ok(())
    }
}

fn initial_switch_to_comic(default: &SwitchToComic, cli: &ArgMatches) -> SwitchToComic {
    cli.get_one::<u64>("number")
        .map(|num| SwitchToComic::Specific(num.to_owned()))
        .unwrap_or_else(|| *cli.get_one("initial_comic").unwrap_or(default))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
enum CommandToApp {
    Quit,
    #[strum(disabled)]
    SwitchToComic(SwitchToComic),
    ToggleProcessing,
    BookmarkComic,
    #[strum(disabled)]
    OpenInBrowser(OpenInBrowser),
    HandleResize,
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
enum OpenInBrowser {
    Comic,
    Explanation,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display, EnumString, ValueEnum)]
#[strum(serialize_all = "snake_case")]
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

impl From<CommandToApp> for RenderOption {
    fn from(val: CommandToApp) -> Self {
        match val {
            CommandToApp::ToggleProcessing => RenderOption::ToggleProcessing,
            CommandToApp::BookmarkComic => RenderOption::ShowBookmarkComicMessage,
            CommandToApp::OpenInBrowser(open_in_browser) => {
                RenderOption::ShowOpenInBrowserMessage(open_in_browser)
            }
            _ => RenderOption::None,
        }
    }
}

fn parse_image_rgb(str: &str) -> Option<Rgb<u8>> {
    let rgb = colors_transform::Rgb::from_hex_str(str).ok()?.as_tuple();
    Some(Rgb::from([rgb.0 as u8, rgb.1 as u8, rgb.2 as u8]))
}
