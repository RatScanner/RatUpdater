use crate::options::Options;
use anyhow::Result;
use std::fs::{self, File};

pub fn create_root_folder(options: &Options) -> Result<()> {
    std::fs::create_dir_all(&options.root_path)?;

    Ok(())
}

pub fn remove_old(options: &Options) -> Result<()> {
    // Skip if path does not exist
    if !options.old_folder_path.exists() {
        return Ok(());
    }

    // Remove dir
    remove_dir_all::remove_dir_all(&options.old_folder_path)?;

    Ok(())
}

pub fn move_to_old(options: &Options) -> Result<()> {
    // Create old folder
    fs::create_dir(&options.old_folder_path)?;

    // Move to old folder except: old_folder_path, keep_files
    for entry in options.root_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path == options.old_folder_path || options.keep_files.contains(&path) {
            continue;
        }

        let mut to = options.old_folder_path.clone();
        to.push(entry.file_name());

        fs::rename(path, to)?;
    }

    Ok(())
}

pub fn extract_zip(zip: File, options: &Options) -> Result<()> {
    let mut archive = zip::ZipArchive::new(zip)?;
    archive.extract(&options.root_path)?;

    Ok(())
}

pub fn recover_from_old(options: &Options) -> Result<()> {
    // Move from old to root
    for entry in options.old_folder_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        let mut to = options.root_path.clone();
        to.push(entry.file_name());

        fs::rename(path, to)?;
    }

    Ok(())
}
