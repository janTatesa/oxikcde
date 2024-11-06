use color_eyre::{eyre::Ok, Result};
use isahc::ReadResponseExt;
use ratatui::{style::Stylize, text::Line};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use serde_json::Value;

pub struct Comic {
    name: String,
    pub number: u32,
    image: Vec<u8>,
    alt_text: String,
    date_uploaded: String,
}

impl Comic {
    //TODO: implement caching
    pub fn download(number: Option<u32>) -> Result<Self> {
        let text = isahc::get(match number {
            Some(number) => format!("https://xkcd.com/{}/info.0.json", number),
            _ => String::from("https://xkcd.com/info.0.json"),
        })?
        .text()?;
        let json: Value = serde_json::from_str(text.as_str())?;
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str().unwrap(),
            json["month"].as_str().unwrap().parse::<u16>().unwrap(),
            json["day"].as_str().unwrap().parse::<u16>().unwrap(),
        );
        let image = isahc::get(json["img"].as_str().unwrap())?.bytes()?;
        Ok(Self {
            name: json["title"].as_str().unwrap().to_string(),
            number: json["num"].as_u64().unwrap() as u32,
            image,
            alt_text: json["alt"].as_str().unwrap().to_string(),
            date_uploaded,
        })
    }

    pub fn image(&self, picker: &mut Picker) -> Result<StatefulProtocol> {
        Ok(picker.new_resize_protocol(image::load_from_memory(&self.image)?))
    }

    pub fn alt_text(&self) -> Line {
        Line::from(self.alt_text.as_str()).centered().gray()
    }

    pub fn title(&self) -> Line {
        Line::from(format!("{}: {}", self.number, self.name))
            .centered()
            .yellow()
    }
    pub fn date_uploaded(&self) -> Line {
        Line::from(self.date_uploaded.as_str()).blue()
    }
}
