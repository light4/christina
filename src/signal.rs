use color_eyre::Result;
use signal_hook::{consts::signal::*, iterator::Signals};
use sysinfo::{PidExt, ProcessExt, Signal, System, SystemExt};
use wry::application::event_loop::EventLoopProxy;

use crate::{app::UserEvent, PKG_NAME};

pub fn handle_signals(mut signals: Signals, proxy: &EventLoopProxy<UserEvent>) -> Result<()> {
    dbg!("running handle signals task");
    for signal in signals.forever() {
        match signal {
            SIGHUP => {
                // reload config
            }
            SIGINT => {
                println!("Received signal {signal}");
                proxy.send_event(UserEvent::DoTheJobOnce)?;
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
