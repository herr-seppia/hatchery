// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use crate::snapshot::Snapshot;
use crate::snapshot::{MemoryPath, SnapshotId};
use crate::Error::InvalidSnapshotIndex;

#[derive(Debug)]
pub struct SnapshotBag {
    // first snapshot is always uncompressed
    ids: Vec<SnapshotId>,
    // we keep top uncompressed snapshot to make save snapshot efficient
    top: SnapshotId,
}

impl SnapshotBag {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            top: SnapshotId::random(),
        }
    }
    pub fn save_snapshot(
        &mut self,
        snapshot: &Snapshot,
        memory_path: &MemoryPath,
    ) -> Result<usize, Error> {
        snapshot.capture(memory_path)?;
        self.ids.push(snapshot.id());
        if self.ids.len() == 1 {
            // top is always the last uncompressed
            Snapshot::from_id(self.top, memory_path)?.capture(memory_path)?;
            Ok(0)
        } else {
            let top_snapshot = Snapshot::from_id(self.top, memory_path)?;
            let accu_snapshot =
                Snapshot::from_id(SnapshotId::random(), memory_path)?;
            accu_snapshot.capture(snapshot)?;
            // snapshot saved to accu, accu should be uncompressed, so should be
            // snapshot
            // compressing snapshot against top
            snapshot.capture_compressed(&top_snapshot, memory_path)?;
            // now snapshot is compressed but accu keeps the uncompressed copy
            // top is always the last uncompressed, so setting it from accu
            top_snapshot.capture(&accu_snapshot)?;
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
            let accu_snapshot =
                Snapshot::from_id(SnapshotId::random(), memory_path)?;
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
