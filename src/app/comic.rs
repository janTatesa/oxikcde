use crate::app::SwitchToComic::{self, *};
use cli_log::error;
use color_eyre::{eyre::Context, Result};
use dirs::state_dir;
use image::DynamicImage;
use isahc::ReadResponseExt;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Display, fs, path::PathBuf};

#[derive(Clone)]
pub struct Comic {
    name: String,
    number: u64,
    alt_text: String,
    date_uploaded: String,
    interactive: bool,
}

impl Comic {
    pub fn new(number: u64, json: Value) -> Option<Self> {
        let alt_text = json["alt"].as_str()?.to_string();
        let name = json["title"].as_str()?.to_owned();
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str()?,
            json["month"].as_str()?.parse::<u16>().ok()?,
            json["day"].as_str()?.parse::<u16>().ok()?,
        );
        Some(Self {
            name,
            number,
            alt_text,
            date_uploaded,
            interactive: !json["extra_parts"].is_null(),
        })
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn alt_text(&self) -> &str {
        &self.alt_text
    }

    pub fn date_uploaded(&self) -> &str {
        &self.date_uploaded
    }
}

impl Display for Comic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}{}",
            self.number,
            self.name,
            if self.interactive {
                " (interactive)"
            } else {
                ""
            }
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComicDownloader {
    minimum_latest_comic_number: u64,
    last_seen_comic: u64,
    bookmarked_comic: Option<u64>,
    #[serde(skip)]
    rng: ThreadRng,
}
impl Default for ComicDownloader {
    fn default() -> Self {
        Self {
            minimum_latest_comic_number: 1,
            last_seen_comic: 1,
            bookmarked_comic: None,
            rng: thread_rng(),
        }
    }
}

impl ComicDownloader {
    pub fn new() -> Self {
        Self::try_new()
            .map_err(|error| {
                error!(
                    "Failed to read comic downloader data in {}: {error}. Using default values",
                    get_path_to_state_file().to_string_lossy()
                )
            })
            .unwrap_or_default()
    }

    fn try_new() -> Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(
            get_path_to_state_file(),
        )?)?)
    }

    pub fn switch(&mut self, switch_to_comic: SwitchToComic) -> Result<(Comic, DynamicImage)> {
        self.last_seen_comic = self.get_comic_number(switch_to_comic)?;
        self.download()
            .wrap_err_with(|| format!("Failed to download comic {}", self.last_seen_comic))
    }

    fn download(&self) -> Result<(Comic, DynamicImage)> {
        let json = download_json(Some(self.last_seen_comic))?;
        let image_url = json["img"]
            .as_str()
            .expect("XKCD should always return valid json");
        let image_bytes = &isahc::get(image_url)?.bytes()?;
        let image = image::load_from_memory(image_bytes)?;
        Ok((
            Comic::new(self.last_seen_comic, json).expect("XKCD should always return valid json"),
            image,
        ))
    }

    fn get_comic_number(&mut self, switch_to_comic: SwitchToComic) -> Result<u64> {
        Ok(match switch_to_comic {
            Next => {
                if self.minimum_latest_comic_number > self.last_seen_comic
                    || self.get_latest_comic_number()? > self.last_seen_comic
                {
                    self.last_seen_comic + 1
                } else {
                    self.last_seen_comic
                }
            }
            Previous => {
                if self.last_seen_comic > 1 {
                    self.last_seen_comic - 1
                } else {
                    1
                }
            }
            Latest => self.get_latest_comic_number()?,
            First => 1,
            Random => {
                let latest = self.get_latest_comic_number()?;
                self.rng.gen_range(1..latest)
            }
            Bookmarked => self.bookmarked_comic.unwrap_or(self.last_seen_comic),
            Specific(num) => num,
            LastSeen => self.last_seen_comic,
        })
    }

    pub fn bookmark_comic(&mut self) {
        self.bookmarked_comic = Some(self.last_seen_comic);
    }

    pub fn save_data(&self) -> Result<()> {
        let path = get_path_to_state_file();
        fs::create_dir_all(path.parent().unwrap())?;
        Ok(fs::write(path, serde_json::to_string(self).unwrap())?)
    }

    fn get_latest_comic_number(&mut self) -> Result<u64> {
        let json = download_json(None).wrap_err("Failed to determine latest comic number")?;
        let num = json["num"]
            .as_u64()
            .expect("XKCD should always return valid json");
        self.minimum_latest_comic_number = num;
        Ok(num)
    }
}

fn get_path_to_state_file() -> PathBuf {
    let mut path = state_dir().unwrap_or_default();
    path.push("oxikcde");
    path.push("comic_downloader.json");
    path
}

fn download_json(number: Option<u64>) -> Result<Value> {
    let text = isahc::get(match number {
        Some(number) => format!("https://xkcd.com/{number}/info.0.json"),
        _ => String::from("https://xkcd.com/info.0.json"),
    })?
    .text()?;

    Ok(serde_json::from_str(&text)?)
}
