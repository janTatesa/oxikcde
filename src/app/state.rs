use cli_log::error;
use color_eyre::{eyre::ContextCompat, Result};
use dirs::{data_dir, state_dir};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tap::Tap;

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub last_seen_comic: u16,
    pub minimum_latest_comic_number: u16,
    bookmarked_comic: Option<u16>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_seen_comic: 1,
            minimum_latest_comic_number: 1,
            bookmarked_comic: None,
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::try_new()
            .map_err(|error| {
                error!(
                    "Failed to read comic downloader data in {}: {error}. Using default values",
                    get_path_to_state_file()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            })
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = get_path_to_state_file()?;
        fs::create_dir_all(path.parent().unwrap())?;
        Ok(fs::write(path, serde_json::to_string(self).unwrap())?)
    }

    fn try_new() -> Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(
            get_path_to_state_file()?,
        )?)?)
    }

    // TODO: return an enum
    pub fn toggle_bookmark(&mut self) -> bool {
        self.bookmarked_comic = if self.bookmarked_comic == Some(self.last_seen_comic) {
            None
        } else {
            Some(self.last_seen_comic)
        };
        self.bookmarked_comic.is_some()
    }

    pub fn bookmarked_comic(&self) -> Option<u16> {
        self.bookmarked_comic
    }
}

fn get_path_to_state_file() -> Result<PathBuf> {
    Ok(state_dir()
        .or(data_dir())
        .wrap_err("Unsupported platform")?
        .tap_mut(|path| path.extend(["oxikcde", "state.json"])))
}
