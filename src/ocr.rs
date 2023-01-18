use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use image::{
    imageops::{self, index_colors, ColorMap},
    io::Reader as ImageReader,
    ImageBuffer,
};
use screenshots::Screen;

use crate::color::MyLevel;

pub fn capture_image(dir: impl AsRef<Path>) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let screen_capturers = Screen::all().expect("get all screen error");
    let screen_capturer = screen_capturers.first().unwrap();
    println!("capturer {screen_capturer:?}");
    let image = screen_capturer.capture().unwrap();
    let buffer = image.buffer();
    let filename = now.to_rfc3339() + ".png";
    let abs_path = dir.as_ref().join(filename);
    let mut file = File::create(&abs_path)?;
    file.write_all(&buffer[..])?;

    Ok(abs_path)
}

pub fn preprocess_image(path: impl AsRef<Path>) -> Result<PathBuf> {
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

pub fn image_to_text(path: impl AsRef<Path>) -> Result<String> {
    // tesseract empty.jpg test -l jpn
    let mut origin = tesseract::ocr(
        path.as_ref()
            .to_str()
            .expect("unable read image path string"),
        "jpn",
    )?;
    origin.retain(|c| c != ' ' && c != '\n' && c != '`' && c != '「' && c != '」');

    // remove trailing char
    let chars: Vec<char> = origin.chars().collect();
    if chars.len() > 3 && chars.get(chars.len() - 2) == Some(&'。') {
        origin.pop();
    }

    Ok(origin)
}
