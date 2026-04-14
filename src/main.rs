use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent},
    execute, style, terminal,
    terminal::{disable_raw_mode, enable_raw_mode, ClearType},
};
use std::{
    env,
    error::Error,
    io::{self, Write},
};
use txtv::text_renderer::TextTvTextPage;
use txtv::{Channel, TextTvPage, MIN_PAGE};

enum Page {
    Image(TextTvPage),
    Text(TextTvTextPage),
}

impl Page {
    fn fetch_image(channel: Channel) -> Result<Self, Box<dyn Error>> {
        Ok(Page::Image(TextTvPage::fetch(channel)?))
    }

    fn fetch_text(channel: Channel) -> Result<Self, Box<dyn Error>> {
        Ok(Page::Text(TextTvTextPage::fetch(channel)?))
    }

    fn show(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Page::Image(p) => p.show(),
            Page::Text(p) => p.show(),
        }
    }

    fn next_page(&self) -> Result<Self, Box<dyn Error>> {
        match self {
            Page::Image(p) => Ok(Page::Image(p.next_page()?)),
            Page::Text(p) => Ok(Page::Text(p.next_page()?)),
        }
    }

    fn prev_page(&self) -> Result<Self, Box<dyn Error>> {
        match self {
            Page::Image(p) => Ok(Page::Image(p.prev_page()?)),
            Page::Text(p) => Ok(Page::Text(p.prev_page()?)),
        }
    }

    fn channel(&self) -> Channel {
        match self {
            Page::Image(p) => p.channel(),
            Page::Text(p) => p.channel(),
        }
    }

    fn clear_screen(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Page::Image(p) => p.clear_screen(),
            Page::Text(p) => p.clear_screen(),
        }
    }
}

/// Prints the status line below the image, aligned to left.
fn print_status(channel: Channel) -> Result<(), Box<dyn Error>> {
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
fn prompt_goto() -> Result<Option<Channel>, Box<dyn Error>> {
    let mut input = String::new();
    execute!(io::stdout(), cursor::MoveToColumn(0))?;
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
        let channel = Channel::new(n);
        return Ok(Some(channel));
    }
    Ok(None)
}

/// Prints a message that indicates that the page does not exist.
fn print_page_not_found(page: Channel) {
    let _ = execute!(
        io::stdout(),
        style::SetForegroundColor(style::Color::Red),
        style::Print(format!("Page {} not found\n", page)),
        style::ResetColor,
    );
}

fn print_version() {
    println!("txtv {}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    print_version();
    println!();
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    println!("Usage: txtv [OPTIONS] [PAGE]");
    println!();
    println!("Arguments:");
    println!("  [PAGE]  Page number to open (100–801) [default: 100]");
    println!();
    println!("Options:");
    println!("  -m, --mode <MODE>  Render mode: image (default) or text");
    println!("  -h, --help     Print help");
    println!("  -v, --version  Print version");
    println!();
    println!("Keybindings:");
    println!("  ←        Previous page");
    println!("  →        Next page");
    println!("  g        Go to a specific page");
    println!("  q        Quit");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut text_mode = false;
    let mut page_num: Option<i32> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-m" | "--mode" => {
                if let Some(val) = args.get(i + 1) {
                    if val == "text" {
                        text_mode = true;
                    }
                    i += 1;
                }
            }
            other => {
                if let Ok(n) = other.parse::<i32>() {
                    page_num = Some(n);
                }
            }
        }
        i += 1;
    }

    execute!(io::stdout(), cursor::Hide)?;

    let channel = page_num
        .map(Channel::new)
        .unwrap_or_else(|| Channel::new(MIN_PAGE));

    let fetch = if text_mode {
        Page::fetch_text
    } else {
        Page::fetch_image
    };

    let mut page = fetch(channel)?;
    page.clear_screen()?;
    page.show()?;
    print_status(page.channel())?;

    enable_raw_mode()?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Left => {
                    if let Ok(new_page) = page.prev_page() {
                        page = new_page;
                        print_status(page.channel())?;
                    }
                }

                KeyCode::Right => {
                    if let Ok(new_page) = page.next_page() {
                        page = new_page;
                        print_status(page.channel())?;
                    }
                }

                KeyCode::Char('g') => {
                    if let Some(target_channel) = prompt_goto()? {
                        page.clear_screen()?;
                        if let Ok(new_page) = fetch(target_channel) {
                            page = new_page;
                            page.show()?;
                            print_status(page.channel())?;
                        } else {
                            print_page_not_found(target_channel);
                        }
                    }
                }

                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), cursor::Show)?;
    Ok(())
}
