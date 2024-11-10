use crate::{
    CommandToApp::{self, *},
    SwitchToComic::*,
};
use cli_log::debug;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub fn handle_events() -> Result<Option<CommandToApp>> {
    let event = event::read()?;
    debug!("Event: {:?}", event);
    match event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            Ok(handle_key_event(key_event.code))
        }
        Event::Resize(_, _) => Ok(Some(HandleResize)),
        _ => Ok(None),
    }
}
fn handle_key_event(code: KeyCode) -> Option<CommandToApp> {
    match code {
        KeyCode::Backspace => None,
        KeyCode::Enter => None,
        KeyCode::Left => Some(SwitchToComic(Previous)),
        KeyCode::Right => Some(SwitchToComic(Next)),
        KeyCode::Up => None,
        KeyCode::Down => None,
        KeyCode::Home => Some(SwitchToComic(First)),
        KeyCode::End => Some(SwitchToComic(Latest)),
        KeyCode::PageUp => None,
        KeyCode::PageDown => None,
        KeyCode::Tab => None,
        KeyCode::BackTab => None,
        KeyCode::Delete => None,
        KeyCode::Insert => None,
        KeyCode::F(_) => None,
        KeyCode::Char(char) => handle_char_keypress(char),
        KeyCode::Null => None,
        KeyCode::Esc => Some(Quit),
        KeyCode::CapsLock => None,
        KeyCode::ScrollLock => None,
        KeyCode::NumLock => None,
        KeyCode::PrintScreen => None,
        KeyCode::Pause => None,
        KeyCode::Menu => None,
        KeyCode::KeypadBegin => None,
        KeyCode::Media(_) => None,
        KeyCode::Modifier(_) => None,
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
        _ => None,
    }
}
