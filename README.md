# Swedish text tv in the terminal

A client for reading swedish text tv in the terminal for us who want to read the news without having to open the browser.

![image of the tui](assets/screenshot.png)

## Prerequisites

- [Kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol/), [iTerm](https://iterm2.com/documentation-images.html) or [Sixel](https://github.com/saitoha/libsixel) graphics protocol is required to render full resolution images.

## Installation

More options coming soon.

### Build from source (requires [rust](https://rustup.rs/))

```sh
$ git clone https://github.com/uherman/txtv.git
$ cd txtv
$ cargo build --release
$ cp target/release/txtv ~/path/to/bin # usually ~/.local/bin
```

## Usage

```sh
$ txtv
```

### Controls

- <- - Go to previous page
- -> - Go to next page
- g - go to a specific page
- q - quit
