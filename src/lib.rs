use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use indexmap::IndexMap;

use serde::{Deserialize, Serialize};

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

type Index = IndexMap<Vec<u8>, u64>;

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

    /// Failed to load index.
    FailedLoadIndex,

    /// Faild to save index.
    FailedSaveIndex,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            Self::DataFormat(e) => write!(f, "Data format error: {e}"),
            Self::FailedLoadIndex => write!(f, "Failed to load index"),
            Self::FailedSaveIndex => write!(f, "Failed to save index"),
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

#[derive(Debug, PartialEq)]
pub struct Storage<T> {
    inner: T,
    index: IndexMap<Vec<u8>, u64>,
    version: u8,
}

impl Storage<File> {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| StorageError::IO(e.kind()))?;

        let version = Storage::check_prefix(&mut file)?;
        let index = Storage::load_index(&mut file)?;
        Ok(Self {
            inner: file,
            index,
            version,
        })
    }

    /// Creating a new data file.
    pub fn new_file(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let file = File::create(path).map_err(|e| StorageError::IO(e.kind()))?;
        let mut st = Storage {
            inner: file,
            index: IndexMap::new(),
            version: OLDEST_VERSION,
        };
        st.initialize()?;
        Ok(st)
    }
}

impl Storage<Cursor<Vec<u8>>> {
    pub fn from_vec(buf: Vec<u8>) -> Result<Self, StorageError> {
        let mut data = Cursor::new(buf);
        let version = Storage::check_prefix(&mut data)?;
        let index = Storage::load_index(&mut data)?;
        Ok(Self {
            inner: data,
            index,
            version,
        })
    }

    pub fn new_vectored() -> Result<Self, StorageError> {
        let mut st = Storage {
            inner: Cursor::new(Vec::new()),
            index: IndexMap::new(),
            version: OLDEST_VERSION,
        };
        st.initialize()?;
        Ok(st)
    }
}

impl<T: Read> Storage<T> {
    fn check_prefix(data: &mut T) -> Result<u8, StorageError> {
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

        Ok(version)
    }
}

impl<T: Read + Seek> Storage<T> {
    /// Load index from the data stream.
    fn load_index(data: &mut T) -> Result<Index, StorageError> {
        data.seek(SeekFrom::Start(9))
            .map_err(|e| StorageError::IO(e.kind()))?;

        let index_pos = data
            .read_u64::<BigEndian>()
            .map_err(|e| StorageError::IO(e.kind()))?;

        data.seek(SeekFrom::Start(index_pos))
            .map_err(|e| StorageError::IO(e.kind()))?;

        let index_len = data
            .read_u64::<BigEndian>()
            .map_err(|e| StorageError::IO(e.kind()))?;

        let mut buf: Vec<u8> = vec![0; index_len as usize];
        data.read_exact(&mut buf)
            .map_err(|e| StorageError::IO(e.kind()))?;

        let index = postcard::from_bytes(&buf).map_err(|_| StorageError::FailedLoadIndex)?;
        Ok(index)
    }
}

impl<T: Write + Seek> Storage<T> {
    /// Storage (database) initialization.
    fn initialize(&mut self) -> Result<(), StorageError> {
        let ser_index =
            postcard::to_allocvec(&self.index).map_err(|_| StorageError::FailedSaveIndex)?;

        let ser_vb_list = postcard::to_allocvec(&Vec::<VacantBlock>::new())
            .map_err(|_| StorageError::FailedSaveIndex)?;

        // Identifier
        self.inner
            .write_u64::<BigEndian>(IDENTIFIER)
            .map_err(|e| StorageError::IO(e.kind()))?;

        // Version
        self.inner
            .write(&[self.version])
            .map_err(|e| StorageError::IO(e.kind()))?;

        // Index position
        self.inner
            .write_u64::<BigEndian>(25)
            .map_err(|e| StorageError::IO(e.kind()))?;

        // Vacant blocks list position
        self.inner
            .write_u64::<BigEndian>(33 + ser_index.len() as u64)
            .map_err(|e| StorageError::IO(e.kind()))?;

        // Index size
        self.inner
            .write_u64::<BigEndian>(ser_index.len() as u64)
            .map_err(|e| StorageError::IO(e.kind()))?;
        // Index
        self.inner
            .write_all(&ser_index)
            .map_err(|e| StorageError::IO(e.kind()))?;

        // Vacant blocks list size
        self.inner
            .write_u64::<BigEndian>(ser_vb_list.len() as u64)
            .map_err(|e| StorageError::IO(e.kind()))?;
        // Vacant blocks list
        self.inner
            .write_all(&ser_vb_list)
            .map_err(|e| StorageError::IO(e.kind()))?;

        self.inner.flush().map_err(|e| StorageError::IO(e.kind()))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VacantBlock {
    pos: u64,
    size: u64,
}

impl VacantBlock {
    pub fn new(pos: u64, size: u64) -> Self {
        Self { pos, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_check_prefix() {
        // Just random bytes
        let tail_data: Vec<u8> = vec![73, 42, 255, 0, 0, 123, 64, 90, 17, 48, 33];

        let mut correct: Vec<u8> = vec![0xc4, 0xb7, 0xd1, 0xb5, 0xc5, 0x97, 0xc5, 0xa1, 1];
        assert_eq!(
            Storage::check_prefix(&mut Cursor::new(correct.clone())),
            Ok(1)
        );
        correct.append(&mut tail_data.clone());
        assert_eq!(Storage::check_prefix(&mut Cursor::new(correct)), Ok(1));

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

    #[test]
    fn test_load_index() {
        // Just random bytes
        let tail_data: Vec<u8> = vec![
            73, 80, 42, 255, 0, 0, 123, 84, 19, 1, 2, 64, 90, 17, 48, 55, 33,
        ];

        let do_test = |index: Index| {
            let ser_ind = postcard::to_allocvec(&index).unwrap();

            let mut data: Vec<u8> = vec![0xc4, 0xb7, 0xd1, 0xb5, 0xc5, 0x97, 0xc5, 0xa1, 1];
            data.extend(&[0, 0, 0, 0, 0, 0, 0, 17]); // Index position
            data.extend((ser_ind.len() as u64).to_be_bytes());
            data.append(&mut ser_ind.clone());

            assert_eq!(
                Storage::load_index(&mut Cursor::new(data.clone())),
                Ok(index.clone()),
            );

            data.extend(&tail_data);
            assert_eq!(Storage::load_index(&mut Cursor::new(data)), Ok(index),);
        };

        // Empty index
        do_test(Index::new());
        // Index length = 1
        do_test(Index::from([(vec![1, 2, 3], 123)]));
        // Index length = 2
        do_test(Index::from([(vec![1, 2, 3], 123), (vec![10], 20)]));

        // Random tests
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let len = rng.gen_range(1..200usize);

            let mut ind = Index::with_capacity(len);
            for _ in 0..len {
                let key_len = rng.gen_range(1..300);
                let key = (0..key_len).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();
                let value_pos = rng.gen::<u64>();
                ind.insert(key, value_pos);
            }

            do_test(ind);
        }
    }

    #[test]
    fn test_initialize() {
        let mut st = Storage::new_vectored().unwrap();

        assert_eq!(
            st.inner.get_ref(),
            &[
                196, 183, 209, 181, 197, 151, 197, 161, 1, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0,
                0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0
            ]
        );

        let mut left = Storage::from_vec(st.inner.get_ref().clone()).unwrap();
        left.inner.seek(SeekFrom::Start(0)).unwrap();
        st.inner.seek(SeekFrom::Start(0)).unwrap();

        assert_eq!(left, st);
    }
}
