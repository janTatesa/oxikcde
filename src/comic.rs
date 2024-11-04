use color_eyre::{owo_colors::OwoColorize, Result};
use isahc::ReadResponseExt;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct Comic {
    pub name: String,
    pub number: u32,
    image: Vec<u8>,
    pub alt_text: String,
}

const URL: &str = "https://xkcd.com/";
const URL_LENGHT: usize = 17;
impl Comic {
    pub fn download(number: Option<u32>) -> Result<Self> {
        let num_string = match number {
            Some(num) => &num.to_string(),
            _ => "",
        };
        let html = Html::parse_document(isahc::get(URL.to_string() + num_string)?.text()?.as_str());
        let img_element = html
            .select(&Selector::parse("#comic img").unwrap())
            .next()
            .unwrap();
        let alt_text = img_element.attr("title").unwrap().to_owned();
        let name = img_element.attr("alt").unwrap().to_owned();
        let image = isahc::get("https:".to_string() + img_element.attr("src").unwrap())?.bytes()?;
        let number = number.unwrap_or({
            let elements = html
                .select(&Selector::parse("#middleContainer a").unwrap())
                .collect::<Vec<ElementRef>>();
            let element = elements.get(10).unwrap();
            let url = element.attr("href").unwrap();
            url[URL_LENGHT..].parse()?
        });
        Ok(Self {
            name,
            number,
            image,
            alt_text,
        })
    }

    pub fn image(&self) -> Result<Box<dyn StatefulProtocol>> {
        let mut picker = Picker::from_termios().unwrap();
        picker.guess_protocol();
        Ok(picker.new_resize_protocol(image::load_from_memory(&self.image)?))
    }

    pub fn title(&self) -> String {
        format!("{}: {}", self.number, self.name)
    }
}
