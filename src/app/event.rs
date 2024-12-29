use super::SwitchToComic;
use crate::app::{
    CommandToApp::{self, *},
    SwitchToComic::*,
};
use cli_log::debug;
use crossterm::event::{
    self, Event,
    KeyCode::{self, *},
    KeyEventKind,
};
use eyre::Result;

pub fn get_command() -> Result<CommandToApp> {
    loop {
        let event = event::read()?;
        debug!("Event: {:?}", event);
        let command = match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                get_key_event(key_event.code)
            }
            Event::Resize(..) => Some(Resize),
            _ => None,
        };

        if let Some(command) = command {
            return Ok(command);
        }
    }
}

fn get_key_event(code: KeyCode) -> Option<CommandToApp> {
    match code {
        Backspace => None,
        Enter => None,
        Left => Some(SwitchToComic(Previous)),
        Right => Some(SwitchToComic(Next)),
        Up => None,
        Down => None,
        Home => Some(SwitchToComic(First)),
        End => Some(SwitchToComic(Latest)),
        PageUp => None,
        PageDown => None,
        Tab => None,
        BackTab => None,
        Delete => None,
        Insert => None,
        F(_) => None,
        Char(char) => handle_char_keypress(char),
        Null => None,
        Esc => Some(Quit),
        CapsLock => None,
        ScrollLock => None,
        NumLock => None,
        PrintScreen => None,
        Pause => None,
        Menu => None,
        KeypadBegin => None,
        Media(_) => None,
        Modifier(_) => None,
    }
}

fn handle_char_keypress(char: char) -> Option<CommandToApp> {
    match char {
        'q' => Some(Quit),
        'p' => Some(SwitchToComic(Previous)),
        'n' => Some(SwitchToComic(Next)),
        'f' => Some(SwitchToComic(First)),
        'l' => Some(SwitchToComic(Latest)),
        'i' => Some(ToggleInvert),
        'b' => Some(BookmarkComic),
        'r' => Some(SwitchToComic(Random)),
        'd' => Some(SwitchToComic(LastSeen)),
        _ => None,
    }
}
