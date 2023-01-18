use std::{
    borrow::Cow,
    fs::{canonicalize, read},
    sync::{Arc, RwLock},
    thread,
};

use color_eyre::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use wry::{
    application::{
        event::{DeviceEvent, ElementState, Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopProxy},
        keyboard,
        menu::{ContextMenu, MenuId, MenuItem, MenuItemAttributes},
        system_tray::{SystemTray, SystemTrayBuilder},
        window::{Icon, Window, WindowBuilder},
    },
    http::{header::CONTENT_TYPE, Response},
    webview::{WebView, WebViewBuilder},
};

use crate::{
    ocr::capture_image, process_and_translate, signal::handle_signals, sites::get_translate_sites,
    translate::translate, GLOBAL_CONFIG, GLOBAL_ORIGIN, GLOBAL_TRANSLATED, HOMEPAGE, INDEX_PAGE,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransQuery {
    origin: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransData {
    origin: String,
    translated: String,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    NewWindow(String),
    DoTheJobOnce,
}

static GLOBAL_EVENTLOOP: OnceCell<Arc<RwLock<EventLoopProxy<UserEvent>>>> = OnceCell::new();

fn ipc_handler(window: &Window, msg: String) {
    println!("ipc_handler: [window] {window:?} [msg] {msg}");
}

// capture, process, translate, reload web content
pub fn do_the_job_once(webview: &WebView) -> Result<()> {
    let config = GLOBAL_CONFIG.read().unwrap();
    let cache_dir = config.proj_dirs.cache_dir();
    let origin_image = capture_image(cache_dir)?;
    process_and_translate(origin_image)?;
    webview.evaluate_script("window.load_content()")?;

    Ok(())
}

fn get_path_mime(path: &str) -> &'static str {
    // Return asset contents and mime types based on file extentions
    // If you don't want to do this manually, there are some crates for you.
    // Such as `infer` and `mime_guess`.
    if path.ends_with(".html") || path == "/" {
        "text/html"
    } else if path.ends_with(".js") {
        "text/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else {
        unimplemented!();
    }
}

fn process_request<T>(
    request: &wry::http::Request<T>,
) -> wry::Result<Response<Cow<'static, [u8]>>> {
    let path = request.uri().path();

    let content = match path {
        "/" => INDEX_PAGE.into(),
        "/data.json" => {
            let origin = GLOBAL_ORIGIN.lock().unwrap().to_string();
            let translated = GLOBAL_TRANSLATED.lock().unwrap().to_string();
            serde_json::to_vec(&TransData { origin, translated })?
        }
        "/translate.json" => {
            let origin = request
                .uri()
                .query()
                .and_then(|i| serde_qs::from_str::<TransQuery>(i).ok())
                .map(|q| q.origin)
                .unwrap_or_else(|| GLOBAL_ORIGIN.lock().unwrap().to_string());
            let translated = translate(&origin).unwrap_or("".to_string());
            {
                let mut o_lock = GLOBAL_ORIGIN.lock().unwrap();
                *o_lock = origin.clone();
                let mut t_lock = GLOBAL_TRANSLATED.lock().unwrap();
                *t_lock = translated.clone();
            }
            serde_json::to_vec(&TransData { origin, translated })?
        }
        "/sites.json" => {
            let origin = request
                .uri()
                .query()
                .and_then(|i| serde_qs::from_str::<TransQuery>(i).ok())
                .map(|q| q.origin)
                .unwrap_or_else(|| GLOBAL_ORIGIN.lock().unwrap().to_string());
            serde_json::to_vec(&get_translate_sites(&origin))?
        }
        _ => {
            // `1..` for removing leading slash
            read(canonicalize(&path[1..])?)?
        }
    };

    let mimetype = get_path_mime(path);
    Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .body(content.into())
        .map_err(Into::into)
}

pub fn run() -> Result<()> {
    let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();
    GLOBAL_EVENTLOOP.set(Arc::new(RwLock::new(proxy))).unwrap();

    let window = WindowBuilder::new()
        .with_title("Christina")
        .with_always_on_top(true)
        .build(&event_loop)?;

    let _tray = setup_system_tray(&event_loop)?;

    let webview = WebViewBuilder::new(window)
        .unwrap()
        .with_ipc_handler(ipc_handler)
        .with_custom_protocol("wry".into(), move |request| process_request(request))
        .with_new_window_req_handler(move |uri: String| {
            let proxy_lock = GLOBAL_EVENTLOOP.get().unwrap();
            let proxy = proxy_lock.read().unwrap();
            let submitted = proxy.send_event(UserEvent::NewWindow(uri.clone())).is_ok();

            submitted && uri.contains("github.com")
        })
        // tell the webview to load the custom protocol
        .with_url("wry://localhost")?
        .build()?;

    thread::spawn(move || {
        dbg!("spawn handle signals task");
        let proxy_lock = GLOBAL_EVENTLOOP.get().unwrap();
        let proxy = proxy_lock.read().unwrap();
        let signals = Signals::new([SIGINT]).expect("register signals error");
        handle_signals(signals, &proxy).expect("error while handle signals");
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry application started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // user event start
            Event::UserEvent(UserEvent::NewWindow(uri)) => {
                open::that(&uri).unwrap_or_else(|e| eprintln!("open {uri} error {e:?}"));
            }
            Event::UserEvent(UserEvent::DoTheJobOnce) => do_the_job_once(&webview)
                .unwrap_or_else(|e| eprintln!("something wrong while process: {e:?}")),
            // user event end
            Event::MenuEvent {
                window_id: _,
                menu_id,
                origin: _,
                ..
            } => system_tray_event_handler(menu_id, &webview),
            Event::DeviceEvent {
                device_id: _,
                event: DeviceEvent::Key(key),
                ..
            } => {
                let window = webview.window();
                if !window.is_focused()
                    && key.state == ElementState::Released
                    && matches!(
                        key.physical_key,
                        keyboard::KeyCode::Enter
                            | keyboard::KeyCode::Space
                            | keyboard::KeyCode::NumpadEnter
                    )
                {
                    dbg!(key);
                    do_the_job_once(&webview).expect("something wrong");
                }
            }
            _ => (),
        }
    });
}

fn setup_system_tray<T>(event_loop: &EventLoop<T>) -> Result<SystemTray> {
    let mut tray_menu = ContextMenu::new();
    let image_data = include_bytes!("../assets/icon.png");
    let icon_image = image::load_from_memory_with_format(image_data, image::ImageFormat::Png)?;
    let icon_rgba = icon_image.as_rgba8().unwrap().to_vec();
    let icon = Icon::from_rgba(icon_rgba, icon_image.width(), icon_image.height())?;

    tray_menu.add_item(MenuItemAttributes::new("Capture"));
    tray_menu.add_item(MenuItemAttributes::new("Toggle"));
    tray_menu.add_item(MenuItemAttributes::new("About"));
    tray_menu.add_native_item(MenuItem::Separator);
    tray_menu.add_item(MenuItemAttributes::new("Quit"));

    let system_tray = SystemTrayBuilder::new(icon, Some(tray_menu)).build(event_loop)?;
    Ok(system_tray)
}

pub fn system_tray_event_handler(menu_id: MenuId, webview: &WebView) {
    let capture_id = MenuId::new("Capture");
    let toggle_id = MenuId::new("Toggle");
    let about_id = MenuId::new("About");
    let quit_id = MenuId::new("Quit");

    if menu_id == capture_id {
        do_the_job_once(webview).expect("something wrong");
    } else if menu_id == toggle_id {
        let window = webview.window();
        if window.is_visible() {
            window.set_visible(false);
        } else {
            window.set_visible(true);
        }
    } else if menu_id == about_id {
        open::that(HOMEPAGE).unwrap();
    } else if menu_id == quit_id {
        std::process::exit(0);
    } else {
    }
}
