use serial_test::serial;
use std::ffi::OsStr;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[serial]
fn integration() {
    // Run Updater
    let mut child = Command::new(executable_path())
        .args(&[
            OsStr::new("--root-path"),
            root_path().as_os_str(),
            OsStr::new("--update"),
        ])
        .spawn()
        .unwrap();

    let start = Instant::now();
    let end = start + Duration::from_secs(300);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    break;
                } else {
                    panic!("Failed");
                }
            }
            Ok(None) => {
                if Instant::now() > end {
                    panic!("Timed out");
                }
            }
            Err(e) => {
                child.kill().unwrap();
                panic!("{:?}", e);
            }
        }
        thread::yield_now();
    }

    // Check exists
    let root_path = root_path();
    assert!(make_path(&root_path, "RatScanner.old").exists());
    assert!(make_path(&root_path, "RatScanner.files.ref").exists());
    assert!(make_path(&root_path, "RatScanner.exe").exists());
}

#[test]
#[serial]
fn update() {
    let root_path = root_path();

    // Clear root path
    if root_path.exists() {
        remove_dir_all::remove_dir_all(&root_path).unwrap();
        fs::create_dir_all(&root_path).unwrap();
    }

    // Add existing file
    fs::File::create(make_path(&root_path, "my-file.txt"))
        .unwrap()
        .write_all(b"Hello, world!")
        .unwrap();

    // Update
    rat_updater::update(&root_path).unwrap();

    // Check exists
    assert!(make_path(&root_path, "RatScanner.old").exists());
    assert!(make_path(&root_path, "RatScanner.unknown").exists());
    assert!(make_path(&root_path, "RatScanner.files.ref").exists());
    assert!(make_path(&root_path, "RatScanner.exe").exists());

    // Check RatScanner.old is empty
    assert!(make_path(&root_path, "RatScanner.old")
        .read_dir()
        .unwrap()
        .next()
        .is_none());

    // Check RatScanner.unknown (my-file.txt)
    assert_eq!(
        make_path(&root_path, "RatScanner.unknown")
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name(),
        "my-file.txt"
    );

    // Update again
    rat_updater::update(&root_path).unwrap();

    // Check exists
    assert!(make_path(&root_path, "RatScanner.old").exists());
    assert!(make_path(&root_path, "RatScanner.unknown").exists());
    assert!(make_path(&root_path, "RatScanner.files.ref").exists());
    assert!(make_path(&root_path, "RatScanner.exe").exists());

    // Check RatScanner.old is not empty
    assert!(make_path(&root_path, "RatScanner.old")
        .read_dir()
        .unwrap()
        .next()
        .is_some());

    // Check RatScanner.unknown (my-file.txt)
    assert_eq!(
        make_path(&root_path, "RatScanner.unknown")
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name(),
        "my-file.txt"
    );
}

#[test]
#[serial]
fn recover() {
    let root_path = root_path();

    // Clear root path
    if root_path.exists() {
        remove_dir_all::remove_dir_all(&root_path).unwrap();
        fs::create_dir_all(&root_path).unwrap();
    }

    // Setup
    fs::create_dir_all(make_path(&root_path, "RatScanner.old/old_x")).unwrap();

    fs::File::create(make_path(&root_path, "q.txt")).unwrap();
    fs::File::create(make_path(&root_path, "existing.txt"))
        .unwrap()
        .write_all(b"Hello, world!")
        .unwrap();
    fs::File::create(make_path(&root_path, "w.txt")).unwrap();

    fs::File::create(make_path(&root_path, "RatScanner.old/old_a.txt")).unwrap();
    fs::File::create(make_path(&root_path, "RatScanner.old/existing.txt"))
        .unwrap()
        .write_all(b"Hello, world! (Recovered)")
        .unwrap();
    fs::File::create(make_path(&root_path, "RatScanner.old/old_x/old_b.txt")).unwrap();

    // Recover
    rat_updater::recover_from_old(&root_path).unwrap();

    // Check exists
    assert!(make_path(&root_path, "q.txt").exists());
    assert!(make_path(&root_path, "existing.txt").exists());
    assert!(make_path(&root_path, "w.txt").exists());

    assert!(make_path(&root_path, "old_a.txt").exists());
    assert!(make_path(&root_path, "old_x/old_b.txt").exists());

    // Check content
    let mut file = fs::File::open(make_path(&root_path, "existing.txt")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "Hello, world! (Recovered)");
}

fn make_path(base: impl AsRef<Path>, path: &str) -> PathBuf {
    [base.as_ref().as_os_str(), OsStr::new(path)]
        .iter()
        .collect()
}

fn root_path() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tmp"].iter().collect()
}

fn executable_path() -> PathBuf {
    env!("CARGO_BIN_EXE_rat-updater").into()
}
