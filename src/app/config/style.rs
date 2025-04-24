use std::str::FromStr;

use color_eyre::{Result, eyre::ContextCompat};
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

#[macro_export]
macro_rules! styles {
    ($raw:ident, $( $style_name:ident ),* $(,)?) => {
        Self {
            $(
                $style_name: {
                    use color_eyre::eyre::WrapErr;
                    parse_style(&$raw.$style_name).wrap_err_with(|| format!(
                    concat!(
                        "Failed to determine ",
                        stringify!($style_name),
                        " with config option {}",
                    ),
                    &$raw.$style_name
                ))?},
            )*
        }
    };
}

impl StylingConfig {
    pub fn from_raw(raw: StylingConfigRaw) -> Result<Self> {
        Ok(styles![
            raw,
            title_style,
            alt_text_style,
            date_style,
            messages_style,
            errors_style,
        ])
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

fn parse_style(string: &str) -> Result<Style> {
    let mut split = string.split_whitespace();
    let color = Color::from_str(split.next().wrap_err("Expected color")?)?;
    let modifiers = parse_modifiers(split.collect(), STYLE_MODIFIERS)?;
    Ok(Style::new().fg(color).add_modifier(modifiers))
}
