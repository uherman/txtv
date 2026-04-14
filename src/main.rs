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
use txtv::{Channel, TextTvPage, MIN_PAGE};

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

fn main() -> Result<(), Box<dyn Error>> {
    execute!(io::stdout(), cursor::Hide)?;

    // Load initial channel from args, or default to min.
    let channel = env::args()
        .nth(1)
        .and_then(|s| s.parse::<i32>().ok())
        .map(Channel::new)
        .unwrap_or_else(|| Channel::new(MIN_PAGE));

    let mut page = TextTvPage::fetch(channel)?;
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
                    } else {
                        print_page_not_found(page.channel());
                    }
                }

                KeyCode::Right => {
                    if let Ok(new_page) = page.next_page() {
                        page = new_page;
                        print_status(page.channel())?;
                    } else {
                        print_page_not_found(page.channel());
                    }
                }

                KeyCode::Char('g') => {
                    if let Some(target_channel) = prompt_goto()? {
                        page.clear_screen()?;
                        if let Ok(new_page) = TextTvPage::fetch(target_channel) {
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
