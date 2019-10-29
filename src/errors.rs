use std::error::Error;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
    PathNotRelative,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::PathNotRelative => write!(f, "Input path to store is not relative"),
        }
    }
}

impl Error for StorageError {
    fn description(&self) -> &str {
        "Some error happened when using PathStorage"
    }
}
