use directories::ProjectDirs;

use crate::PKG_NAME;

pub struct Config {
    pub proj_dirs: ProjectDirs,
}

impl Config {
    pub fn new() -> Self {
        let proj_dirs =
            ProjectDirs::from("io", "i01", PKG_NAME).expect("cannot construct project directories");
        Self { proj_dirs }
    }
}
