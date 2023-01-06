#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod color;
mod sites;
mod translate;

use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::Context;
/// 1. capture screen
/// 2. process image
/// 3. ocr
/// 4. process text
/// 5. copy to clipboard
/// 6. translate
/// 7. show a translate panel
use anyhow::Result;
use arboard::Clipboard;
use color::MyLevel;
use directories::ProjectDirs;
use image::{
    imageops,
    imageops::colorops::{index_colors, ColorMap},
    io::Reader as ImageReader,
    ImageBuffer,
};
use once_cell::sync::Lazy;
use screenshots::Screen;
use sites::web_get_translate_sites;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

static GLOBAL_ORIGIN: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("それにも、当然ながら関心があった。".to_string()));
static GLOBAL_TRANSLATED: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("对此，我当然很感兴趣。".to_string()));

#[tauri::command]
fn web_translate(origin: &str) -> String {
    translate::translate(origin).unwrap_or_else(|| "翻译出错了".to_string())
}

#[tauri::command]
fn web_get_origin() -> String {
    let lock = GLOBAL_ORIGIN.lock().unwrap();
    lock.to_string()
}

#[tauri::command]
fn web_get_translated() -> String {
    let lock = GLOBAL_TRANSLATED.lock().unwrap();
    lock.to_string()
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
    let mut origin = tesseract::ocr(new_filename.to_str().unwrap(), "jpn")?;
    origin.retain(|c| c != ' ');
    {
        dbg!(&origin);
        let mut lock = GLOBAL_ORIGIN.lock().unwrap();
        *lock = origin.clone();
    }

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(origin.clone())?;

    if let Some(chi_sim) = translate::translate(&origin) {
        let mut lock = GLOBAL_TRANSLATED.lock().unwrap();
        *lock = chi_sim
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            web_translate,
            web_get_origin,
            web_get_translated,
            web_get_translate_sites,
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}
