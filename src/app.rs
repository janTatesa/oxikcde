mod comic;
pub mod config;

mod state;
mod ui;

use clap::{ArgMatches, ValueEnum};
use cli_log::info;
use color_eyre::{Result, eyre::Context};
use colors_transform::Color;
use comic::*;
use config::Config;
use crossterm::event::{Event, EventStream, KeyEvent};
use futures::future::Fuse;
use futures::{FutureExt, StreamExt};
use image::{DynamicImage, Rgb};
use rand::{rngs::ThreadRng, thread_rng};
use state::State;
use std::path::PathBuf;
use std::time::Duration;
use std::{collections::HashMap, panic};
use strum::{Display, EnumString};
use tokio::time::{Interval, interval};
use tokio::{select, time};
use ui::*;

type Keybindings = HashMap<KeyEvent, CommandToApp>;

type JoinHandle<T> = Fuse<tokio::task::JoinHandle<Result<T>>>;
pub struct App {
    running: bool,
    state: State,
    rng: ThreadRng,
    process_image: bool,
    event_stream: EventStream,
    keybindings: Keybindings,
    xkcd_url: String,
    explanation_url: String,
    ui: Ui,
    comic: Comic,
    image_join_handle: JoinHandle<DynamicImage>,
    delete_message_interval: Interval,
}

const MESSAGE_DURATION: Duration = Duration::from_secs(2);
const WAIT_DURATION: Duration = Duration::from_millis(100);
impl App {
    pub async fn run(cli: ArgMatches) -> Result<()> {
        let config = Config::new(
            cli.get_one::<PathBuf>("config_path")
                .expect("Option has default value"),
        )
        .wrap_err("Failed to parse config")?;
        let mut rng = thread_rng();
        let mut state = State::new();
        state.current_comic = get_comic_number(
            &mut rng,
            &state,
            initial_switch_to_comic(config.initial_comic, &cli),
        )
        .await?;
        let comic = download(state.current_comic).await?;
        let ui = Ui::new(config.styling, config.terminal, config.keep_colors)
            .wrap_err("Failed to initialise ui")
            .and_then(|mut ui| {
                ui.update(&comic, true, RenderOption::None)?;
                Ok(ui)
            })?;
        Self {
            state,
            rng,
            process_image: true,
            ui,
            image_join_handle: tokio::spawn(download_image(comic.image_url().to_string())).fuse(),
            comic,
            event_stream: EventStream::new(),
            keybindings: config.keybindings,
            xkcd_url: config.url,
            explanation_url: config.explanation_url,
            running: true,
            delete_message_interval: interval(MESSAGE_DURATION),
        }
        .main_loop()
        .await
    }

    async fn main_loop(mut self) -> Result<()> {
        while self.running {
            select! {
                    Some(result) = self.event_stream.next().fuse() => {self.handle_crossterm_event(result?).await?}
                    image_download_result = &mut self.image_join_handle => {self.on_new_image(image_download_result.unwrap())?},
                    _ = self.delete_message_interval.tick() => self.ui.update(&self.comic, self.process_image, RenderOption::DeleteMessage)?,
                    _ = time::sleep(WAIT_DURATION) => {
                        // Sleep for a short duration to avoid busy waiting.
                    }
            }
        }
        info!("Quiting");
        self.state.save()?;
        Ok(())
    }

    async fn switch_to_comic(&mut self, switch_to_comic: SwitchToComic) -> Result<()> {
        let number = get_comic_number(&mut self.rng, &self.state, switch_to_comic).await?;
        if number != self.state.current_comic {
            self.state.current_comic = number;
            let comic = download(number).await?;
            self.image_join_handle =
                tokio::spawn(download_image(comic.image_url().to_string())).fuse();
            self.comic = comic;
            self.ui.clear_image_protocols();
        };
        Ok(())
    }

    fn on_new_image(&mut self, comic_download_result: Result<DynamicImage>) -> Result<()> {
        let render_option = match comic_download_result {
            Ok(image) => RenderOption::NewImage(image),
            Err(error) => RenderOption::ShowError(error.to_string()),
        };

        self.update_ui(render_option)
    }

    async fn handle_crossterm_event(&mut self, event: Event) -> Result<()> {
        let command = match event {
            Event::Key(key_event) => match self.keybindings.get(&key_event) {
                Some(command) => *command,
                None => return Ok(()),
            },
            Event::Resize(_, _) => CommandToApp::HandleResize,
            _ => return Ok(()),
        };

        self.handle_command(command).await
    }

    async fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        let render_option = match command {
            CommandToApp::SwitchToComic(switch_to_comic) => {
                match self.switch_to_comic(switch_to_comic).await {
                    Ok(_) => RenderOption::None,
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
            CommandToApp::Quit => {
                self.running = false;
                return Ok(());
            }
            CommandToApp::ToggleProcessing => {
                self.process_image = !self.process_image;
                RenderOption::ShowMessage(if self.process_image {
                    "Image processing on"
                } else {
                    "Image processing off"
                })
            }
        };
        self.update_ui(render_option)
    }

    fn update_ui(&mut self, render_option: RenderOption) -> Result<()> {
        if let RenderOption::ShowError(_) | RenderOption::ShowMessage(_) = render_option {
            self.delete_message_interval.reset();
        }
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
