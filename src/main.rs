use app::app_loop;
use cli::cli;
use cli_log::init_cli_log;
use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};

mod app;
mod cli;
fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    let cli = cli();
    let terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let result = app_loop(terminal, cli);
    ratatui::restore();
    result
}
