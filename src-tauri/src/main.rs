#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod color;
mod sites;
mod translate;
mod web_cmds;

use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
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
use signal_hook::{consts::SIGINT, iterator::Signals};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

static GLOBAL_ORIGIN: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("それにも、当然ながら関心があった。".to_string()));
static GLOBAL_TRANSLATED: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("对此，我当然很感兴趣。".to_string()));

fn capture_image(path: impl AsRef<Path>) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let screen_capturers = Screen::all().expect("get all screen error");
    let screen_capturer = screen_capturers.first().unwrap();
    println!("capturer {screen_capturer:?}");
    let image = screen_capturer.capture().unwrap();
    let buffer = image.buffer();
    let filename = now.to_rfc3339() + ".png";
    let abs_path = path.as_ref().join(filename);
    let mut file = File::create(&abs_path)?;
    file.write_all(&buffer[..])?;

    Ok(abs_path)
}

fn preprocess_image(path: impl AsRef<Path>) -> Result<PathBuf> {
    let img = ImageReader::open(path.as_ref())?.decode()?.to_rgba8();
    let sub_img = imageops::crop_imm(&img, 0, 770, 1920, 310).to_image();
    let cmap = MyLevel;

    let palletized = index_colors(&sub_img, &cmap);
    let mapped = ImageBuffer::from_fn(sub_img.width(), sub_img.height(), |x, y| {
        let p = palletized.get_pixel(x, y);
        cmap.lookup(p.0[0] as usize)
            .expect("indexed color out-of-range")
    });

    let new_filename = path.as_ref().parent().unwrap().join("processed.jpg");
    mapped.save(&new_filename)?;

    Ok(new_filename)
}

fn image_to_text(path: impl AsRef<Path>) -> Result<String> {
    // tesseract empty.jpg test -l jpn
    let mut origin = tesseract::ocr(
        path.as_ref()
            .to_str()
            .expect("unable read image path string"),
        "jpn",
    )?;
    origin.retain(|c| c != ' ' && c != '\n');

    Ok(origin)
}

fn do_the_job(origin_image: impl AsRef<Path>) -> Result<()> {
    let new_image = preprocess_image(origin_image)?;
    let origin = image_to_text(new_image)?;
    println!("origin text: {origin}");

    {
        let mut lock = GLOBAL_ORIGIN.lock().unwrap();
        *lock = origin.clone();
    }

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(origin.clone())?;

    if let Some(chi_sim) = translate::translate(&origin) {
        println!("translated text: {chi_sim}");
        let mut lock = GLOBAL_TRANSLATED.lock().unwrap();
        *lock = chi_sim
    }

    Ok(())
}

pub fn system_tray_event_handler(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a double click");
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "toggle" => {
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                }
            }
            "about" => {
                open::that(HOMEPAGE).unwrap();
            }
            "quit" => {
                std::process::exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

fn main() -> Result<()> {
    let proj_dirs =
        ProjectDirs::from("com", "i01", "christina").expect("cannot construct project directories");
    let cache_dir = proj_dirs.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }

    let mut signals = Signals::new(&[SIGINT])?;

    let cache_dir_clone = cache_dir.to_owned().clone();
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            let origin_image = capture_image(&cache_dir_clone).expect("capture image error");
            do_the_job(origin_image).expect("unable to get the image content");
        }
    });

    let first_arg = std::env::args().nth(1);
    let origin_image = {
        if let Some(s) = first_arg {
            PathBuf::from(s)
        } else {
            capture_image(cache_dir)?
        }
    };

    do_the_job(origin_image)?;

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("toggle", "Toggle"))
        .add_item(CustomMenuItem::new("about", "About"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"));
    let system_tray = SystemTray::new().with_menu(tray_menu);

    use web_cmds::*;
    let app = tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(system_tray_event_handler)
        .invoke_handler(tauri::generate_handler![
            web_translate,
            web_get_origin,
            web_get_translated,
            web_get_translate_sites,
            web_open_homepage,
        ])
        .build(tauri::generate_context!())
        .context("error while building tauri application")?;

    app.run(|app_handle, event| match event {
        tauri::RunEvent::WindowEvent { label, event, .. } => match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let window = app_handle
                    .get_window(label.as_str())
                    .expect("get window error");
                window.hide().expect("hide window error");
                api.prevent_close();
            }
            _ => {}
        },
        _ => {}
    });

    Ok(())
}
