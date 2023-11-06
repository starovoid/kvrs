use std::{fmt, io};

/// Top-level type of library error.
#[derive(Debug, PartialEq)]
pub enum StorageError {
    /// I/O error with kinds from `std::io`.`
    IO(io::ErrorKind),

    /// Wrong data format.
    DataFormat(DataFormatError),

    /// Failed to load index.
    FailedLoadIndex,

    /// Failed to serialize something.
    SerializationError,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            Self::DataFormat(e) => write!(f, "Data format error: {e}"),
            Self::FailedLoadIndex => write!(f, "Failed to load index"),
            Self::SerializationError => write!(f, "Failed to serialize something"),
        }
    }
}

#[derive(Debug, PartialEq)]
// Possible data format errors.
pub enum DataFormatError {
    /// Missing identifier of the bytes `c4 b7 d1 b5 c5 97 c5 a1` (the first 8 bytes of data).
    MissedIdentifier,

    /// Incorrect version number is specified (byte with index 8 from the beginning of the data).
    IncorrectVersion(u8),
}

impl fmt::Display for DataFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::OLDEST_VERSION;

        match self {
            Self::MissedIdentifier => write!(f, "Missing identifier at the beginning of the data file."),
            Self::IncorrectVersion(n) => write!(
                f, "Incorrect version number of the data file format: {}. The older version has the number {}",
                n, OLDEST_VERSION
            ),
        }
    }
}
