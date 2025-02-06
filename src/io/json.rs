use std::{
    fs::OpenOptions,
    io::{BufWriter, Read},
    path::Path,
};

use serde::{de::DeserializeOwned, Serialize};

use super::{ensure_parent_folder_exists, IOError};

/// Serialize the given object to a json string an write that to the given file.
pub fn write_json_to_file<T: Serialize>(file_path: &Path, object: T) -> Result<(), IOError> {
    ensure_parent_folder_exists(file_path)?;

    let display: String = file_path.display().to_string();

    // open file for writing
    let file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
    {
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
