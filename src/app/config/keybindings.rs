use crate::{
    SwitchToComic,
    app::{CommandToApp, OpenInBrowser, event::Keybindings},
};
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
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
            parse_key_event(event).ok_or_eyre(format!("Failed to parse keybinding {}", event))?,
            CommandToApp::parse(command)
                .ok_or_eyre(format!("Failed to parse action {}", command))?,
        );
    }
    Ok(keybindings)
}

const MINUS: KeyEvent = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::empty());
pub fn parse_key_event(string: &str) -> Option<KeyEvent> {
    if string == "-" {
        return Some(MINUS);
    }

    let mut split: Vec<&str> = string.split("-").collect();
    let keycode = parse_key_code(split.pop()?)?;
    let modifiers = KeyModifiers::from_bits(parse_modifiers(split, KEY_MODIFIERS).bits())
        .expect("KeyModifiers should always be parsed correctly");
    Some(KeyEvent::new(keycode, modifiers))
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

fn parse_key_code(string: &str) -> Option<KeyCode> {
    match string {
        "minus" => Some(Char('-')),
        "backspace" => Some(Backspace),
        "space" => Some(Char(' ')),
        "ret" => Some(Enter),
        "left" => Some(Left),
        "right" => Some(Right),
        "up" => Some(Up),
        "down" => Some(Down),
        "home" => Some(Home),
        "end" => Some(End),
        "pageup" => Some(PageUp),
        "pagedown" => Some(PageDown),
        "tab" => Some(Tab),
        "del" => Some(Delete),
        "ins" => Some(Insert),
        "null" => Some(Null),
        "esc" => Some(Esc),
        character if character.len() == 1 => Some(KeyCode::Char(character.chars().next()?)),
        _ => None,
    }
}
