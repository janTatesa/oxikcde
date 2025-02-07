use color_eyre::eyre::{OptionExt, Result};
use image::Rgb;
use serde::Deserialize;

use crate::app::parse_image_rgb;

pub struct TerminalConfig {
    pub foreground_color: Option<Rgb<u8>>,
    pub background_color: Option<Rgb<u8>>,
}

#[derive(Deserialize)]
pub struct TerminalConfigRaw {
    foreground_color: String,
    background_color: String,
}

impl TerminalConfig {
    pub fn from_raw(raw: TerminalConfigRaw) -> Result<Self> {
        Ok(Self {
            foreground_color: parse_color(&raw.foreground_color).ok_or_eyre(format!(
                "Failed to determine foreground color with config option {}",
                &raw.foreground_color
            ))?,
            background_color: parse_color(&raw.background_color).ok_or_eyre(format!(
                "Failed to determine background color with config option {}",
                &raw.background_color
            ))?,
        })
    }
}

fn parse_color(string: &str) -> Option<Option<Rgb<u8>>> {
    Some(match string {
        "query" => None,
        _ => Some(parse_image_rgb(string)?),
    })
}
