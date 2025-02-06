use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::Path,
};

use futures_util::StreamExt;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use reqwest::Client;

use super::ensure_parent_folder_exists;

pub async fn download_file(
    source_url: &str,
    destination_path: &Path,
    client: Option<Client>,
) -> Result<String, String> {
    ensure_parent_folder_exists(destination_path).map_err(|why| {
        format!(
            "Failed to create parent folder for {}: {}",
            destination_path.display(),
            why
        )
    })?;
    let mut file = match OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination_path)
    {
        Err(why) => match why.kind() {
            ErrorKind::AlreadyExists => return Ok("already exists".to_string()),
            _ => {
                return Err(format!(
                    "Failed to create output file {}: {}",
                    destination_path.display(),
                    why
                ))
            }
        },
        Ok(f) => f,
    };

    let client = client.unwrap_or_default();
    let response = client
        .get(source_url)
        .send()
        .await
        .map_err(|why| format!("Failed to download from {}: {}", source_url, why))?;

    let size = response.content_length();

    let progress = match size {
        Some(len) => ProgressBar::new(len),
        None => ProgressBar::no_length(),
    };
    progress.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}, {eta}",
        )
        .expect("Static template string should be ok."),
    );

    let mut done = 0;

    let mut bytes_stream = response.bytes_stream();
    while let Some(stream_part) = bytes_stream.next().await {
        let chunk =
            stream_part.map_err(|why| format!("Error downloading {}: {}", source_url, why))?;
        file.write_all(&chunk)
            .map_err(|why| format!("Error writing to {}: {}", destination_path.display(), why))?;
        done += chunk.len() as u64;
        progress.set_position(done);
    }

    progress.finish_and_clear();
    Ok(HumanBytes(done).to_string())
}
