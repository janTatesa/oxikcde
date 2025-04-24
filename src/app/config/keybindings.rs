use crate::{
    SwitchToComic,
    app::{CommandToApp, Keybindings, OpenInBrowser},
};
use color_eyre::eyre::{Context, ContextCompat, OptionExt};
use color_eyre::{Result, eyre::eyre};
use crossterm::event::{
    KeyCode::{self, *},
    KeyEvent, KeyModifiers,
};
use std::{collections::HashMap, str::FromStr};

use super::{Modifiers, parse_modifiers};

type Type = HashMap<String, String>;

pub fn parse_keybindings(raw: Type) -> Result<Keybindings> {
    let mut keybindings: Keybindings = HashMap::with_capacity(raw.len());
    for (event, command) in raw.iter() {
        keybindings.insert(
            parse_key_event(event)
                .wrap_err_with(|| format!("Failed to parse keybinding {}", event))?,
            CommandToApp::parse(command)
                .ok_or_eyre(format!("Failed to parse action {}", command))?,
        );
    }
    Ok(keybindings)
}

const MINUS: KeyEvent = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::empty());
pub fn parse_key_event(string: &str) -> Result<KeyEvent> {
    if string == "-" {
        return Ok(MINUS);
    }

    let mut split: Vec<&str> = string.split("-").collect();
    let keycode = parse_key_code(split.pop().wrap_err("Expected key")?)?;
    let modifiers = KeyModifiers::from_bits(parse_modifiers(split, KEY_MODIFIERS)?.bits())
        .expect("KeyModifiers should always be parsed correctly");
    Ok(KeyEvent::new(keycode, modifiers))
}

const KEY_MODIFIERS: Modifiers<KeyModifiers, 3> = [
    ("C", KeyModifiers::CONTROL),
    ("S", KeyModifiers::SHIFT),
    ("A", KeyModifiers::ALT),
];

impl CommandToApp {
    pub fn parse(string: &str) -> Option<Self> {
        let mut split = string.split_whitespace();
        let parsed = match split.next()? {
            "switch_to_comic" => Self::SwitchToComic(SwitchToComic::from_str(split.next()?).ok()?),
            "open_in_browser" => Self::OpenInBrowser(OpenInBrowser::from_str(split.next()?).ok()?),
            command => Self::from_str(command).ok()?,
        };
        Some(parsed)
    }
}

fn parse_key_code(string: &str) -> Result<KeyCode> {
    match string {
        "minus" => Ok(Char('-')),
        "backspace" => Ok(Backspace),
        "space" => Ok(Char(' ')),
        "ret" => Ok(Enter),
        "left" => Ok(Left),
        "right" => Ok(Right),
        "up" => Ok(Up),
        "down" => Ok(Down),
        "home" => Ok(Home),
        "end" => Ok(End),
        "pageup" => Ok(PageUp),
        "pagedown" => Ok(PageDown),
        "tab" => Ok(Tab),
        "del" => Ok(Delete),
        "ins" => Ok(Insert),
        "null" => Ok(Null),
        "esc" => Ok(Esc),
        character if character.len() == 1 => Ok(KeyCode::Char(character.chars().next().unwrap())),
        invalid => Err(eyre!("Invalid key {invalid}")),
    }
}
