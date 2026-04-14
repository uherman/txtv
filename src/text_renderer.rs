use colored::Colorize;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;
use std::io;

use crate::Channel;

const WRAPPER_CLASS: &str = "div.TextContent_textWrapper__HaYCn";
const HEADER_CLASS: &str = "div.TextContent_header__9h_7_";
const TEXT_CONTENT_CLASS: &str = "div.TextContent_textContent__N_jyS";

#[derive(Debug)]
pub struct TextTvTextPage {
    channel: Channel,
    document: Html,
    prev: Channel,
    next: Channel,
}

impl TextTvTextPage {
    pub fn fetch(channel: Channel) -> Result<Self, Box<dyn Error>> {
        let url = format!("https://www.svt.se/text-tv/webb/{}", channel);
        let html = reqwest::blocking::get(&url)?.text()?;
        let document = Html::parse_document(&html);

        let prev = channel.prev_from_document(&document);
        let next = channel.next_from_document(&document);

        Ok(Self {
            channel,
            document,
            prev,
            next,
        })
    }

    pub fn show(&self) -> Result<(), Box<dyn Error>> {
        let selector = Selector::parse(WRAPPER_CLASS)?;

        if let Some(element) = self.document.select(&selector).next() {
            let header_sel = Selector::parse(HEADER_CLASS)?;
            let mut header_lines: Vec<String> = Vec::new();
            if let Some(header) = element.select(&header_sel).next() {
                let text = html_to_text(&header.inner_html());
                for line in text.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        print!("{}\r\n", trimmed.bold());
                        header_lines.push(trimmed.to_string());
                    }
                }
            }

            let content_sel = Selector::parse(TEXT_CONTENT_CLASS)?;
            let mut prev_blank = true;
            for text_content in element.select(&content_sel) {
                let text = html_to_text(&text_content.inner_html());
                for line in text.lines() {
                    let trimmed = line.trim();
                    if header_lines.iter().any(|h| h == trimmed) {
                        continue;
                    }
                    if trimmed.is_empty() {
                        if !prev_blank {
                            print!("\r\n");
                            prev_blank = true;
                        }
                    } else {
                        print!("{}\r\n", trimmed.cyan());
                        prev_blank = false;
                    }
                }
            }
        } else {
            print!("Content not found for page {}\r\n", self.channel);
        }

        Ok(())
    }

    pub fn next_page(&self) -> Result<Self, Box<dyn Error>> {
        if self.next == self.channel {
            return Err("Already at the last page".into());
        }
        self.clear_screen()?;
        let page = Self::fetch(self.next)?;
        page.show()?;
        Ok(page)
    }

    pub fn prev_page(&self) -> Result<Self, Box<dyn Error>> {
        if self.prev == self.channel {
            return Err("Already at the first page".into());
        }
        self.clear_screen()?;
        let page = Self::fetch(self.prev)?;
        page.show()?;
        Ok(page)
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn clear_screen(&self) -> Result<(), Box<dyn Error>> {
        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }
}

fn html_to_text(html: &str) -> String {
    let br_re = Regex::new(r"<br\s*/?>").unwrap();
    let with_newlines = br_re.replace_all(html, "\n");
    let fragment = Html::parse_fragment(&with_newlines);
    fragment.root_element().text().collect::<Vec<_>>().join("")
}
