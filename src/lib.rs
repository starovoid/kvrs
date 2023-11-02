use byteorder::{BigEndian, ReadBytesExt};
use indexmap::IndexMap;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Cursor, Read};
use std::path::Path;

/// The first 8 bytes of the file indicating working with the correct format.
/// This number represents the following set of bytes: `c4 b7 d1 b5 c5 97 c5 a1`.
/// Which in UTF-8 format is equivalent to: `ķѵŗš`.
const IDENTIFIER: u64 = 14175028930806269345;

/// The oldest version of the data file format.
const OLDEST_VERSION: u8 = 1;

/// Top-level type of library error.
#[derive(Debug, PartialEq)]
pub enum StorageError {
    /// I/O error with kinds from `std::io`.`
    IO(io::ErrorKind),

    /// Wrong data format.
    DataFormat(DataFormatError),

    /// Unknown operation.
    UnknownOperation(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            Self::DataFormat(e) => write!(f, "Data format error: {e}"),
            Self::UnknownOperation(command) => write!(f, "Operation '{command}' not found"),
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
        match self {
            Self::MissedIdentifier => write!(f, "Missing identifier at the beginning of the data file."),
            Self::IncorrectVersion(n) => write!(f, "Incorrect version number of the data file format: {n}. The older version has the number {OLDEST_VERSION}"),
        }
    }
}

#[derive(Debug)]
pub struct Storage<T> {
    inner: T,
    index: IndexMap<Vec<u8>, u64>,
}

impl Storage<File> {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| StorageError::IO(e.kind()))?;

        Storage::check_prefix(&mut file)?;
        let index = Storage::load_index(&mut file)?;
        Ok(Self { inner: file, index })
    }
}

impl Storage<Cursor<Vec<u8>>> {
    pub fn from_vec(buf: Vec<u8>) -> Result<Self, StorageError> {
        let mut data = Cursor::new(buf);
        Storage::check_prefix(&mut data)?;
        let index = Storage::load_index(&mut data)?;
        Ok(Self { inner: data, index })
    }
}

impl<T: Read> Storage<T> {
    fn check_prefix(data: &mut T) -> Result<(), StorageError> {
        let ind = data
            .read_u64::<BigEndian>()
            .map_err(|e| StorageError::IO(e.kind()))?;

        if ind != IDENTIFIER {
            return Err(StorageError::DataFormat(DataFormatError::MissedIdentifier));
        }

        let version = data.read_u8().map_err(|e| StorageError::IO(e.kind()))?;
        if version > OLDEST_VERSION {
            return Err(StorageError::DataFormat(DataFormatError::IncorrectVersion(
                version,
            )));
        }

        Ok(())
    }

    fn load_index(_data: &mut T) -> Result<IndexMap<Vec<u8>, u64>, StorageError> {
        // STUB
        Ok(IndexMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_prefix() {
        // Just random bytes
        let tail_data: Vec<u8> = vec![73, 42, 255, 0, 0, 123, 64, 90, 17, 48, 33];

        let mut correct: Vec<u8> = vec![0xc4, 0xb7, 0xd1, 0xb5, 0xc5, 0x97, 0xc5, 0xa1, 1];
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(correct.clone())),
            Ok(())
        );
        correct.append(&mut tail_data.clone());
        assert_eq!(Storage::check_prefix(&mut Cursor::new(correct)), Ok(()));

        let mut wrong_version: Vec<u8> = vec![0xc4, 0xb7, 0xd1, 0xb5, 0xc5, 0x97, 0xc5, 0xa1, 123];
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_version.clone())),
            Err(StorageError::DataFormat(DataFormatError::IncorrectVersion(
                123
            )))
        );
        wrong_version.append(&mut tail_data.clone());
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_version)),
            Err(StorageError::DataFormat(DataFormatError::IncorrectVersion(
                123
            )))
        );

        let mut wrong_ident: Vec<u8> = vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 1];
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_ident.clone())),
            Err(StorageError::DataFormat(DataFormatError::MissedIdentifier))
        );
        wrong_ident.append(&mut tail_data.clone());
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_ident)),
            Err(StorageError::DataFormat(DataFormatError::MissedIdentifier))
        );

        let mut wrong_ident_and_version: Vec<u8> =
            vec![0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 1];
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_ident_and_version.clone())),
            Err(StorageError::DataFormat(DataFormatError::MissedIdentifier))
        );
        wrong_ident_and_version.append(&mut tail_data.clone());
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(wrong_ident_and_version)),
            Err(StorageError::DataFormat(DataFormatError::MissedIdentifier))
        );
    }
}
