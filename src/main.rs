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

/// Fetches & displays the given page image inline.
fn fetch_and_show(channel: i32) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

/// Get the page number for the specified page direction.
fn get_page_number(document: Html, page: PageDirection) -> Result<i32, Box<dyn Error>> {
    let selector = match page {
        PageDirection::Next => Selector::parse("[title='Nästa sida']")?,
        PageDirection::Prev => Selector::parse("[title='Förra sidan']")?,
    };

    let element = document.select(&selector).next().ok_or("Failed to fetch page number")?;
    let href = element.value().attr("href").unwrap();

    let parts : Vec<&str> = href.split('/').collect();
    let page : i32 = parts.last().unwrap().parse().unwrap();

    println!("{}", page);

    Ok(page)
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
        let page = n.clamp(100, 801);
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
    // hide cursor
    execute!(io::stdout(), cursor::Hide)?;

    // Start page
    let mut channel = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
        .clamp(100, 801);

    // Initial clear
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0),)?;

    // Initial render
    if let Err(_) = fetch_and_show(channel) {
        print_page_not_found(channel);
    }
    print_status(channel)?;

    enable_raw_mode()?;
    loop {
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Left => {
                    // TODO: Implement get_page_number
                    let new_ch = (channel - 1).max(100);
                    if new_ch != channel {
                        channel = new_ch;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        if let Err(_) = fetch_and_show(channel) {
                            print_page_not_found(channel);
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Right => {
                    // TODO: Implement get_page_number
                    let new_ch = (channel + 1).min(801);
                    if new_ch != channel {
                        channel = new_ch;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        if let Err(_) = fetch_and_show(channel) {
                           print_page_not_found(channel); 
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Char('g') => {
                    if let Some(new_ch) = prompt_goto(channel)? {
                        channel = new_ch;
                        execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        if let Err(_) = fetch_and_show(channel) {
                            print_page_not_found(channel);
                        }
                        print_status(channel)?;
                    }
                }
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    // restore state
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
        let next_page = get_page_number(document, PageDirection::Next).unwrap();

        assert_eq!(next_page, 103);
    }

    #[test]
    fn get_page_number_returns_prev_page() {
        let document = Html::parse_document(&PAGE_NUMBERS_HTML);
        let prev_page = get_page_number(document, PageDirection::Prev).unwrap();

        assert_eq!(prev_page, 101);
    }
}
