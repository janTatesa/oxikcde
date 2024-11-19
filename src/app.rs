use clap::{ArgMatches, ValueEnum};
use cli_log::info;
use color_eyre::Result;
use comic::ComicDownloader;
use event::handle_events;
use ratatui::DefaultTerminal;
use ui::Ui;
mod comic;
mod event;
mod ui;
use self::CommandToApp::*;
pub fn app_loop(terminal: DefaultTerminal, cli: ArgMatches) -> Result<()> {
    let mut app = App::new(terminal, cli)?;
    loop {
        if let Some(command) = handle_events()? {
            app.handle_command(command)?;
            if let CommandToApp::Quit = command {
                break;
            }
        }
    }
    Ok(())
}

struct App {
    comic_downloader: ComicDownloader,
    ui: Ui,
}

impl App {
    fn new(terminal: DefaultTerminal, cli: ArgMatches) -> Result<Self> {
        let (comic_downloader, (comic, comic_number)) =
            ComicDownloader::new(match cli.get_one::<u64>("number") {
                Some(num) => SwitchToComic::Specific(num.to_owned()),
                _ => cli
                    .get_one::<SwitchToComic>("initial_comic")
                    .unwrap_or(&SwitchToComic::Latest)
                    .to_owned(),
            });
        let ui = Ui::new(terminal, comic, comic_number)?;
        Ok(Self {
            comic_downloader,
            ui,
        })
    }
    fn handle_command(&mut self, command: CommandToApp) -> Result<()> {
        info!("Performing {:?}", command);
        match command {
            SwitchToComic(action) => {
                let (comic, comic_number) = self.comic_downloader.switch(action);
                self.ui.render_new_comic(comic, comic_number)
            }
            HandleResize => self.ui.handle_resize(),
            Quit => self.comic_downloader.save(),
            ToggleInvert => self.ui.toggle_invert(),
            BookmarkComic => {
                self.comic_downloader.bookmark_comic();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CommandToApp {
    Quit,
    SwitchToComic(SwitchToComic),
    HandleResize,
    ToggleInvert,
    BookmarkComic,
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
