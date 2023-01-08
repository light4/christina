pub fn get_translate_sites(origin: &str) -> Vec<(String, String)> {
    let mut result = vec![];
    result.push((
        "Google".to_string(),
        format!("https://translate.google.com.hk/?sl=auto&tl=zh-CN&text={origin}&op=translate"),
    ));

    result.push((
        "YouDao".to_string(),
        "https://fanyi.youdao.com/index.html".to_string(),
    ));

    result.push((
        "Bing".to_string(),
        format!("https://cn.bing.com/translator/?to=cn&text={origin}"),
    ));

    result.push((
        "DeepL".to_string(),
        format!("https://www.deepl.com/translator#ja/zh/{origin}"),
    ));

    result.push((
        "NihongoDera".to_string(),
        "https://nihongodera.com/tools/romaji-converter".to_string(),
    ));

    result
}
