// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::snapshot::{MemoryPath, SnapshotId};
use std::path::{Path, PathBuf};
use crate::Error::PersistenceError;
use crate::error::Error;
use crate::snapshot::Snapshot;

#[derive(Debug)]
pub struct SnapshotBag {
    ids: Vec<SnapshotId>,
    top: SnapshotId,
    accu: SnapshotId,
}

impl SnapshotBag {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            top: SnapshotId::random(),
            accu: SnapshotId::random(),
        }
    }
    pub fn save_snapshot(&mut self, snapshot: &Snapshot, memory_path: &MemoryPath) -> Result<usize, Error> {
        self.ids.push(snapshot.id());
        if self.ids.len() == 1 {
            snapshot.save_from_snapshot(memory_path)?;
            Ok(0)
        } else {
            let top_snapshot = Snapshot::from_id(self.top, memory_path)?;
            let accu_snapshot = Snapshot::from_id(self.accu, memory_path)?;
            accu_snapshot.save_from_snapshot(snapshot);
            snapshot.save_compressed(&top_snapshot, memory_path)?;// now snapshot is compressed but accu keeps the uncompressed copy
            top_snapshot.save_from_snapshot(&accu_snapshot)?; // top is always the last uncompressed
            Ok(self.ids.len() - 1)
        }
    }
    pub fn restore_snapshot(&self, snapshot_index: usize, memory_path: &MemoryPath) -> Result<(), Error> {
        if snapshot_index >= self.ids.len(){
            return Ok(())
        }
        if self.ids.len() == 1 {
            let base_snapshot = Snapshot::from_id(*self.ids.get(0).unwrap(), memory_path)?;
            base_snapshot.restore_this(memory_path)
        } else if self.ids.len() == (snapshot_index + 1) {
            let top_snapshot = Snapshot::from_id(self.top, memory_path)?;
            top_snapshot.restore_this(memory_path)
        } else {
            let accu_snapshot = Snapshot::from_id(self.accu, memory_path)?;
            let base_snapshot = Snapshot::from_id(*self.ids.get(0).unwrap(), memory_path)?;
            let snapshot = Snapshot::from_id(*self.ids.get(1).unwrap(), memory_path)?;
            snapshot.decompress(&base_snapshot, &accu_snapshot)?;
            for i in 2..(snapshot_index + 1) {
                let snapshot = Snapshot::from_id(*self.ids.get(i).unwrap(), memory_path)?;
                snapshot.decompress(&accu_snapshot, &accu_snapshot)?;
            }
            accu_snapshot.restore_this(memory_path)
        }
    }
}
