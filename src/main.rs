use std::process;

fn main() {
    // Windows Setup
    #[cfg(windows)]
    windows_setup::windows_setup();

    // Get args
    let args = match args::get_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("   X    Failed to parse args: {:?}\n", e);

            pause();
            process::exit(1);
        }
    };

    // Update rat scanner
    if args.update {
        if let Err(e) = rat_updater::update(&args.root_path) {
            eprintln!("   X    An error occurred while updating: {:?}\n", e.error);

            // Recover from old
            if e.should_try_recover {
                if let Err(e) = rat_updater::recover_from_old(&args.root_path) {
                    eprintln!("   X    An error occurred while recovering: {:?}\n", e);
                }
            }

            pause();
            process::exit(1);
        }
    }

    // Start rat scanner
    if args.start {
        if let Err(e) = rat_updater::start_rat_scanner(&args.root_path) {
            eprintln!(
                "   X    An error occurred while starting RatScanner: {:?}\n",
                e
            );

            pause();
            process::exit(1);
        }
    }
}

fn pause() {
    use std::io;
    use std::io::prelude::*;

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("Press enter to continue...");
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
}

mod args {
    use anyhow::{anyhow, Context};
    use std::path::PathBuf;

    pub struct Args {
        pub root_path: PathBuf,
        pub update: bool,
        pub start: bool,
    }

    pub fn get_args() -> anyhow::Result<Args> {
        let mut args_iter = std::env::args().skip(1);

        let mut root_path = None;
        let mut update = None;
        let mut start = None;

        while let Some(arg) = args_iter.next() {
            if arg == "--root-path" {
                match args_iter.next() {
                    Some(val) => root_path = Some(val.into()),
                    None => return Err(anyhow!("Missing value for {}", arg)),
                }
            } else if arg == "--update" {
                update = Some(true);
            } else if arg == "--start" {
                start = Some(true);
            } else {
                return Err(anyhow!("Unrecognized argument {}", arg));
            }
        }

        Ok(Args {
            root_path: match root_path {
                Some(root_path) => root_path,
                None => std::env::current_exe()
                    .context("Could not get current exe")?
                    .parent()
                    .unwrap()
                    .to_path_buf(),
            },
            update: update.unwrap_or(false),
            start: start.unwrap_or(false),
        })
    }
}

#[cfg(windows)]
mod windows_setup {
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_INPUT_HANDLE;
    use winapi::um::wincon::{ENABLE_EXTENDED_FLAGS, ENABLE_QUICK_EDIT_MODE};

    pub fn windows_setup() {
        // Disable quick edit
        unsafe {
            let h_input = GetStdHandle(STD_INPUT_HANDLE);
            let mut prev_mode = 0;
            GetConsoleMode(h_input, &mut prev_mode);
            SetConsoleMode(
                h_input,
                (prev_mode | ENABLE_EXTENDED_FLAGS) & !ENABLE_QUICK_EDIT_MODE,
            );
        }
    }
}
