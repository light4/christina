use anyhow::Result;
use regex::Regex;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, ORIGIN, REFERER, USER_AGENT},
};

use std::collections::HashMap;

const MOCK_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Mobile/15E148 Safari/604.1";

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(MOCK_USER_AGENT));
    // headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://m.youdao.com"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,en-US;q=0.7,en;q=0.3"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://m.youdao.com/translate"),
    );
    headers
}

fn search(text: &str) -> Result<String> {
    let url = "https://m.youdao.com/translate";
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("inputtext", text);
    params.insert("type", "AUTO");

    let res = client
        .post(url)
        .headers(construct_headers())
        .form(&params)
        .send()?;

    // dbg!(res.status());
    let res_html = res.text()?;

    // dbg!(&res_html);
    Ok(res_html)
}

fn find_translated_from_html(html: &str) -> Option<String> {
    let re = Regex::new(r###"(?m)(?s)<ul id="translateResult".*?/ul>"###).unwrap();
    re.find(html).map(|mat| {
        let text = re
            .captures_iter(mat.as_str())
            .map(|cap| cap[0].to_string())
            .collect::<Vec<_>>()
            .join("");
        let dom = tl::parse(&text, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        dom.nodes()
            .iter()
            .filter_map(|n| n.as_tag())
            .filter(|n| n.name() == "li")
            .map(|n| n.inner_text(parser))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .into()
    })
}

pub fn translate(text: &str) -> Option<String> {
    if let Ok(html) = search(text) {
        find_translated_from_html(&html)
    } else {
        None
    }
}
