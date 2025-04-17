// This is a WIP text renderer for SVT Text TV.
// Currently not in use.

use colored::Colorize;
use scraper::{Html, Selector};
use std::env;

const DEFAULT_CHANNEL: i32 = 100;

const WRAPPER_CLASS: &str = "div.TextContent_textWrapper__HaYCn";
const HEADER_CLASS: &str = "div.TextContent_header__9h_7_";
const TEXT_CONTENT_CLASS: &str = "div.TextContent_textContent__N_jyS";

fn main() {
    let args: Vec<String> = env::args().collect();

    let channel = args
        .get(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(DEFAULT_CHANNEL);

    let req = format!("https://www.svt.se/text-tv/webb/{}", channel);
    let resp = reqwest::blocking::get(req).unwrap().text().unwrap();

    let document = Html::parse_document(resp.as_str());
    let selector = Selector::parse(WRAPPER_CLASS).unwrap();

    if let Some(element) = document.select(&selector).next() {
        let selector = Selector::parse(HEADER_CLASS).unwrap();
        if let Some(header) = element.select(&selector).next() {
            println!("{}\n", header.text().collect::<Vec<_>>().join("\n").trim());
        };

        let selector = Selector::parse(TEXT_CONTENT_CLASS).unwrap();
        if let Some(text_content) = element.select(&selector).nth(1) {
            let raw_html = text_content.inner_html();

            let segments: Vec<String> = raw_html
                .split("</a>")
                .map(|segment| format!("{}</a>", segment.trim().to_string()))
                .filter(|segment| !segment.is_empty())
                .collect();

            for (idx, segment) in segments.iter().enumerate() {
                print_segment(segment.as_str(), idx as i32);
            }
        };
    } else {
        println!("Content not found. Channels range from 100-801");
    }
}

fn print_segment(segment: &str, idx: i32) {
    let selector = Selector::parse("a").unwrap();
    let document = Html::parse_document(segment);

    let mut page_number: String = String::new();

    for element in document.select(&selector) {
        page_number = element.text().collect::<Vec<_>>().join("\n");
    }

    let content = document
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join("\n");

    let content = content
        .trim()
        .lines()
        .fold(Vec::new(), |mut acc, mut line| {
            line = line.trim();
            if !(line.is_empty() && acc.last().map_or(true, |last: &&str| last.is_empty())) {
                if !line.contains(&page_number) {
                    acc.push(line);
                }
            }
            acc
        })
        .join("\n");

    if idx % 2 == 0 {
        println!("{}", content.yellow());
    } else {
        println!("{}", content.cyan());
    }

    // TODO: Add some cool separator thingy here
    // example: ----------- 130 -----------
    println!("{}", page_number);
}
