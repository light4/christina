use std::path::Path;

use anyhow::Result;
use signal_hook::{consts::signal::*, iterator::Signals};
use tauri::{Manager, Window};

use crate::{capture_image, do_the_job};

pub fn handle_signals(cache_dir: &Path, mut signals: Signals, window: &Window) -> Result<()> {
    dbg!("running handle signals task");
    for signal in signals.forever() {
        match signal {
            SIGHUP => {
                // reload config
            }
            SIGINT => {
                println!("Received signal {signal}");
                let origin_image = capture_image(cache_dir).expect("capture image error");
                do_the_job(origin_image).expect("unable to get the image content");
                window
                    .emit_all("reload_content", ())
                    .expect("unable to send event: reload_content");
            }
            SIGTERM | SIGQUIT => {
                // Shutdown the system;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
