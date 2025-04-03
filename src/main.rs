use scraper::{Html, Selector};
use std::env;

const DEFAULT_CHANNEL: i32 = 100;
const CONTENT_CLASS: &str = "div.Content_screenreaderOnly__3Cnkp";

// TODO: Undersök varför inte denna dyker upp. Får fram den med en vanlig curl.
const TEXT_CONTENT_CLASS: &str = "TextContent_textWrapper__HaYCn";
fn main() {
    let args: Vec<String> = env::args().collect();

    let channel = args
        .get(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(DEFAULT_CHANNEL);

    let req = format!("https://www.svt.se/text-tv/{}", channel);
    let resp = reqwest::blocking::get(req).unwrap().text().unwrap();

    let document = Html::parse_document(resp.as_str());
    let selector = Selector::parse(CONTENT_CLASS).unwrap();

    if let Some(element) = document.select(&selector).next() {
        let content = element.text().collect::<Vec<_>>().join("\n");
        let content = content
            .trim()
            .lines()
            .fold(Vec::new(), |mut acc, mut line| {
                line = line.trim();
                if !(line.is_empty() && acc.last().map_or(true, |last: &&str| last.is_empty())) {
                    acc.push(line);
                }
                acc
            })
            .join("\n");

        println!("{}", content);
    } else {
        println!("Content not found. Channels range from 100-801");
    }
}
