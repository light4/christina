/// 1. capture screen
/// 2. process image
/// 3. ocr
/// 4. process text
/// 5. copy to clipboard
/// 6. translate
use anyhow::Result;
use arboard::Clipboard;
use image::{
    imageops::colorops::{index_colors, ColorMap},
    io::Reader as ImageReader,
    ImageBuffer,
};
use notify_rust::Notification;
use screenshots::ScreenCapturer;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use std::{fs::File, io::Write};

mod color;

use color::MyLevel;

fn main() -> Result<()> {
    let first_arg = std::env::args().nth(1);
    let filename = {
        if let Some(s) = first_arg {
            s
        } else {
            capture_img()?
        }
    };

    let img = ImageReader::open(&filename)?.decode()?.to_rgba8();
    let cmap = MyLevel;

    let palletized = index_colors(&img, &cmap);
    let mapped = ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = palletized.get_pixel(x, y);
        cmap.lookup(p.0[0] as usize)
            .expect("indexed color out-of-range")
    });

    let new_filename = "processed.jpg";
    mapped.save(new_filename)?;

    // tesseract empty.jpg test -l jpn
    let mut text = tesseract::ocr(new_filename, "jpn")?;
    text.retain(|c| c != ' ');
    dbg!(&text);

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.clone())?;

    Notification::new()
        .summary("Christina")
        .body(&text)
        .icon("firefox")
        .show()?;

    Ok(())
}

fn capture_img() -> Result<String> {
    let now = OffsetDateTime::now_local()?;
    let screen_capturers = ScreenCapturer::all();
    let screen_capturer = screen_capturers.first().unwrap();
    println!("capturer {:?}", screen_capturer);
    let image = screen_capturer.capture().unwrap();
    let buffer = image.png()?;
    let path = now.format(&Rfc3339)? + ".png";
    let mut file = File::create(&path)?;
    file.write_all(&buffer[..])?;

    Ok(path)
}
