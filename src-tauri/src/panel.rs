use std::{
    fs::File,
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

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
