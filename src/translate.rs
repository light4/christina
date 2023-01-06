use std::collections::HashMap;

use anyhow::Result;
use regex::Regex;

const MOCK_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Mobile/15E148 Safari/604.1";

fn search(text: &str) -> Result<String> {
    let url = "https://m.youdao.com/translate";

    let mut params = HashMap::new();
    params.insert("inputtext", text);
    params.insert("type", "AUTO");

    let res = ureq::post(url)
        .set("user-agent", MOCK_USER_AGENT)
        .set("origin", "https://m.youdao.com")
        .set("accept-language", "zh-CN,en-US;q=0.7,en;q=0.3")
        .set("referer", url)
        .send_form(&[("inputtext", text), ("type", "AUTO")])?;

    // dbg!(res.status());
    let res_html = res.into_string()?;

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
