use app::App;
use cli_log::init_cli_log;
use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};

mod app;
mod comic;

fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    let terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let result = App::run(terminal);
    ratatui::restore();
    result
}
