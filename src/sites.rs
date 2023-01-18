use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Site {
    brand: String,
    url: String,
}

impl Site {
    pub fn new(brand: impl ToString, url: impl ToString) -> Self {
        Self {
            brand: brand.to_string(),
            url: url.to_string(),
        }
    }
}

pub fn get_translate_sites(origin: &str) -> Vec<Site> {
    let mut result = vec![];
    result.push(Site::new(
        "Google",
        format!("https://translate.google.com.hk/?sl=auto&tl=zh-CN&text={origin}&op=translate"),
    ));

    result.push(Site::new("YouDao", "https://fanyi.youdao.com/index.html"));

    result.push(Site::new(
        "Bing",
        format!("https://cn.bing.com/translator/?to=cn&text={origin}"),
    ));

    result.push(Site::new(
        "DeepL",
        format!("https://www.deepl.com/translator#ja/zh/{origin}"),
    ));

    result.push(Site::new(
        "NihongoDera",
        "https://nihongodera.com/tools/romaji-converter".to_string(),
    ));

    result
}
