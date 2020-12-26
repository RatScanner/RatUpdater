use anyhow::{anyhow, bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, Read};
use tempfile::tempfile;

pub fn download_zip(download_url: &str) -> Result<File> {
    // Make request
    let response = ureq::get(download_url)
        .timeout_connect(15_000)
        .timeout_read(15_000)
        .call();

    // Check for synthetic error
    if response.synthetic() {
        let error = response.into_synthetic_error().unwrap();
        return Err(error.into());
    }

    // Check for other errors
    if response.error() {
        bail!("Received status code {}", response.status());
    }

    // Get content length
    let content_length = response
        .header("Content-Length")
        .ok_or(anyhow!("Missing Content-Length"))?
        .parse()
        .context("Invalid Content-Length")?;

    // Create progress bar
    let progress_bar = ProgressBar::new(content_length);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("  {spinner:.blue}   Downloading new version\n\n  [{elapsed_precise}] [{bar:32.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙"])
        .progress_chars("#>-"));
    progress_bar.tick();
    progress_bar.enable_steady_tick(250);

    // Create reader
    let mut reader = ReaderWithProgress {
        inner: response.into_reader(),
        progress_bar: progress_bar.clone(),
    };

    // Write to file
    let mut file = tempfile()?;
    io::copy(&mut reader, &mut file)?;

    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.blue}   {msg}")
            .tick_strings(&["∙∙∙", "●∙∙", "∙●∙", "∙∙●", "∙∙∙"]),
    );
    progress_bar.finish_with_message("Done - Downloading new version");
    println!();

    Ok(file)
}

struct ReaderWithProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for ReaderWithProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}
