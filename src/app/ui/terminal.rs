use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use image::Rgb;
use ratatui::DefaultTerminal;

use crate::app::parse_image_rgb;

pub fn initialise_terminal() -> Result<DefaultTerminal> {
    let terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    Ok(terminal)
}

pub const FOREGROUND_COLOR: u8 = 10;
pub const BACKGROUND_COLOR: u8 = 11;
const DELAY_MS: u64 = 20;
pub fn get_color(code: u8) -> Result<Rgb<u8>> {
    let string = xterm_query::query_osc(format!("\x1b]{code};?\x1b\\").as_str(), DELAY_MS)
        .wrap_err_with(|| format!("Failed to query ocs with code {code}"))?;
    let mut hex = String::new();
    (8..19)
        .step_by(5)
        .for_each(|i| hex.push_str(&string[i..(i + 2)]));
    parse_image_rgb(&hex).wrap_err("The terminal didn't give the correct code")
}
