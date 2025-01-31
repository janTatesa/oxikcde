use crate::app::CommandToApp;
use cli_log::debug;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use eyre::Result;
use std::collections::HashMap;

pub type Keybindings = HashMap<KeyEvent, CommandToApp>;
pub struct EventHandler(Keybindings);

impl EventHandler {
    pub fn new(keybindings: Keybindings) -> Self {
        Self(keybindings)
    }

    pub fn get_command(&self) -> Result<CommandToApp> {
        loop {
            let event = event::read()?;
            debug!("Event: {:?}", event);
            let command = match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.0.get(&key_event)
                }
                Event::Resize(..) => Some(&CommandToApp::HandleResize),
                _ => None,
            };

            if let Some(command) = command {
                return Ok(command.to_owned());
            }
        }
    }
}
