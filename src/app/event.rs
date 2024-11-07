use super::{App, SwitchToComic::*};
use cli_log::{debug, info};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
impl App {
    pub(super) fn handle_events(&mut self) -> Result<bool> {
        let event = event::read()?;
        debug!("Event: {:?}", event);
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event.code)
            }

            _ => Ok(false),
        }
    }
    fn handle_key_event(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Backspace => {}
            KeyCode::Enter => {}
            KeyCode::Left => self.switch_to_comic(Previous)?,
            KeyCode::Right => self.switch_to_comic(Next)?,
            KeyCode::Up => {}
            KeyCode::Down => {}
            KeyCode::Home => self.switch_to_comic(First)?,
            KeyCode::End => self.switch_to_comic(Latest)?,
            KeyCode::PageUp => {}
            KeyCode::PageDown => {}
            KeyCode::Tab => {}
            KeyCode::BackTab => {}
            KeyCode::Delete => {}
            KeyCode::Insert => {}
            KeyCode::F(_) => {}
            KeyCode::Char(char) => return self.handle_char_keypress(char),
            KeyCode::Null => {}
            KeyCode::Esc => {
                info!("Quiting");
                return Ok(true);
            }
            KeyCode::CapsLock => {}
            KeyCode::ScrollLock => {}
            KeyCode::NumLock => {}
            KeyCode::PrintScreen => {}
            KeyCode::Pause => {}
            KeyCode::Menu => {}
            KeyCode::KeypadBegin => {}
            KeyCode::Media(_) => {}
            KeyCode::Modifier(_) => {}
        };
        Ok(false)
    }

    fn handle_char_keypress(&mut self, char: char) -> Result<bool> {
        match char {
            'q' => {
                info!("Quiting");
                return Ok(true);
            }
            'p' => self.switch_to_comic(Previous)?,
            'n' => self.switch_to_comic(Next)?,
            'f' => self.switch_to_comic(First)?,
            'l' => self.switch_to_comic(Latest)?,
            _ => {}
        }
        Ok(false)
    }
}
