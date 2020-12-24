use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Options {
    pub root_path: PathBuf,
    pub keep_files: HashSet<PathBuf>,
    pub old_folder_path: PathBuf,
}

impl Options {
    pub fn new(root_path: &Path) -> Self {
        Self {
            root_path: root_path.to_owned(),
            keep_files: ["config.cfg"]
                .iter()
                .map(|&file| [root_path.as_os_str(), OsStr::new(file)].iter().collect())
                .collect(),
            old_folder_path: [root_path.as_os_str(), OsStr::new("RatScanner.old")]
                .iter()
                .collect(),
        }
    }
}
