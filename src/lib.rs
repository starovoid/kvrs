pub mod error;

use crate::error::{DataFormatError, StorageError};
use byteorder::{BigEndian, ReadBytesExt};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use std::{mem, vec};

type Index = IndexMap<Vec<u8>, u64>;

/// The first 8 bytes of the file indicating working with the correct format.
/// This number represents the following set of bytes: `c4 b7 d1 b5 c5 97 c5 a1`.
/// Which in UTF-8 format is equivalent to: `ķѵŗš`.
const IDENTIFIER: u64 = 14175028930806269345;

/// The oldest version of the data file format.
const OLDEST_VERSION: u8 = 1;

#[derive(Debug)]
pub struct Storage<T> {
    inner: T,
    index: IndexMap<Vec<u8>, u64>,
    vacant_blocks: Vec<VacantBlock>,
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
        Ok(Self {
            inner: file,
            index,
            vacant_blocks: vec![],
        })
    }
}

impl Storage<Cursor<Vec<u8>>> {
    pub fn from_vec(buf: Vec<u8>) -> Result<Self, StorageError> {
        let mut data = Cursor::new(buf);
        Storage::check_prefix(&mut data)?;
        let index = Storage::load_index(&mut data)?;
        Ok(Self {
            inner: data,
            index,
            vacant_blocks: vec![],
        })
    }
}

impl<T: Read + Seek> Storage<T> {
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

    fn serialize(&self) -> Result<Vec<u8>, StorageError> {
        let identifier =
            postcard::to_allocvec(&IDENTIFIER).map_err(|_| StorageError::SerializationError)?;
        // todo: Change to current version?
        let version =
            postcard::to_allocvec(&OLDEST_VERSION).map_err(|_| StorageError::SerializationError)?;
        let index_position =
            postcard::to_allocvec(&self.index).map_err(|_| StorageError::SerializationError)?;

        let vacant_blocks_size = mem::size_of::<VacantBlock>() * self.vacant_blocks.len();
        let mut vacant_blocks = Vec::with_capacity(vacant_blocks_size);
        for vacant_block in self.vacant_blocks.iter() {
            vacant_blocks.extend(vacant_block.serialize()?);
        }

        let bytes = [identifier, version, index_position, vacant_blocks].concat();
        Ok(bytes)
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

    pub fn serialize(&self) -> Result<Vec<u8>, StorageError> {
        postcard::to_allocvec(self).map_err(|_| StorageError::SerializationError)
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
}
