use crate::{sites, translate, GLOBAL_ORIGIN, GLOBAL_TRANSLATED, HOMEPAGE};

#[tauri::command]
pub const fn web_get_homepage() -> &'static str {
    HOMEPAGE
}

#[tauri::command]
pub fn web_translate(origin: &str) -> String {
    translate::translate(origin).unwrap_or_else(|| "翻译出错了".to_string())
}

#[tauri::command]
pub fn web_get_origin() -> String {
    let lock = GLOBAL_ORIGIN.lock().unwrap();
    lock.to_string()
}

#[tauri::command]
pub fn web_get_translated() -> String {
    let lock = GLOBAL_TRANSLATED.lock().unwrap();
    lock.to_string()
}

#[tauri::command]
pub fn web_get_translate_sites(origin: &str) -> Vec<(String, String)> {
    sites::get_translate_sites(origin)
}
