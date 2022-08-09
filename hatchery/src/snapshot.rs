// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::storage_helpers::{
    combine_module_snapshot_names, snapshot_id_to_name,
};
use crate::Error::PersistenceError;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::OpenOptions;
use std::mem;
use bsdiff::diff::diff;
use bsdiff::patch::patch;

use rkyv::{Archive, Deserialize, Serialize};

const COMPRESSION_LEVEL: i32 = 11;

pub const SNAPSHOT_ID_BYTES: usize = 32;
#[derive(
    Debug,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
)]
pub struct SnapshotId([u8; SNAPSHOT_ID_BYTES]);
impl SnapshotId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl From<[u8; 32]> for SnapshotId {
    fn from(array: [u8; 32]) -> Self {
        SnapshotId(array)
    }
}

pub trait SnapshotLike {
    fn path(&self) -> &PathBuf;
    /// Read's snapshot's content into buffer
    fn read(&self) -> Result<Vec<u8>, Error> {
        let mut f = std::fs::File::open(self.path().as_path())
            .map_err(PersistenceError)?;
        let metadata = std::fs::metadata(self.path().as_path())
            .map_err(PersistenceError)?;
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(buffer.as_mut_slice()).map_err(PersistenceError)?;
        Ok(buffer)
    }
}

pub struct MemoryPath {
    path: PathBuf,
}

impl MemoryPath {
    pub fn new(path: impl AsRef<Path>) -> Self {
        MemoryPath {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl SnapshotLike for MemoryPath {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub struct Snapshot {
    path: PathBuf,
    id: SnapshotId,
}

impl Snapshot {
    pub fn new(memory_path: &MemoryPath) -> Result<Self, Error> {
        let snapshot_id: SnapshotId = SnapshotId::from(
            *blake3::hash(memory_path.read()?.as_slice()).as_bytes(),
        );
        Snapshot::from_id(snapshot_id, memory_path)
    }

    pub fn from_id(
        snapshot_id: SnapshotId,
        memory_path: &MemoryPath,
    ) -> Result<Self, Error> {
        let mut path = memory_path.path().to_owned();
        path.set_file_name(combine_module_snapshot_names(
            path.file_name()
                .expect("filename exists")
                .to_str()
                .expect("filename is UTF8"),
            snapshot_id_to_name(snapshot_id),
        ));
        Ok(Snapshot {
            path,
            id: snapshot_id,
        })
    }

    /// Saves current snapshot as uncompressed file.
    pub fn save(&self, memory_path: &MemoryPath, snapshot_ids: &Vec<SnapshotId>) -> Result<(), Error> {
        if snapshot_ids.len() == 1 {
            println!("saving uncompressed {}", snapshot_id_to_name(self.id));
            std::fs::copy(memory_path.path(), self.path().as_path())
                .map_err(PersistenceError)?;
        }
        else if snapshot_ids.len() == 2 {
            let base_snapshot = Snapshot::from_id(*snapshot_ids.get(0).unwrap(), memory_path)?;
            println!("saving compressed {}", snapshot_id_to_name(self.id));
            self.save_compressed(&base_snapshot, memory_path);
        } else {
            unreachable!("case not implemented: snapshot_ids.len() == 0 or more than 2")// todo
        }
        Ok(())
    }

    /// Saves current snapshot as compressed file.
    fn save_compressed(
        &self,
        base_snapshot: &Snapshot,
        memory_path: &MemoryPath,
    ) -> Result<(), Error> {
        let mut compressor = zstd::block::Compressor::new();
        let mut delta: Vec<u8> = Vec::new();
        let memory_buffer = memory_path.read()?;
        let base_buffer = base_snapshot.read()?;
        diff(base_buffer.as_slice(), memory_buffer.as_slice(), &mut delta)
            .unwrap();
        let compressed_delta =
            compressor.compress(&delta, COMPRESSION_LEVEL).unwrap();
        self.write_compressed(
            compressed_delta,
            delta.len(),
            base_buffer.as_slice().len(),
        )?;
        Ok(())
    }

    /// Writes compressed data to snapshot's file.
    fn write_compressed(
        &self,
        buf: Vec<u8>,
        uncompressed_size: usize,
        original_len: usize,
    ) -> Result<(), Error> {
        println!("writing compressed {} bytes to {:?}", buf.len(), self.path());
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path())
            .map_err(PersistenceError)?;
        file.write_u32::<LittleEndian>(uncompressed_size as u32)
            .map_err(PersistenceError)?;
        file.write_u32::<LittleEndian>(original_len as u32)
            .map_err(PersistenceError)?;
        file.write_all(buf.as_slice()).map_err(PersistenceError)?;
        Ok(())
    }

    /// Restores current snapshot from uncompressed file.
    pub fn load(&self, memory_path: &MemoryPath, snapshot_ids: &[SnapshotId]) -> Result<(), Error> {
        if snapshot_ids.len() == 1 {
            println!("load decompressed from {:?}", self.path());
            std::fs::copy(self.path().as_path(), memory_path.path())
                .map_err(PersistenceError)?;
            Ok(())
        } else if snapshot_ids.len() == 2 {
            let base_snapshot = Snapshot::from_id(*snapshot_ids.get(0).unwrap(), memory_path)?;
            let compressed_snapshot = Snapshot::from_id(*snapshot_ids.get(1).unwrap(), memory_path)?;
            println!("load compressed from {}", snapshot_id_to_name(compressed_snapshot.id));
            compressed_snapshot.decompress(&base_snapshot, memory_path)?;
            Ok(())
        } else if snapshot_ids.len() == 0 {
            unreachable!("todo0")// todo 0
        } else {
            unreachable!("todoN>=2")// todo N>2
        }
    }

    /// Decompresses current snapshot as patch and patches with it a given
    /// snapshot. The result will be returned in a new snapshot
    /// equivalent to given memory path.
    pub fn decompress(
        &self,
        snapshot_to_patch: &Snapshot,
        memory_path: &MemoryPath,
    ) -> Result<(), Error> {
        let (original_len, uncompressed_size, compressed) =
            self.read_compressed()?;
        let mut decompressor = zstd::block::Decompressor::new();
        let mut patch_data = std::io::Cursor::new(
            decompressor
                .decompress(compressed.as_slice(), original_len)
                .map_err(PersistenceError)?,
        );
        let mut patched = vec![0; uncompressed_size];
        patch(
            snapshot_to_patch.read()?.as_slice(),
            &mut patch_data,
            patched.as_mut_slice(),
        )
            .map_err(PersistenceError)?;

        // copy contents of 'patched' to memory_path.path()
        let mut file = OpenOptions::new() // todo refactor into a method
            .write(true)
            .create(true)
            .truncate(true)
            .open(memory_path.path())
            .map_err(PersistenceError)?;
        file.write_all(patched.as_slice()).map_err(PersistenceError)?;

        Ok(())
    }

    /// Reads compressed data from snapshot's file.
    fn read_compressed(&self) -> Result<(usize, usize, Vec<u8>), Error> {
        let mut f = std::fs::File::open(self.path().as_path())
            .map_err(PersistenceError)?;
        let metadata = std::fs::metadata(self.path().as_path())
            .map_err(PersistenceError)?;
        const SIZES_LEN: usize = (mem::size_of::<u32>() as usize) * 2;
        let mut buffer = vec![0; metadata.len() as usize - SIZES_LEN];
        let size = f.read_u32::<LittleEndian>().map_err(PersistenceError)?;
        let original_len =
            f.read_u32::<LittleEndian>().map_err(PersistenceError)?;
        f.read(buffer.as_mut_slice()).map_err(PersistenceError)?;
        Ok((size as usize, original_len as usize, buffer))
    }

    /// Writes buffer to file at snapshot's path.
    fn write(&self, buf: Vec<u8>) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path())
            .map_err(PersistenceError)?;
        file.write_all(buf.as_slice()).map_err(PersistenceError)?;
        Ok(())
    }

    pub fn id(&self) -> SnapshotId {
        self.id
    }
}

impl SnapshotLike for Snapshot {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}
