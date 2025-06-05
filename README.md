# Swedish Text TV in the Terminal

A simple and fast terminal client for browsing Swedish Text TV — for us who prefer reading the news without launching a web browser.

This project is inspired by the now-unmaintained [txtv by voidcase](https://github.com/voidcase/txtv), which no longer works. 

![image of the tui](assets/screenshot.png)

## Features
> 🗓️ Planned | ✅ Implemented
- Display text tv pages as images ✅
- Plaintext mode 🗓️

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

- **`←`** Go to previous page
- **`→`** Go to next page
- `g` Go to a specific page
- `q` Quit

