#[cfg(not(windows))]
compile_error!("This tool is only designed/tested for windows");

mod api;
mod download;
mod fs;
pub(crate) mod options;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UpdateError {
    pub error: anyhow::Error,
    pub should_try_recover: bool,
}

impl UpdateError {
    fn new(error: anyhow::Error, should_try_recover: bool) -> Self {
        Self {
            error,
            should_try_recover,
        }
    }
}

pub fn update(root_path: &Path) -> std::result::Result<(), UpdateError> {
    let options = options::Options::new(root_path);

    // Get download url from api
    let download_url = run("Get download url", || {
        api::get_resource("RSDownload").context("Failed to get download url")
    })
    .map_err(|e| UpdateError::new(e, false))?;

    // Download zip
    let zip = download::download_zip(&download_url)
        .context("Failed to download zip")
        .map_err(|e| UpdateError::new(e, false))?;

    // Create root folder
    run("Creating installation folder", || {
        fs::create_root_folder(&options).context("Failed to create installation folder")
    })
    .map_err(|e| UpdateError::new(e, false))?;

    // Remove old folder
    run("Removing old folder", || {
        fs::remove_old(&options).context("Failed to remove old folder")
    })
    .map_err(|e| UpdateError::new(e, false))?;

    // Move all (except keep_files) into old folder
    run("Removing existing installation", || {
        fs::move_to_old(&options).context("Failed to remove existing installation")
    })
    .map_err(|e| UpdateError::new(e, true))?;

    // Extract zip
    run("Extracting zip", || {
        fs::extract_zip(zip, &options).context("Failed to extract zip")
    })
    .map_err(|e| UpdateError::new(e, true))?;

    // Success
    println!("        Successfully updated\n");

    Ok(())
}

pub fn recover_from_old(root_path: &Path) -> Result<()> {
    let options = options::Options::new(root_path);

    // Try to recover
    run("Trying to recover", || fs::recover_from_old(&options))?;

    // Success
    println!("        Successfully recovered\n");

    Ok(())
}

pub fn start_rat_scanner(root_path: &Path) -> Result<()> {
    let path = [root_path.as_os_str(), OsStr::new("RatScanner.exe")]
        .iter()
        .collect::<PathBuf>();

    std::process::Command::new(path).spawn()?;

    Ok(())
}

fn run<F, T, E>(message: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(250);
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙"])
            .template("  {spinner:.blue}   {msg}"),
    );
    pb.set_message(message);

    let result = f();

    if result.is_ok() {
        pb.finish_with_message(&format!("Done - {}", message));
    } else {
        pb.finish_with_message(message);
    }
    println!();

    result
}
