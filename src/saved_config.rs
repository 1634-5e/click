use std::{fs, fs::File, io::BufReader};

use app_dirs2::{get_app_dir, AppDataType, AppDirsError, AppInfo};
use mki_fork::Keyboard;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const APP_INFO: AppInfo = AppInfo {
    name: "never use",
    author: "Click",
};
const FILE_NAME: &str = "saved_config";

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot get proper configuration dir of this OS")]
    GetDirError,
    #[error("failed to get data from file")]
    IoError,
    #[error("failed to ser/de")]
    SerdeError,
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

impl From<AppDirsError> for Error {
    fn from(_: AppDirsError) -> Self {
        Self::GetDirError
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::SerdeError
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedState {
    pub key_bind: Keyboard,
    pub freq: u64,
    pub always_on_top: bool,
}

impl Default for SavedState {
    fn default() -> Self {
        Self {
            key_bind: Keyboard::F5,
            freq: 10,
            always_on_top: true,
        }
    }
}

pub fn save_config(saved_state: SavedState) -> Result<(), Error> {
    let path = get_app_dir(AppDataType::UserCache, &APP_INFO, "/")?;
    println!("saving config to {:?}", path);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    let serialized = serde_json::to_string_pretty(&saved_state)?;

    fs::write(path.join(FILE_NAME), serialized)?;
    Ok(())
}

pub fn load_config() -> Result<Option<SavedState>, Error> {
    let path = get_app_dir(AppDataType::UserCache, &APP_INFO, "/")?;
    println!("loading config from {:?}", path);
    let file = File::open(path.join(FILE_NAME))?;
    let reader = BufReader::new(file);

    let deserde = serde_json::from_reader(reader)?;

    Ok(deserde)
}
