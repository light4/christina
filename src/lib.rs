//! Steps:
//! 1. capture screen
//! 2. process image
//! 3. ocr
//! 4. process text
//! 5. copy to clipboard
//! 6. translate
//! 7. show a translate panel

mod app;
mod color;
mod config;
mod ocr;
mod signal;
mod sites;
mod translate;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use arboard::Clipboard;
use color_eyre::Result;
use ocr::capture_image;
use once_cell::sync::Lazy;
use signal::signal_running;

use crate::{
    config::Config,
    ocr::{image_to_text, preprocess_image},
};

const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const INDEX_PAGE: &[u8] = include_bytes!("../index.html");

static GLOBAL_CONFIG: Lazy<Arc<RwLock<Config>>> =
    Lazy::new(|| Arc::new(RwLock::new(Config::new())));
static GLOBAL_ORIGIN: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("それにも、当然ながら関心があった。".to_string()));
static GLOBAL_TRANSLATED: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("对此，我当然很感兴趣。".to_string()));

fn process_and_translate(origin_image: impl AsRef<Path>) -> Result<()> {
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

pub fn real_main() -> Result<()> {
    if let Some(pid) = signal_running() {
        println!("Signal running, pid: {pid}");
        std::process::exit(0);
    }

    let config = GLOBAL_CONFIG.read().unwrap();
    let cache_dir = config.proj_dirs.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }

    let first_arg = std::env::args().nth(1);
    let origin_image = {
        if let Some(s) = first_arg {
            PathBuf::from(s)
        } else {
            capture_image(cache_dir)?
        }
    };

    process_and_translate(origin_image)?;

    app::run()
}
