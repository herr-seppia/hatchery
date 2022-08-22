// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::snapshot::{
    ModuleId, ModuleSnapshotBag,
};
use bytecheck::CheckBytes;
use rkyv::{
    ser::serializers::{BufferScratch, CompositeSerializer, WriteSerializer},
    ser::Serializer,
    Archive, Deserialize, Serialize,
};
use std::collections::btree_map::BTreeMap;

use crate::error::Error;
use crate::Error::PersistenceError;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

const MODULE_SNAPSHOT_DATA_SCRATCH_SIZE: usize = 64;

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct ModuleSnapshotData {
    module_snapshot_bags: BTreeMap<ModuleId, ModuleSnapshotBag>,
}

impl ModuleSnapshotData {
    pub fn new() -> Self {
        ModuleSnapshotData {
            module_snapshot_bags: BTreeMap::new(),
        }
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file =
            std::fs::File::open(path.as_ref()).map_err(PersistenceError)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(PersistenceError)?;
        let archived = rkyv::check_archived_root::<Self>(&data[..]).unwrap();
        let module_snapshot_data: Self =
            archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(module_snapshot_data)
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(PersistenceError)?;

        let mut scratch_buf = [0u8; MODULE_SNAPSHOT_DATA_SCRATCH_SIZE];
        let scratch = BufferScratch::new(&mut scratch_buf);
        let serializer = WriteSerializer::new(file);
        let mut composite =
            CompositeSerializer::new(serializer, scratch, rkyv::Infallible);

        composite.serialize_value(self).unwrap();
        Ok(())
    }
}
