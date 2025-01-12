use colors_transform::Color;
use crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use eyre::Result;
use image::Rgb;
use ratatui::DefaultTerminal;

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
    let string = xterm_query::query_osc(format!("\x1b]{code};?\x1b\\").as_str(), DELAY_MS)?;
    let mut hex = String::new();
    (8..19)
        .step_by(5)
        .for_each(|i| hex.push_str(&string[i..(i + 2)]));
    let rgb = colors_transform::Rgb::from_hex_str(&hex)
        .unwrap()
        .as_tuple();
    Ok(Rgb::from([rgb.0 as u8, rgb.1 as u8, rgb.2 as u8]))
}
