use crate::app::SwitchToComic::{self, *};
use dirs::state_dir;
use eyre::{eyre, Result};
use image::DynamicImage;
use isahc::ReadResponseExt;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct ComicDownloader {
    last_seen_comic: u64,
    bookmarked_comic: Option<u64>,
    #[serde(skip)]
    rng: ThreadRng,
}

#[derive(Clone)]
pub struct Comic {
    pub name: String,
    pub number: u64,
    pub alt_text: String,
    pub date_uploaded: String,
}

impl ComicDownloader {
    pub fn new() -> Result<Self> {
        let json = match fs::read_to_string(Self::get_path_to_state_file()) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    last_seen_comic: Self::get_latest_comic_number()?,
                    bookmarked_comic: None,
                    rng: thread_rng(),
                })
            }
            Err(error) => {
                return Err(eyre!(
                    "Failed to read {}: {error}",
                    Self::get_path_to_state_file().display(),
                ))
            }
        };

        Ok(serde_json::from_str(&json)?)
    }

    pub fn switch(&mut self, switch_to_comic: SwitchToComic) -> Result<(Comic, DynamicImage)> {
        self.last_seen_comic = self.get_comic_number(switch_to_comic)?;
        self.download()
            .map_err(|e| eyre!("Failed to download comic {}: {e}", self.last_seen_comic,))
    }

    fn download(&self) -> Result<(Comic, DynamicImage)> {
        let json = Self::download_json(Some(self.last_seen_comic))?;
        let alt_text = json["alt"].as_str().unwrap().to_string();
        let name = json["title"].as_str().unwrap().to_owned();
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str().unwrap(),
            json["month"].as_str().unwrap().parse::<u16>().unwrap(),
            json["day"].as_str().unwrap().parse::<u16>().unwrap(),
        );
        let image_url = json["img"].as_str().unwrap();
        let image_bytes = &isahc::get(image_url)?.bytes()?;
        let image = image::load_from_memory(image_bytes).unwrap();
        Ok((
            Comic {
                name,
                number: self.last_seen_comic,
                alt_text,
                date_uploaded,
            },
            image,
        ))
    }

    fn get_comic_number(&mut self, switch_to_comic: SwitchToComic) -> Result<u64> {
        Ok(match switch_to_comic {
            Next => {
                if Self::get_latest_comic_number()? > self.last_seen_comic {
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
            Latest => Self::get_latest_comic_number()?,
            First => 1,
            Random => self.rng.gen_range(1..Self::get_latest_comic_number()?),
            Bookmarked => self.bookmarked_comic.unwrap_or(self.last_seen_comic),
            Specific(num) => num,
            LastSeen => self.last_seen_comic,
        })
    }

    pub fn bookmark_comic(&mut self) {
        self.bookmarked_comic = Some(self.last_seen_comic);
    }

    pub fn save_data(&self) -> Result<()> {
        let path = Self::get_path_to_state_file();
        fs::create_dir_all(path.parent().unwrap())?;
        Ok(fs::write(path, serde_json::to_string(self).unwrap())?)
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

    fn get_latest_comic_number() -> Result<u64> {
        let json = Self::download_json(None)
            .map_err(|error| eyre!("Failed to determine latest comic number: {error}"))?;

        Ok(json["num"].as_u64().unwrap())
    }
}
