mod app;

use app::App;
use cli_log::init_cli_log;
use eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    let result = App::run();
    ratatui::restore();
    result
}
