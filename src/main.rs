#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    christina::real_main()
}
