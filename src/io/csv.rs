use std::{fs::OpenOptions, io::ErrorKind, path::Path};

use serde::{de::DeserializeOwned, Serialize};

use super::{ensure_parent_folder_exists, IOError};

pub enum WriteMode {
    Append,
    Replace,
}

pub enum HeaderMode {
    Auto,
    None,
}

pub fn write_to_file<T: Serialize>(
    file_path: impl AsRef<Path>,
    objects: &[T],
    mode: WriteMode,
    headers: HeaderMode,
) -> Result<(), IOError> {
    let file_path = file_path.as_ref();
    ensure_parent_folder_exists(file_path)?;

    let display: String = file_path.display().to_string();

    let mut writer = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
    {
        Ok(file) => {
            // write to new file, maybe including headers
            csv::WriterBuilder::new()
                .has_headers(matches!(headers, HeaderMode::Auto))
                .from_writer(file)
        }
        Err(why) => match why.kind() {
            ErrorKind::AlreadyExists => {
                match OpenOptions::new()
                    .write(true)
                    .append(matches!(mode, WriteMode::Append))
                    .truncate(matches!(mode, WriteMode::Replace))
                    .open(file_path)
                {
                    Ok(file) => {
                        // Headers should be written if we're truncating and auto header mode is on.
                        // Otherwise we're appending to a file that already has headers.
                        csv::WriterBuilder::new()
                            .has_headers(
                                matches!(mode, WriteMode::Replace)
                                    && matches!(headers, HeaderMode::Auto),
                            )
                            .from_writer(file)
                    }
                    Err(why) => return Result::Err(IOError::CannotWrite(display, why.to_string())),
                }
            }
            _ => {
                return Result::Err(IOError::CannotWrite(display, why.to_string()));
            }
        },
    };

    for object in objects {
        writer
            .serialize(object)
            .map_err(|why| IOError::CannotSerialize(display.clone(), why.to_string()))?
    }
    Ok(())
}

pub fn append_to_file<T: Serialize>(
    file_path: impl AsRef<Path>,
    objects: &[T],
) -> Result<(), IOError> {
    write_to_file(file_path, objects, WriteMode::Append, HeaderMode::Auto)
}

pub fn read_from_file<T: DeserializeOwned>(file_path: impl AsRef<Path>) -> Result<Vec<T>, IOError> {
    let file_path = file_path.as_ref();
    let display: String = file_path.display().to_string();

    // open file for reading
    let file = match OpenOptions::new().read(true).open(file_path) {
        Err(why) => return Result::Err(IOError::CannotRead(display, why.to_string())),
        Ok(file) => file,
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    Ok(reader.deserialize::<T>().filter_map(Result::ok).collect())
}
