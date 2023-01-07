use std::path::Path;

use anyhow::Result;
use signal_hook::{consts::signal::*, iterator::Signals};
use sysinfo::{PidExt, ProcessExt, Signal, System, SystemExt};
use tauri::{Manager, Window};

use crate::{capture_image, do_the_job, PKG_NAME};

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

pub fn signal_running() -> Option<u32> {
    let my_pid = std::process::id();
    dbg!(my_pid);
    let mut sys = System::new_all();
    sys.refresh_processes();
    for (pid, process) in sys.processes() {
        if pid.as_u32() != my_pid && process.name() == PKG_NAME {
            println!("{pid:15} {}", process.name());
            process.kill_with(Signal::Interrupt);
            return Some(pid.as_u32());
        }
    }

    None
}
