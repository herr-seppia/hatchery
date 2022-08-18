// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dallo::ModuleId;
use hatchery::Error::SnapshotError;
use hatchery::{
    module_bytecode, Error, ModuleSnapshotId, Receipt, SnapshotId, World,
};

#[test]
pub fn snapshot_hash_excludes_argbuf() -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    let snapshot_id1 = world.persist()?;
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let _: Receipt<()> = world.transact(id, "noop_query_with_arg", 0x22)?;
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let snapshot_id2 = world.persist()?;
    assert_ne!(snapshot_id1, snapshot_id2); // snapshot 1 has empty heap, not init-ed yet
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let _: Receipt<()> = world.transact(id, "noop_query_with_arg", 0x22)?;
    let _: Receipt<()> = world.transact(id, "mem_snap", ())?;
    let snapshot_id3 = world.persist()?;
    assert_eq!(snapshot_id2, snapshot_id3);

    println!("snapshot1 = {:?}", snapshot_id1); // todo remove me
    println!("snapshot2 = {:?}", snapshot_id2); // todo remove me
    println!("snapshot3 = {:?}", snapshot_id3); // todo remove me
    Ok(())
}

#[test]
pub fn world_snapshots_refer_to_different_module_snapshot_ids(
) -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    world.transact::<i16, ()>(id, "set", 0x23)?;
    let value = world.query::<_, Option<i16>>(id, "get", ())?;
    assert_eq!(*value, Some(0x23));

    let snapshot_id1 = world.persist()?;

    world.transact::<i16, ()>(id, "set", 0x24)?;
    let value = world.query::<_, Option<i16>>(id, "get", ())?;
    assert_eq!(*value, Some(0x24));

    let snapshot_id2 = world.persist()?;

    fn get_module_snapshot_id(
        world: &World,
        id: ModuleId,
        snapshot_id: SnapshotId,
    ) -> Result<ModuleSnapshotId, Error> {
        let module_snapshots = world.get_module_snapshots(&snapshot_id)?;
        for (module, module_snapshot_id) in module_snapshots.iter() {
            if *module == id {
                return Ok(*module_snapshot_id);
            }
        }
        Err(SnapshotError(String::from("module snapshot id not found")))
    }

    assert_ne!(
        get_module_snapshot_id(&world, id, snapshot_id1)?,
        get_module_snapshot_id(&world, id, snapshot_id2)?
    );
    Ok(())
}
