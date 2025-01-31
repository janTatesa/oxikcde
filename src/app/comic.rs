use crate::app::SwitchToComic::{self, *};
use derive_getters::Getters;
use dirs::state_dir;
use eyre::{eyre, Result};
use image::DynamicImage;
use isahc::ReadResponseExt;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Clone, Getters)]
pub struct Comic {
    name: String,
    number: u64,
    alt_text: String,
    date_uploaded: String,
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
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComicDownloader {
    last_seen_comic: u64,
    bookmarked_comic: Option<u64>,
    #[serde(skip)]
    rng: ThreadRng,
}

impl ComicDownloader {
    pub fn new() -> Result<Self> {
        match fs::read_to_string(get_path_to_state_file()) {
            Ok(json) => Ok(serde_json::from_str(&json)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                last_seen_comic: get_latest_comic_number()?,
                bookmarked_comic: None,
                rng: thread_rng(),
            }),
            Err(error) => Err(eyre!(
                "Failed to read {}: {error}",
                get_path_to_state_file().display(),
            )),
        }
    }

    pub fn switch(&mut self, switch_to_comic: SwitchToComic) -> Result<(Comic, DynamicImage)> {
        self.last_seen_comic = self.get_comic_number(switch_to_comic)?;
        self.download()
            .map_err(|e| eyre!("Failed to download comic {}: {e}", self.last_seen_comic,))
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
                if get_latest_comic_number()? > self.last_seen_comic {
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
            Latest => get_latest_comic_number()?,
            First => 1,
            Random => self.rng.gen_range(1..get_latest_comic_number()?),
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
    let json = download_json(None)
        .map_err(|error| eyre!("Failed to determine latest comic number: {error}"))?;

    Ok(json["num"].as_u64().unwrap())
}
