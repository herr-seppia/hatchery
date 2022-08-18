// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use hatchery::{module_bytecode, Error, Receipt, World};
use std::path::PathBuf;

#[test]
pub fn snapshot_hash_excludes_argbuf() -> Result<(), Error> {
    let mut world = World::new(PathBuf::from("/tmp/mmm")); // todo change to ephemeral
    let id = world.deploy(module_bytecode!("box"))?;

    let snapshot_id1 = world.persist()?;
    let _: Receipt<()> = world.transact(id, "noop_query_with_arg", 0x22)?;
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let snapshot_id2 = world.persist()?;
    assert_eq!(snapshot_id1, snapshot_id2);
    let _: Receipt<()> = world.transact(id, "noop_query_with_arg", 0x22)?;
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let snapshot_id3 = world.persist()?;
    assert_eq!(snapshot_id2, snapshot_id3);

    println!("snapshot1 = {:?}", snapshot_id1); // todo remove me
    println!("snapshot2 = {:?}", snapshot_id2); // todo remove me
    println!("snapshot3 = {:?}", snapshot_id3); // todo remove me
    Ok(())
}
