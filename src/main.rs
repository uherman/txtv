use base64::{engine::general_purpose, Engine as _};
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent},
    execute, style,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    terminal,
};
use scraper::{Html, Selector};
use std::{
    env,
    error::Error,
    io::{self, Write},
};
use viuer::Config;

/// Represents a page direction.
enum PageDirection {
    Next,
    Prev
}

const MIN_PAGE : i32 = 100;
const MAX_PAGE : i32 = 801;

/// Fetches & displays the given page image inline.
/// TODO: Refator this, it looks terrible.
fn fetch_and_show(channel: i32) -> Result<(i32, i32), Box<dyn Error>> {
    let url = format!("https://www.svt.se/text-tv/{}", channel);
    let html = reqwest::blocking::get(&url)?.text()?;
    let document = Html::parse_document(&html);
    let selector = Selector::parse("img.Content_pageImage__bS0mg")?;
    let elem = document.select(&selector).next().ok_or("Page not found")?;
    let data_url = elem.value().attr("src").unwrap();
    let b64 = data_url.splitn(2, ',').nth(1).ok_or("invalid data URL")?;
    let img_data = general_purpose::STANDARD.decode(b64)?;
    let img = image::load_from_memory(&img_data)?;
    let config = Config {
        width: Some(100),
        ..Default::default()
    };
    viuer::print(&img, &config)?;
    let prev = get_page_number(&document, PageDirection::Prev, channel);
    let next = get_page_number(&document, PageDirection::Next, channel);
    Ok((prev, next))
}

/// Get the page number for the specified page direction.
fn get_page_number(document: &Html, direction: PageDirection, channel: i32) -> i32 {
   let fallback = match direction {
        PageDirection::Next => channel + 1,
        PageDirection::Prev => channel - 1,
   };

    let selector = match direction {
        PageDirection::Next => Selector::parse("[title='Nästa sida']"),
        PageDirection::Prev => Selector::parse("[title='Förra sidan']"),
    };

    let page_number = selector
        .ok()
        .and_then(|sel| document.select(&sel).next())
        .and_then(|el| el.value().attr("href"))
        .and_then(|href| href.split('/').last())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(fallback)
        .clamp(MIN_PAGE, MAX_PAGE);

    page_number
}

/// Prints the status line below the image, aligned to left.
fn print_status(channel: i32) -> Result<(), Box<dyn Error>> {
    execute!(
        io::stdout(),
        cursor::MoveToColumn(0),
        cursor::MoveToNextLine(1),
    )?;
    println!(
        "← prev   → next   g: go to page   q: quit   (now on {})",
        channel
    );
    Ok(())
}

/// Prompt in raw mode for a new page number.
fn prompt_goto(current: i32) -> Result<Option<i32>, Box<dyn Error>> {
    let mut input = String::new();
    execute!(
        io::stdout(),
        cursor::MoveToColumn(0)
    )?;
    print!("Go to page (100–801): ");
    io::stdout().flush()?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    input.push(c);
                    print!("{}", c);
                    io::stdout().flush()?;
                }
                KeyCode::Backspace => {
                    if input.pop().is_some() {
                        execute!(
                            io::stdout(),
                            cursor::MoveLeft(1),
                            style::Print(" "),
                            cursor::MoveLeft(1),
                        )?;
                        io::stdout().flush()?;
                    }
                }
                KeyCode::Enter => break,
                KeyCode::Esc => {
                    execute!(
                        io::stdout(),
                        cursor::MoveToColumn(0),
                        terminal::Clear(ClearType::CurrentLine),
                    )?;
                    io::stdout().flush()?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }

    if let Ok(n) = input.parse::<i32>() {
        let page = n.clamp(MIN_PAGE, MAX_PAGE);
        if page != current {
            return Ok(Some(page));
        }
    }
    Ok(None)
}

/// Prints a message that indicates that the page does not exist.
fn print_page_not_found(page: i32) -> () {
    let _ = execute!(
        io::stdout(),
        style::SetForegroundColor(style::Color::Red),
        style::Print(format!("Page {} not found\n", page)),
        style::ResetColor,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    // Hide cursor
    execute!(io::stdout(), cursor::Hide)?;

    // Start page
    let mut channel = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(MIN_PAGE)
        .clamp(MIN_PAGE, MAX_PAGE);

    // Initialize prev & next page with default values
    let mut prev: i32 = (channel - 1).clamp(MIN_PAGE, MAX_PAGE);
    let mut next: i32 = channel + 1.clamp(MIN_PAGE, MAX_PAGE);

    // Initial clear
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0),)?;
    
    // Initial render
    match fetch_and_show(channel) {
        Ok((p, n)) => {
            prev = p;
            next = n;
        },
        Err(_) => {
            print_page_not_found(channel);
        }
    }

    print_status(channel)?;
    enable_raw_mode()?;
    loop {
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Left => {
                    if prev != channel {
                        channel = prev;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        match fetch_and_show(channel) {
                            Ok((p, n)) => {
                                prev = p;
                                next = n;
                            },
                            Err(_) => {
                                print_page_not_found(channel);
                            }
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Right => {
                    if next != channel {
                        channel = next;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        match fetch_and_show(channel) {
                            Ok((p, n)) => {
                                prev = p;
                                next = n;
                            },
                            Err(_) => {
                                print_page_not_found(channel);
                            }
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Char('g') => {
                    if let Some(new_ch) = prompt_goto(channel)? {
                        channel = new_ch;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        match fetch_and_show(channel) {
                            Ok((p, n)) => {
                                prev = p;
                                next = n;
                            },
                            Err(_) => {
                                print_page_not_found(channel);
                            }
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    // Restore state
    disable_raw_mode()?;
    execute!(io::stdout(), cursor::Show)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const PAGE_NUMBERS_HTML: &str = "
        <a class=\"NavigationArrow_enabled__ueMbi NavigationArrow_navigationArrow__eaKzk\" title=\"Nästa sida\" href=\"/text-tv/103\"></a>
        <a class=\"NavigationArrow_enabled__ueMbi NavigationArrow_navigationArrow__eaKzk\" title=\"Förra sidan\" href=\"/text-tv/101\"></a>
        ";

    #[test]
    fn get_page_number_returns_next_page() {
        let document = Html::parse_document(&PAGE_NUMBERS_HTML);
        let channel = 200; // Use 200 to ensure fallback is not used
        let next_page = get_page_number(&document, PageDirection::Next, channel);

        assert_eq!(next_page, 103);
    }

    #[test]
    fn get_page_number_returns_prev_page() {
        let document = Html::parse_document(&PAGE_NUMBERS_HTML);
        let channel = 200; // Use 200 to esnure fallback is not used
        let prev_page = get_page_number(&document, PageDirection::Prev, channel);

        assert_eq!(prev_page, 101);
    }

    #[test]
    fn get_next_page_number_when_no_page_in_html_returns_fallback_plus_one() {
        let document = Html::parse_document("");
        let channel = 103;
        let next_page = get_page_number(&document, PageDirection::Next, channel);

        assert_eq!(next_page, 104);
    }

    #[test]
    fn get_prev_page_number_when_no_page_in_html_returns_fallback_minus_one() {
        let document = Html::parse_document("");
        let channel = 103;
        let prev_page = get_page_number(&document, PageDirection::Prev, channel);

        assert_eq!(prev_page, 102);
    }

    #[test]
    fn get_next_page_should_not_go_above_max_page() {
        let document = Html::parse_document("");
        let channel = MAX_PAGE;
        let next_page = get_page_number(&document, PageDirection::Next, channel);

        assert_eq!(next_page, MAX_PAGE);
    }

    #[test]
    fn get_prev_page_should_not_go_below_min_page() {
        let document = Html::parse_document("");
        let channel = MIN_PAGE;
        let prev_page = get_page_number(&document, PageDirection::Prev, channel);

        assert_eq!(prev_page, MIN_PAGE);
    }
}
