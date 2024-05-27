use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufWriter, ErrorKind, Read};
use std::path::Path;

#[derive(Debug)]
pub enum IOError {
    Unknown,
    CannotWrite(String, String),
    CannotSerialize(String, String),
    CannotRead(String, String),
    CannotDeserialize(String, String),
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = match self {
            IOError::Unknown => String::from("Unknown error"),
            IOError::CannotWrite(what, why) => format!("Couldn't write to {}: {}", what, why),
            IOError::CannotSerialize(what, why) => {
                format!("Couldn't serialize to {}: {}", what, why)
            }
            IOError::CannotRead(what, why) => format!("Couldn't read from {}: {}", what, why),
            IOError::CannotDeserialize(what, why) => {
                format!("Couldn't deserialize from {}: {}", what, why)
            }
        };

        write!(f, "{err}",)
    }
}
impl std::error::Error for IOError {}

/// Serialize the given object to a json string an write that to the given file.
pub fn write_json_to_file<T: Serialize>(file_path: &Path, object: T) -> Result<(), IOError> {
    ensure_parent_folder_exists(file_path)?;

    let display: String = file_path.display().to_string();

    // open file for writing
    let file = match OpenOptions::new().write(true).create(true).open(file_path) {
        Err(why) => return Result::Err(IOError::CannotWrite(display, why.to_string())),
        Ok(file) => file,
    };

    // serialize object to file
    match serde_json::to_writer(BufWriter::new(file), &object) {
        Err(why) => Result::Err(IOError::CannotSerialize(display, why.to_string())),
        Ok(_) => Ok(()),
    }
}

pub fn read_json_from_file<T: DeserializeOwned>(file_path: &Path) -> Result<T, IOError> {
    let display: String = file_path.display().to_string();

    // open file for reading
    let mut file = match OpenOptions::new().read(true).open(file_path) {
        Err(why) => return Result::Err(IOError::CannotRead(display, why.to_string())),
        Ok(file) => file,
    };

    let mut file_content = String::new();
    if let Err(why) = file.read_to_string(&mut file_content) {
        return Result::Err(IOError::CannotRead(display, why.to_string()));
    }

    // load object from file
    match serde_json::from_str(&file_content) {
        Err(why) => Result::Err(IOError::CannotDeserialize(display, why.to_string())),
        Ok(o) => Ok(o),
    }
}

pub fn append_csv_to_file<T: Serialize>(file_path: &Path, objects: &[T]) -> Result<(), IOError> {
    ensure_parent_folder_exists(file_path)?;

    let display: String = file_path.display().to_string();

    let mut writer;

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
    {
        Ok(file) => {
            // write to new file, including headers
            writer = csv::WriterBuilder::new()
                .has_headers(true)
                .from_writer(file);
        }
        Err(why) => match why.kind() {
            ErrorKind::AlreadyExists => {
                match OpenOptions::new().append(true).create(true).open(file_path) {
                    Ok(file) => {
                        // append to existing file that already has headers
                        writer = csv::WriterBuilder::new()
                            .has_headers(false)
                            .from_writer(file);
                    }
                    Err(why) => return Result::Err(IOError::CannotWrite(display, why.to_string())),
                };
            }
            _ => {
                return Result::Err(IOError::CannotWrite(display, why.to_string()));
            }
        },
    }
    for object in objects {
        writer
            .serialize(object)
            .map_err(|why| IOError::CannotSerialize(display.clone(), why.to_string()))?
    }
    Ok(())
}

pub fn read_csv_from_file<T: DeserializeOwned>(file_path: &Path) -> Result<Vec<T>, IOError> {
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

fn ensure_parent_folder_exists(file_path: &Path) -> Result<(), IOError> {
    let display: String = file_path.display().to_string();

    let parent = match file_path.parent() {
        None => return Result::Err(IOError::CannotWrite(display, String::from("Not a file."))),
        Some(p) => p,
    };
    // ensure folder exists
    if let Err(why) = create_dir_all(parent) {
        Result::Err(IOError::CannotWrite(display, why.to_string()))
    } else {
        Ok(())
    }
}
