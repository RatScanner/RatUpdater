use crate::options::{Options, FILES_REF_FILE_NAME};
use anyhow::Result;
use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

    // Check files ref path exists
    if !options.files_ref_path.exists() {
        return Ok(());
    }

    // Get created list and prefix with root path
    let created =
        serde_json::from_reader::<_, HashSet<PathBuf>>(File::open(&options.files_ref_path)?)?
            .into_iter()
            .map(|path| options.root_path.join(path))
            .collect::<HashSet<_>>();

    // Go through all files in root path
    for entry in options.root_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        // Move to old folder if
        // - in created and
        // - not in keep files
        let move_to_old = created.contains(&path) && !options.keep_files.contains(&path);

        if move_to_old {
            let to = options.old_folder_path.join(entry.file_name());
            fs::rename(path, to)?;
        }
    }

    Ok(())
}

pub fn move_to_unknown(options: &Options) -> Result<()> {
    // Folder
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let unique_unknown_folder = options
        .unknown_folder_path
        .join(since_the_epoch.as_millis().to_string());

    // Flag if folder is created
    let mut folder_is_created = false;

    // Go through all files in root path
    for entry in options.root_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        // Move to unknown folder if
        // - not equals old folder
        // - not equals unknown folder
        // - not in keep files
        let move_to_unknown = path != options.old_folder_path
            && path != options.unknown_folder_path
            && !options.keep_files.contains(&path);

        if move_to_unknown {
            if !folder_is_created {
                // Create folder
                fs::create_dir_all(&unique_unknown_folder)?;
                folder_is_created = true;
            }

            let to = unique_unknown_folder.join(entry.file_name());
            fs::rename(path, to)?;
        }
    }

    Ok(())
}

pub fn extract_zip(zip: File, options: &Options) -> Result<HashSet<PathBuf>> {
    let mut archive = zip::ZipArchive::new(zip)?;
    let created = extract_(&mut archive, &options.root_path)?;

    Ok(created)
}

pub fn save_files_ref(
    mut created: HashSet<PathBuf>,
    options: &Options,
) -> Result<HashSet<PathBuf>> {
    // Add files ref file
    created.insert(FILES_REF_FILE_NAME.into());

    // Create and write created
    let file = File::create(&options.files_ref_path)?;
    serde_json::to_writer(file, &created)?;

    Ok(created)
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

/// Extract a Zip archive into a directory, overwriting files if they
/// already exist. Paths are sanitized with [`ZipFile::enclosed_name`].
///
/// Extraction is not atomic; If an error is encountered, some of the files
/// may be left on disk.
///
/// Returns set of created paths.
// Impl based on zip::ZipArchive::extract(...)
fn extract_<P: AsRef<Path>>(
    self_: &mut zip::ZipArchive<File>,
    directory: P,
) -> zip::result::ZipResult<HashSet<PathBuf>> {
    let mut created = HashSet::new();

    for i in 0..self_.len() {
        let mut file = self_.by_index(i)?;
        let filepath = file
            .enclosed_name()
            .ok_or(zip::result::ZipError::InvalidArchive("Invalid file path"))?;

        // Only add to created if
        // - has parent => path is not ""
        // - parent does not have a parent => in this case we can just store the parent
        for ancestor in filepath.ancestors() {
            if let Some(parent) = ancestor.parent() {
                if parent.parent().is_none() {
                    created.insert(ancestor.to_path_buf());
                }
            }
        }

        let outpath = directory.as_ref().join(filepath);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(created)
}
