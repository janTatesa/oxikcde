use crate::app::SwitchToComic::{self, *};
use color_eyre::{Result, eyre::Context};
use image::DynamicImage;
use isahc::ReadResponseExt;
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
}

impl Comic {
    pub fn new(number: u16, json: Value) -> Option<Self> {
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

    pub fn number(&self) -> u16 {
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
        let interactive_msg = if self.interactive {
            " (interactive)"
        } else {
            ""
        };
        write!(f, "{}: {}{}", self.number, self.name, interactive_msg)
    }
}

pub fn download(state: &State) -> Result<(Comic, DynamicImage)> {
    let json = download_json(Some(state.last_seen_comic))?;
    let image_url = json["img"]
        .as_str()
        .expect("XKCD should always return valid json");
    let image_bytes = &isahc::get(image_url)?.bytes()?;
    let image = image::load_from_memory(image_bytes)?;
    Ok((
        Comic::new(state.last_seen_comic, json).expect("XKCD should always return valid json"),
        image,
    ))
}

pub fn get_comic_number(
    rng: &mut ThreadRng,
    state: &mut State,
    switch_to_comic: SwitchToComic,
) -> Result<u16> {
    Ok(match switch_to_comic {
        Next => {
            if state.minimum_latest_comic_number > state.last_seen_comic
                || get_latest_comic_number(state)? > state.last_seen_comic
            {
                state.last_seen_comic + 1
            } else {
                state.last_seen_comic
            }
        }
        Previous => state.last_seen_comic.sub(1).max(1),
        Latest => get_latest_comic_number(state)?,
        First => 1,
        Random => {
            let latest = get_latest_comic_number(state)?;
            rng.gen_range(1..latest)
        }
        Bookmarked => state.bookmarked_comic().unwrap_or(state.last_seen_comic),
        Specific(num) => num,
        LastSeen => state.last_seen_comic,
    })
}

fn get_latest_comic_number(state: &mut State) -> Result<u16> {
    let json = download_json(None).wrap_err("Failed to determine latest comic number")?;
    let num = json["num"]
        .as_u64()
        .expect("XKCD should always return valid json") as u16;
    state.minimum_latest_comic_number = num;
    Ok(num)
}

fn download_json(number: Option<u16>) -> Result<Value> {
    let text = isahc::get(match number {
        Some(number) => format!("https://xkcd.com/{number}/info.0.json"),
        _ => String::from("https://xkcd.com/info.0.json"),
    })?
    .text()?;

    Ok(serde_json::from_str(&text)?)
}
