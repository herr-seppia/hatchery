// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::snapshot::Snapshot;
use crate::snapshot::{MemoryPath, SnapshotId};
use crate::storage_helpers::snapshot_id_to_name;
use crate::Error::InvalidSnapshotIndex;

#[derive(Debug)]
pub struct SnapshotBag {
    // first snapshot is always uncompressed
    ids: Vec<SnapshotId>,
    // we keep top uncompressed snapshot to make save efficient
    top: SnapshotId,
    // accu is needed for a series of compressed snapshots depending on each
    // other
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
    pub fn save_snapshot(
        &mut self,
        snapshot: &Snapshot,
        memory_path: &MemoryPath,
    ) -> Result<usize, Error> {
        snapshot.save(memory_path)?;
        println!("save snapshot {}", snapshot_id_to_name(snapshot.id()));
        self.ids.push(snapshot.id());
        if self.ids.len() == 1 {
            snapshot.save_from_snapshot(memory_path)?;
            let top_snapshot = Snapshot::from_id(self.top, memory_path)?;
            top_snapshot.save_from_snapshot(memory_path)?;
            println!("snapshot saved, ids len={}", self.ids.len());
            Ok(0)
        } else {
            let top_snapshot = Snapshot::from_id(self.top, memory_path)?;
            let accu_snapshot = Snapshot::from_id(self.accu, memory_path)?;
            println!("saving accu");
            accu_snapshot.save_from_snapshot(snapshot)?;
            println!("snapshot saved to accu, accu should be uncompressed, so should be snapshot");
            println!("compressing snapshot against top");
            snapshot.save_compressed(&top_snapshot, memory_path)?; // now snapshot is compressed but accu keeps the uncompressed copy
            println!("compressing done");
            println!("recreating top from accu");
            top_snapshot.save_from_snapshot(&accu_snapshot)?; // top is always the last uncompressed
            println!("recreating top from accu done");
            Ok(self.ids.len() - 1)
        }
    }
    pub fn restore_snapshot(
        &self,
        snapshot_index: usize,
        memory_path: &MemoryPath,
    ) -> Result<(), Error> {
        if (snapshot_index + 1) > self.ids.len() {
            return Err(InvalidSnapshotIndex);
        }
        if self.ids.len() == 1 || snapshot_index == 0 {
            Snapshot::from_id(self.ids[0], memory_path)?.restore(memory_path)
        } else if self.ids.len() == (snapshot_index + 1) {
            Snapshot::from_id(self.top, memory_path)?.restore(memory_path)
        } else {
            let accu_snapshot = Snapshot::from_id(self.accu, memory_path)?;
            let base_snapshot = Snapshot::from_id(self.ids[0], memory_path)?;
            let snapshot = Snapshot::from_id(self.ids[1], memory_path)?;
            snapshot.decompress(&base_snapshot, &accu_snapshot)?;
            for i in 2..(snapshot_index + 1) {
                let snapshot = Snapshot::from_id(self.ids[i], memory_path)?;
                snapshot.decompress(&accu_snapshot, &accu_snapshot)?;
            }
            accu_snapshot.restore(memory_path)
        }
    }
}
