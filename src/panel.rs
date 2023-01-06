use std::{
    fs::File,
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

use eframe::egui::{self, FontDefinitions};

fn add_fonts(fonts: &mut FontDefinitions, font_name: &str, font_path: impl AsRef<Path>) {
    let mut font_data = vec![];
    let mut file = File::open(font_path).expect("open font file error");
    file.read_to_end(&mut font_data)
        .expect("read font data error");
    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts
        .font_data
        .insert(font_name.to_string(), egui::FontData::from_owned(font_data));

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, font_name.to_string());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(font_name.to_string());
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    add_fonts(
        &mut fonts,
        "Source Han Sans JP",
        "/usr/share/fonts/adobe-source-han-sans/SourceHanSansJP-Regular.otf",
    );
    add_fonts(
        &mut fonts,
        "Source Han Sans CN",
        "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
    );

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn show(origin: &str, translated: &str, timeout: Option<Duration>) {
    // there is no focus_window option?
    let options = eframe::NativeOptions {
        // Hide the OS-specific "chrome" around the window:
        // decorated: false,
        // To have rounded corners we need transparency:
        transparent: true,
        always_on_top: true,
        min_window_size: Some(egui::vec2(320.0, 100.0)),
        initial_window_size: Some(egui::vec2(320.0, 300.0)),
        ..Default::default()
    };

    let origin = origin.to_owned();
    let translated = translated.to_owned();
    eframe::run_native(
        env!("CARGO_PKG_NAME"), // unused title
        options,
        Box::new(move |cc| Box::new(MyApp::new(&origin, &translated, timeout, cc))),
    )
}

pub struct MyApp {
    pub origin: String,
    pub translated: String,
    pub creat_at: Instant,
    pub timeout: Option<Duration>,
}

impl MyApp {
    pub fn new(
        origin: &str,
        translated: &str,
        timeout: Option<Duration>,
        cc: &eframe::CreationContext<'_>,
    ) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            origin: origin.to_owned(),
            translated: translated.to_owned(),
            creat_at: Instant::now(),
            timeout,
        }
    }

    pub fn google_translate_url(&self) -> String {
        format!(
            "https://translate.google.com.hk/?sl=auto&tl=zh-CN&text={}&op=translate",
            self.origin
        )
    }

    pub fn youdao_url(&self) -> String {
        "https://fanyi.youdao.com/index.html".to_string()
    }

    pub fn deepl_url(&self) -> String {
        format!("https://www.deepl.com/translator#ja/zh/{}", self.origin)
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(env!("CARGO_PKG_NAME"));
            ui.text_edit_multiline(&mut self.origin);
            ui.text_edit_multiline(&mut self.translated);

            // links
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.hyperlink_to("Google", self.google_translate_url());
                ui.hyperlink_to("YouDao", self.youdao_url());
                ui.hyperlink_to("DeepL", self.deepl_url());
            });
        });
        if let Some(d) = self.timeout {
            if self.creat_at.elapsed() > d {
                println!("Times up, stopping.");
                frame.close();
            }
        }
    }
}
