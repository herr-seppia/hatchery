// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dallo::ModuleId;
use hatchery::{module_bytecode, Error, Receipt, World, WorldSnapshotId};
use std::path::PathBuf;

#[test]
pub fn box_set_get() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("box"))?;

    let value: Receipt<Option<i32>> = world.query(id, "get", ())?;

    assert_eq!(*value, None);

    let _: Receipt<()> = world.transact(id, "set", 0x11)?;

    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;

    assert_eq!(*value, Some(0x11));

    Ok(())
}

#[test]
pub fn box_set_store_restore_get() -> Result<(), Error> {
    let mut storage_path = PathBuf::new();
    let first_id: ModuleId;

    {
        let mut first_world = World::ephemeral()?;

        first_id = first_world.deploy(module_bytecode!("box"))?;

        let _: Receipt<()> = first_world.transact(first_id, "set", 0x23)?;

        first_world.storage_path().clone_into(&mut storage_path);
    }

    let mut second_world = World::new(storage_path);

    let second_id = second_world.deploy(module_bytecode!("box"))?;

    assert_eq!(first_id, second_id);

    let value: Receipt<Option<i16>> =
        second_world.query(second_id, "get", ())?;

    assert_eq!(*value, Some(0x23));

    Ok(())
}

#[test]
pub fn world_snapshot_persist_restore() -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    fn create_snapshot(
        world: &mut World,
        id: ModuleId,
        arg: i16,
    ) -> Result<WorldSnapshotId, Error> {
        let _: Receipt<()> = world.transact(id, "set", arg)?;
        let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
        assert_eq!(*value, Some(arg));

        world.persist()
    }

    fn restore_snapshot(
        world: &mut World,
        id: ModuleId,
        world_snapshot_id: &WorldSnapshotId,
        arg: i16,
    ) -> Result<(), Error> {
        world.restore(&world_snapshot_id)?;
        let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
        assert_eq!(*value, Some(arg));
        Ok(())
    }

    let mut snapshot_ids = Vec::new();
    let random_i = vec![3, 1, 0, 4, 2];
    for i in 0..random_i.len() {
        snapshot_ids.push(create_snapshot(&mut world, id, i as i16)?);
    }
    for i in random_i {
        restore_snapshot(&mut world, id, &snapshot_ids[i], (i) as i16)?;
    }
    Ok(())
}
