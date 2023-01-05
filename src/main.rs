use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// 1. capture screen
/// 2. process image
/// 3. ocr
/// 4. process text
/// 5. copy to clipboard
/// 6. translate
use anyhow::Result;
use arboard::Clipboard;
use directories::ProjectDirs;
use image::{
    imageops,
    imageops::colorops::{index_colors, ColorMap},
    io::Reader as ImageReader,
    ImageBuffer,
};
use notify_rust::{Notification, Urgency};
use screenshots::Screen;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

mod color;
mod translate;

use color::MyLevel;

fn main() -> Result<()> {
    let proj_dirs =
        ProjectDirs::from("com", "i01", "christina").expect("cannot construct project directories");
    let cache_dir = proj_dirs.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }

    let first_arg = std::env::args().nth(1);
    let filename = {
        if let Some(s) = first_arg {
            PathBuf::from(s)
        } else {
            capture_img(cache_dir)?
        }
    };

    let img = ImageReader::open(filename)?.decode()?.to_rgba8();
    let sub_img = imageops::crop_imm(&img, 0, 770, 1920, 310).to_image();
    let cmap = MyLevel;

    let palletized = index_colors(&sub_img, &cmap);
    let mapped = ImageBuffer::from_fn(sub_img.width(), sub_img.height(), |x, y| {
        let p = palletized.get_pixel(x, y);
        cmap.lookup(p.0[0] as usize)
            .expect("indexed color out-of-range")
    });

    let new_filename = cache_dir.join("processed.jpg");
    mapped.save(&new_filename)?;

    // tesseract empty.jpg test -l jpn
    let mut text = tesseract::ocr(new_filename.to_str().unwrap(), "jpn")?;
    text.retain(|c| c != ' ');
    dbg!(&text);

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.clone())?;

    if let Some(chi_sim) = translate::translate(&text) {
        // dbg!(&chi_sim);
        // text.push('\n');
        // text.push_str(&chi_sim);
        Notification::new()
            .summary("Christina")
            .body(&chi_sim)
            .icon("firefox")
            .urgency(Urgency::Critical)
            .show()?;
    }

    Ok(())
}

fn capture_img(path: &Path) -> Result<PathBuf> {
    let now = OffsetDateTime::now_local()?;
    let screen_capturers = Screen::all().expect("get all screen error");
    let screen_capturer = screen_capturers.first().unwrap();
    println!("capturer {screen_capturer:?}");
    let image = screen_capturer.capture().unwrap();
    let buffer = image.buffer();
    let filename = now.format(&Rfc3339)? + ".png";
    let abs_path = path.join(filename);
    let mut file = File::create(&abs_path)?;
    file.write_all(&buffer[..])?;

    Ok(abs_path)
}
