use app::App;
use cli_log::init_cli_log;
use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use event::handle_events;
use ratatui::DefaultTerminal;

mod app;
mod event;

#[derive(Debug, Copy, Clone)]
enum CommandToApp {
    Quit,
    SwitchToComic(SwitchToComic),
    HandleResize,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum SwitchToComic {
    Next,
    Previous,
    Latest,
    First,
    Random,
    Bookmarked,
    Specific(u32),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    let terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let result = app_loop(terminal);
    ratatui::restore();
    result
}

fn app_loop(terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new(terminal)?;
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
