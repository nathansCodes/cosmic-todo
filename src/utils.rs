use std::path::PathBuf;

use crate::APP_ID;

pub fn data_path() -> PathBuf {
    let path = xdg_basedir::get_data_home();
    match path {
        Ok(mut path) => {
            path.push(APP_ID);
            std::fs::create_dir_all(&path).expect("Could not create directory.");
            path.push("data.json");
            path
        }
        Err(_) => unreachable!(),
    }
}
