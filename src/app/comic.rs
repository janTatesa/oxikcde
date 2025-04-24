use crate::app::SwitchToComic::{self, *};
use color_eyre::{Result, eyre::Context};
use image::DynamicImage;
use isahc::AsyncReadResponseExt;
use rand::{Rng, rngs::ThreadRng};
use serde_json::Value;
use std::{fmt::Display, ops::Sub};

use super::state::State;

#[derive(Clone)]
pub struct Comic {
    name: String,
    number: u16,
    alt_text: String,
    date_uploaded: String,
    interactive: bool,
    image_url: String,
}

impl Comic {
    fn new(json: Value) -> Option<Self> {
        let alt_text = json["alt"].as_str()?.to_string();
        let name = json["title"].as_str()?.to_string();
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str()?,
            json["month"].as_str()?.parse::<u16>().ok()?,
            json["day"].as_str()?.parse::<u16>().ok()?,
        );
        let image_url = json["img"].as_str()?.to_string();
        let number = json["num"].as_u64()? as u16;
        Some(Self {
            name,
            number,
            alt_text,
            date_uploaded,
            interactive: !json["extra_parts"].is_null(),
            image_url,
        })
    }

    pub fn number(&self) -> u16 {
        self.number
    }

    pub fn alt_text(&self) -> &str {
        &self.alt_text
    }

    pub fn date_uploaded(&self) -> &str {
        &self.date_uploaded
    }

    pub fn image_url(&self) -> &str {
        &self.image_url
    }
}

impl Display for Comic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interactive_msg = if self.interactive {
            " (interactive)"
        } else {
            ""
        };
        write!(f, "{}: {}{}", self.number, self.name, interactive_msg)
    }
}

pub async fn download(comic: u16) -> Result<Comic> {
    let json = download_json(Some(comic)).await?;
    Ok(Comic::new(json).expect("XKCD should always return valid json"))
}

pub async fn download_image(image_url: String) -> Result<DynamicImage> {
    let bytes = &isahc::get_async(image_url).await?.bytes().await?;
    Ok(image::load_from_memory(bytes)?)
}

pub async fn get_comic_number(
    rng: &mut ThreadRng,
    state: &State,
    switch_to_comic: SwitchToComic,
) -> Result<u16> {
    Ok(match switch_to_comic {
        Next => {
            if get_latest_comic_number().await? > state.current_comic {
                state.current_comic + 1
            } else {
                state.current_comic
            }
        }
        Previous => state.current_comic.sub(1).max(1),
        Latest => get_latest_comic_number().await?,
        First => 1,
        Random => {
            let latest = get_latest_comic_number().await?;
            rng.gen_range(1..latest)
        }
        Bookmarked => state.bookmarked_comic().unwrap_or(state.current_comic),
        Specific(num) => num,
        LastSeen => state.current_comic,
    })
}

async fn get_latest_comic_number() -> Result<u16> {
    let json = download_json(None)
        .await
        .wrap_err("Failed to determine latest comic number")?;
    let num = json["num"]
        .as_u64()
        .expect("XKCD should always return valid json") as u16;
    Ok(num)
}

async fn download_json(number: Option<u16>) -> Result<Value> {
    let text = isahc::get_async(match number {
        Some(number) => format!("https://xkcd.com/{number}/info.0.json"),
        _ => String::from("https://xkcd.com/info.0.json"),
    })
    .await?
    .text()
    .await?;

    Ok(serde_json::from_str(&text)?)
}
