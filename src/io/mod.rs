pub mod csv;
pub mod download;
pub mod json;

use std::fmt;
use std::fs::create_dir_all;
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
