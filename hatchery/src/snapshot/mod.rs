// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod diff_data;
mod module_snapshot;
mod module_snapshot_bag;
mod module_snapshot_data;
mod snapshot_data;

pub use module_snapshot::{MemoryPath, ModuleSnapshot, ModuleSnapshotId};
pub use module_snapshot_bag::ModuleSnapshotBag;
pub use snapshot_data::SnapshotData;

use core::cmp::Ordering;
use std::collections::BTreeMap;

use dallo::ModuleId;

use crate::error::Error;
use crate::instance::Instance;
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

pub const SNAPSHOT_ID_BYTES: usize = 32;
/// Snapshot of the world encompassing states of all world's modules.
#[derive(
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
#[archive_attr(derive(CheckBytes))]
#[repr(C)]
pub struct SnapshotId([u8; SNAPSHOT_ID_BYTES]);
impl SnapshotId {
    pub const fn uninitialized() -> Self {
        SnapshotId([0u8; SNAPSHOT_ID_BYTES])
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn add(&mut self, module_snapshot_id: &ModuleSnapshotId) {
        let p = module_snapshot_id.as_bytes().as_ptr();
        for (i, b) in self.0.iter_mut().enumerate() {
            *b ^= unsafe { *p.add(i) };
        }
    }
}

impl From<[u8; 32]> for SnapshotId {
    fn from(array: [u8; 32]) -> Self {
        SnapshotId(array)
    }
}

impl core::fmt::Debug for SnapshotId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "0x")?
        }
        for byte in self.0 {
            write!(f, "{:02x}", &byte)?
        }
        Ok(())
    }
}

impl Ord for ArchivedSnapshotId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for ArchivedSnapshotId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
    fn lt(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}

impl PartialEq for ArchivedSnapshotId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ArchivedSnapshotId {}

impl core::fmt::Debug for ArchivedSnapshotId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "0x")?
        }
        for byte in self.0 {
            write!(f, "{:02x}", &byte)?
        }
        Ok(())
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Snapshot {
    id: SnapshotId,
    module_snapshot_indices: BTreeMap<ModuleId, usize>,
}

impl Snapshot {
    pub(crate) fn new() -> Self {
        Self {
            id: SnapshotId::uninitialized(),
            module_snapshot_indices: BTreeMap::new(),
        }
    }

    pub(crate) fn persist_module_snapshot(
        &mut self,
        memory_path: &MemoryPath,
        instance: &mut Instance,
        module_id: &ModuleId,
    ) -> Result<(), Error> {
        let module_snapshot = ModuleSnapshot::new(
            memory_path,
            instance.arg_buffer_span(),
            instance.heap_base(),
        )?;
        let module_snapshot_index = instance
            .module_snapshot_bag_mut()
            .save_module_snapshot(&module_snapshot, memory_path)?;
        self.id.add(&module_snapshot.id());
        self.module_snapshot_indices
            .insert(*module_id, module_snapshot_index);
        Ok(())
    }

    pub(crate) fn restore_module_snapshots<'a, F1, F2>(
        &self,
        get_memory_path: F1,
        get_instance: F2,
    ) -> Result<(), Error>
    where
        F1: Fn(ModuleId) -> MemoryPath,
        F2: Fn(ModuleId) -> &'a Instance,
    {
        for (module_id, module_snapshot_index) in
            self.module_snapshot_indices.iter()
        {
            let memory_path = get_memory_path(*module_id);
            get_instance(*module_id)
                .module_snapshot_bag()
                .restore_module_snapshot(
                    *module_snapshot_index,
                    &memory_path,
                )?;
        }
        Ok(())
    }

    pub fn id(&self) -> SnapshotId {
        self.id
    }
}
