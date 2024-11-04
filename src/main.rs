use clap::Parser;
use color_eyre::Result;
use ratatui::Terminal;

mod app;
mod comic;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new(terminal)?.run();
    ratatui::restore();
    result
}
