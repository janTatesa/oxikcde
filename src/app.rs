mod comic;
mod config;
pub use config::print_default_config;
mod event;
mod state;
mod ui;

use clap::{ArgMatches, ValueEnum};
use cli_log::info;
use color_eyre::{Result, eyre::Context};
use colors_transform::Color;
use comic::*;
use config::Config;
use core::panic;
use event::EventHandler;
use image::{DynamicImage, Rgb};
use rand::{rngs::ThreadRng, thread_rng};
use state::State;
use std::path::PathBuf;
use strum::{Display, EnumString};
use ui::*;

pub struct App {
    state: State,
    rng: ThreadRng,
    process_image: bool,
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
        let mut rng = thread_rng();
        let mut state = State::new();
        state.last_seen_comic = get_comic_number(
            &mut rng,
            &mut state,
            initial_switch_to_comic(config.initial_comic, &cli),
        )?;
        let (comic, image) = download(&state)?;
        let mut ui = Ui::new(image, config.styling, config.terminal, config.keep_colors)
            .wrap_err("Failed to initialise ui")?;
        ui.update(&comic, true, RenderOption::None)?;
        Self {
            state,
            rng,
            process_image: true,
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
                return self.state.save();
            }
            self.handle_command(command)?
        }
    }

    fn switch_to_comic(&mut self, switch_to_comic: SwitchToComic) -> Result<(Comic, DynamicImage)> {
        self.state.last_seen_comic =
            get_comic_number(&mut self.rng, &mut self.state, switch_to_comic)?;
        download(&self.state)
    }
    fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        let render_option = match command {
            CommandToApp::SwitchToComic(switch_to_comic) => {
                match self.switch_to_comic(switch_to_comic) {
                    Ok((comic, image)) => {
                        self.comic = comic;
                        RenderOption::NewComic(image)
                    }
                    Err(error) => RenderOption::ShowError(error.to_string()),
                }
            }
            CommandToApp::ToggleBookmark => {
                RenderOption::ShowMessage(if self.state.toggle_bookmark() {
                    "Bookmarked comic"
                } else {
                    "Unbookmarked comic"
                })
            }
            CommandToApp::OpenInBrowser(open_in_browser) => {
                self.open_in_browser(open_in_browser)?;
                RenderOption::ShowMessage(match open_in_browser {
                    OpenInBrowser::Comic => "Opened comic in your browser!",
                    OpenInBrowser::Explanation => "Opened explanation in your browser!",
                })
            }
            CommandToApp::None => return Ok(()),
            CommandToApp::HandleResize => RenderOption::None,
            CommandToApp::Quit => panic!(),
            CommandToApp::ToggleProcessing => {
                self.process_image = !self.process_image;
                RenderOption::ShowMessage(if self.process_image {
                    "Image processing on"
                } else {
                    "Image processing off"
                })
            }
        };

        self.ui
            .update(&self.comic, self.process_image, render_option)
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

fn initial_switch_to_comic(default: SwitchToComic, cli: &ArgMatches) -> SwitchToComic {
    cli.get_one::<u16>("number")
        .map(|num| SwitchToComic::Specific(num.to_owned()))
        .or_else(|| cli.get_one::<SwitchToComic>("initial_comic").copied())
        .unwrap_or(default)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
enum CommandToApp {
    Quit,
    #[strum(disabled)]
    SwitchToComic(SwitchToComic),
    ToggleProcessing,
    ToggleBookmark,
    #[strum(disabled)]
    OpenInBrowser(OpenInBrowser),
    HandleResize,
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
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
    Specific(u16),
    LastSeen,
}

fn parse_image_rgb(str: &str) -> Option<Rgb<u8>> {
    let rgb = colors_transform::Rgb::from_hex_str(str).ok()?.as_tuple();
    Some(Rgb::from([rgb.0 as u8, rgb.1 as u8, rgb.2 as u8]))
}
