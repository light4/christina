# Christina

An assistant to help play with [STEINS;GATE](https://store.steampowered.com/app/412830/STEINSGATE/).

Take a screenshot then get the Japanese text and translate.

![translated](./assets/2023-01-08.jpg)

## Build

Powered by <https://www.rust-lang.org> and <https://tauri.app>.

Depends on [tesseract](https://github.com/tesseract-ocr/tesseract).

Refactored to use [wry](https://github.com/tauri-apps/wry) directly.

```bash
# for debug
cargo build
# for release
pnpm build && cargo build --release
```

## Usage

Set a global shortcut, start game and run.

This tool will keep a system tray when close. Use system tray to quit.

Program will listen to keyboard events: when `space/enter/m` released, will trigger the
work flow.

`Capture screen -> process image -> OCR -> translate -> reload content.`

If it's already running when start, contents will reload.
So just input shortcut when you want to use it.

![Okabe-Rintaro-and-Makise-Kurisu-Steins-Gate.png](./assets/Okabe-Rintaro-and-Makise-Kurisu-Steins-Gate.png)
