use color_eyre::{eyre::Ok, Result};
use image::DynamicImage;
use isahc::ReadResponseExt;
use serde_json::Value;

use crate::SwitchToComic;

#[derive(Default)]
pub struct ComicDownloader {
    number: u32,
}

#[derive(Default)]
pub struct Comic {
    pub title: String,
    pub alt_text: String,
    pub date_uploaded: String,
    pub image: DynamicImage,
}

impl Comic {
    fn new(json: Value) -> Result<Comic> {
        let title = format!(
            "{}: {}",
            json["num"].as_u64().unwrap(),
            json["title"].as_str().unwrap()
        );
        let alt_text = json["alt"].as_str().unwrap().to_string();
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str().unwrap(),
            json["month"].as_str().unwrap().parse::<u16>().unwrap(),
            json["day"].as_str().unwrap().parse::<u16>().unwrap(),
        );

        let image = image::load_from_memory(&isahc::get(json["img"].as_str().unwrap())?.bytes()?)?;
        Ok(Comic {
            title,
            alt_text,
            date_uploaded,
            image,
        })
    }
}

impl ComicDownloader {
    pub fn switch(&mut self, switch_to_comic: SwitchToComic) -> Result<Comic> {
        let number = match switch_to_comic {
            SwitchToComic::Next => Some(self.number + 1),
            SwitchToComic::Previous => Some(self.number - 1),
            SwitchToComic::Latest => None,
            SwitchToComic::First => Some(1),
            SwitchToComic::Random => todo!(),
            SwitchToComic::Bookmarked => todo!(),
            SwitchToComic::Specific(num) => Some(num),
        };
        let download_result = Self::download(number)?;
        self.number = download_result.0;
        Ok(download_result.1)
    }

    fn download(number: Option<u32>) -> Result<(u32, Comic)> {
        let text = isahc::get(match number {
            Some(number) => format!("https://xkcd.com/{}/info.0.json", number),
            _ => String::from("https://xkcd.com/info.0.json"),
        })?
        .text()?;
        let json: Value = serde_json::from_str(text.as_str())?;
        Ok((json["num"].as_u64().unwrap() as u32, Comic::new(json)?))
    }
}
