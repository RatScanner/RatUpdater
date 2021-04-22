use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub const OLD_FOLDER_NAME: &'static str = "RatScanner.old";
pub const UNKNOWN_FOLDER_NAME: &'static str = "RatScanner.unknown";
pub const FILES_REF_FILE_NAME: &'static str = "RatScanner.files.ref";

#[derive(Debug)]
pub struct Options {
    pub root_path: PathBuf,
    pub keep_files: HashSet<PathBuf>,
    pub old_folder_path: PathBuf,
    pub unknown_folder_path: PathBuf,
    pub files_ref_path: PathBuf,
}

impl Options {
    pub fn new(root_path: &Path) -> Self {
        Self {
            root_path: root_path.to_owned(),
            keep_files: ["config.cfg"]
                .iter()
                .map(|&file| [root_path.as_os_str(), OsStr::new(file)].iter().collect())
                .collect(),
            old_folder_path: [root_path.as_os_str(), OsStr::new(OLD_FOLDER_NAME)]
                .iter()
                .collect(),
            unknown_folder_path: [root_path.as_os_str(), OsStr::new(UNKNOWN_FOLDER_NAME)]
                .iter()
                .collect(),
            files_ref_path: [root_path.as_os_str(), OsStr::new(FILES_REF_FILE_NAME)]
                .iter()
                .collect(),
        }
    }
}
