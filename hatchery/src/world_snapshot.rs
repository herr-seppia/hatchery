// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{Archive, Deserialize, Serialize};
use crate::snapshot::SnapshotId;

pub const WORLD_SNAPSHOT_ID_BYTES: usize = 32;
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
pub struct WorldSnapshotId([u8; WORLD_SNAPSHOT_ID_BYTES]);
impl WorldSnapshotId {
    pub const fn uninitialized() -> Self {
        WorldSnapshotId([0u8; WORLD_SNAPSHOT_ID_BYTES])
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn xor(&mut self, snapshot_id: &SnapshotId) {
        let p = snapshot_id.as_bytes().as_ptr();
        for (i, b) in self.0.iter_mut().enumerate() {
            *b = *b ^ unsafe { *p.offset(i as isize) };
        }
    }
}
impl From<[u8; 32]> for WorldSnapshotId {
    fn from(array: [u8; 32]) -> Self {
        WorldSnapshotId(array)
    }
}
