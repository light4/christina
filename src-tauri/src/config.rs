use directories::ProjectDirs;

pub struct Config {
    pub proj_dirs: ProjectDirs,
}

impl Config {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("io", "i01", "christina")
            .expect("cannot construct project directories");
        Self { proj_dirs }
    }
}
