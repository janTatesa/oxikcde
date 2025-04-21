use std::str::FromStr;

use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use ratatui::style::{Color, Modifier as StyleModifier, Style};
use serde::Deserialize;

use super::{Modifiers, parse_modifiers};
pub struct StylingConfig {
    pub title_style: Style,
    pub alt_text_style: Style,
    pub date_style: Style,
    pub errors_style: Style,
    pub messages_style: Style,
}

#[derive(Deserialize)]
pub struct StylingConfigRaw {
    title_style: String,
    alt_text_style: String,
    date_style: String,
    errors_style: String,
    messages_style: String,
}

impl StylingConfig {
    pub fn from_raw(raw: StylingConfigRaw) -> Result<Self> {
        Ok(Self {
            title_style: parse_style(&raw.title_style).ok_or_eyre(format!(
                "Failed to determine title style with config option {}",
                &raw.title_style
            ))?,
            alt_text_style: parse_style(&raw.alt_text_style).ok_or_eyre(format!(
                "Failed to determine alt text style with config option {}",
                &raw.alt_text_style
            ))?,
            date_style: parse_style(&raw.date_style).ok_or_eyre(format!(
                "Failed to determine date style with config option {}",
                &raw.date_style
            ))?,

            messages_style: parse_style(&raw.messages_style).ok_or_eyre(format!(
                "Failed to determine messages style with config option {}",
                &raw.messages_style
            ))?,
            errors_style: parse_style(&raw.errors_style).ok_or_eyre(format!(
                "Failed to determine errors style with config option {}",
                &raw.errors_style
            ))?,
        })
    }
}

const STYLE_MODIFIERS: Modifiers<StyleModifier, 8> = [
    ("bold", StyleModifier::BOLD),
    ("italic", StyleModifier::ITALIC),
    ("underlined", StyleModifier::UNDERLINED),
    ("slow_blink", StyleModifier::SLOW_BLINK),
    ("rapid_blink", StyleModifier::RAPID_BLINK),
    ("reversed", StyleModifier::REVERSED),
    ("hidden", StyleModifier::HIDDEN),
    ("crossed_out", StyleModifier::CROSSED_OUT),
];

fn parse_style(string: &str) -> Option<Style> {
    let mut split = string.split_whitespace();
    let color = Color::from_str(split.next()?).ok()?;
    let modifiers = parse_modifiers(split.collect(), STYLE_MODIFIERS);
    Some(Style::new().fg(color).add_modifier(modifiers))
}
