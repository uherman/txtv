use base64::{engine::general_purpose, Engine as _};
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use image::DynamicImage;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use std::{
    fmt,
    io::{self},
};
use viuer::{print as print_image, Config};

/// The lowest valid page number in the SVT Text TV range.
pub const MIN_PAGE: i32 = 100;
pub const MAX_PAGE: i32 = 801;

/// Represents a page direction.
pub enum PageDirection {
    Next,
    Prev,
}

/// Represents a valid SVT Text TV channel (i.e. page number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Channel(i32);

impl Channel {
    /// Attempts to create a new `Channel`, clamping the value to the allowed range.
    pub fn new(page: i32) -> Self {
        Self(page.clamp(MIN_PAGE, MAX_PAGE))
    }

    /// Returns the underlying page number.
    pub fn number(self) -> i32 {
        self.0
    }

    /// Returns the next channel, using the HTML document if possible.
    pub fn next_from_document(&self, document: &Html) -> Self {
        self.navigate(document, PageDirection::Next)
    }

    /// Returns the previous channel, using the HTML document if possible.
    pub fn prev_from_document(&self, document: &Html) -> Self {
        self.navigate(document, PageDirection::Prev)
    }

    fn navigate(&self, document: &Html, direction: PageDirection) -> Self {
        let fallback = match direction {
            PageDirection::Next => self.0 + 1,
            PageDirection::Prev => self.0 - 1,
        };

        let selector = match direction {
            PageDirection::Next => "[title='Nästa sida']",
            PageDirection::Prev => "[title='Förra sidan']",
        };

        let parsed = Selector::parse(selector)
            .ok()
            .and_then(|sel| document.select(&sel).next())
            .and_then(|el| el.value().attr("href"))
            .and_then(|href| href.split('/').last())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(fallback);

        Channel::new(parsed)
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a single SVT Text TV page, including its metadata,
/// raw HTML content, and the rendered image.
///
/// This struct encapsulates all necessary data for rendering and
/// navigating a specific Text TV page retrieved from SVT's online service.
#[derive(Debug)]
pub struct TextTvPage {
    /// The page number (channel) of the Text TV page, e.g. `100`, `377`.
    ///
    /// This corresponds to the numeric code used to access the page on SVT's Text TV service.
    channel: Channel,

    /// The parsed HTML document of the Text TV page.
    ///
    /// This is used to extract metadata and navigation links (e.g. previous/next pages).
    document: Html,

    /// The decoded image that visually represents the content of the page.
    ///
    /// This image is typically base64-encoded in the source HTML and rendered inline.
    image: DynamicImage,

    /// The parsed page number of the previous page, extracted from navigation metadata.
    ///
    /// If metadata is unavailable or invalid, this falls back to `channel - 1`,
    /// clamped within the valid channel range.
    prev: Channel,

    /// The parsed page number of the next page, extracted from navigation metadata.
    ///
    /// If metadata is unavailable or invalid, this falls back to `channel + 1`,
    /// clamped within the valid channel range.
    next: Channel,
}

impl TextTvPage {
    pub fn fetch(channel: Channel) -> Result<Self, Box<dyn Error>> {
        let url = format!("https://www.svt.se/text-tv/{}", channel);
        let html = reqwest::blocking::get(&url)?.text()?;
        let document = Html::parse_document(&html);

        let selector = Selector::parse("img.Content_pageImage__bS0mg")?;
        let element = document
            .select(&selector)
            .next()
            .ok_or("Image element not found")?;
        let data_url = element
            .value()
            .attr("src")
            .ok_or("Missing src attribute on image")?;
        let image_data = Self::decode_image_data(data_url)?;
        let image = image::load_from_memory(&image_data)?;

        let prev = channel.prev_from_document(&document);
        let next = channel.next_from_document(&document);

        Ok(Self {
            channel,
            document,
            image,
            prev,
            next,
        })
    }

    pub fn show(&self) -> Result<(), Box<dyn Error>> {
        let config = Config {
            width: Some(100),
            ..Default::default()
        };
        print_image(&self.image, &config)?;
        Ok(())
    }

    pub fn page_number(&self, direction: PageDirection) -> Channel {
        match direction {
            PageDirection::Next => self.channel.next_from_document(&self.document),
            PageDirection::Prev => self.channel.prev_from_document(&self.document),
        }
    }

    pub fn next_page(&self) -> Result<Self, Box<dyn Error>> {
        self.clear_screen()?;
        let page = Self::fetch(self.next)?;
        page.show()?;
        return Ok(page);
    }

    pub fn prev_page(&self) -> Result<Self, Box<dyn Error>> {
        self.clear_screen()?;
        let page = Self::fetch(self.prev)?;
        page.show()?;
        return Ok(page);
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn clear_screen(&self) -> Result<(), Box<dyn Error>> {
        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    fn decode_image_data(data_url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let b64 = data_url
            .splitn(2, ',')
            .nth(1)
            .ok_or("Invalid data URL format")?;
        Ok(general_purpose::STANDARD.decode(b64)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE_NUMBERS_HTML: &str = "
        <a class=\"NavigationArrow_enabled__ueMbi NavigationArrow_navigationArrow__eaKzk\" title=\"Nästa sida\" href=\"/text-tv/103\"></a>
        <a class=\"NavigationArrow_enabled__ueMbi NavigationArrow_navigationArrow__eaKzk\" title=\"Förra sidan\" href=\"/text-tv/101\"></a>
    ";

    #[test]
    fn next_from_document_returns_correct_page() {
        let document = Html::parse_document(&PAGE_NUMBERS_HTML);
        let current = Channel::new(200);
        let next = current.next_from_document(&document);

        assert_eq!(next, Channel::new(103));
    }

    #[test]
    fn prev_from_document_returns_correct_page() {
        let document = Html::parse_document(&PAGE_NUMBERS_HTML);
        let current = Channel::new(200);
        let prev = current.prev_from_document(&document);

        assert_eq!(prev, Channel::new(101));
    }

    #[test]
    fn next_from_document_falls_back_plus_one_if_missing() {
        let document = Html::parse_document("");
        let current = Channel::new(103);
        let next = current.next_from_document(&document);

        assert_eq!(next, Channel::new(104));
    }

    #[test]
    fn prev_from_document_falls_back_minus_one_if_missing() {
        let document = Html::parse_document("");
        let current = Channel::new(103);
        let prev = current.prev_from_document(&document);

        assert_eq!(prev, Channel::new(102));
    }

    #[test]
    fn next_does_not_exceed_max_page() {
        let document = Html::parse_document("");
        let current = Channel::new(MAX_PAGE);
        let next = current.next_from_document(&document);

        assert_eq!(next, Channel::new(MAX_PAGE));
    }

    #[test]
    fn prev_does_not_go_below_min_page() {
        let document = Html::parse_document("");
        let current = Channel::new(MIN_PAGE);
        let prev = current.prev_from_document(&document);

        assert_eq!(prev, Channel::new(MIN_PAGE));
    }
}

