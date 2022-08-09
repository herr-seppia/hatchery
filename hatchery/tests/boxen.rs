// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dallo::ModuleId;
use hatchery::{module_bytecode, Error, Receipt, World};
use std::path::PathBuf;

#[ignore]
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

#[ignore]
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

#[ignore]
pub fn world_persist_dirty_flag() -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    let _: Receipt<()> = world.transact(id, "set", 17)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(17));
    assert_eq!(world.is_dirty(), true);
    let snapshot1 = world.persist()?;
    assert_eq!(world.is_dirty(), false);

    let _value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(world.is_dirty(), false);
    let snapshot2 = world.persist()?;
    assert_eq!(world.is_dirty(), false);
    assert_eq!(snapshot1, snapshot2); // query does not cause new snapshot to be created

    let _: Receipt<()> = world.transact(id, "set", 18)?;
    assert_eq!(world.is_dirty(), true);
    let snapshot3 = world.persist()?;
    assert_eq!(world.is_dirty(), false);
    assert_ne!(snapshot2, snapshot3); // transaction causes new snapshot to be created

    Ok(())
}

#[test]
pub fn world_persist_restore() -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    let _: Receipt<()> = world.transact(id, "set", 17)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(17));
    let snapshot1 = world.persist()?;

    let _: Receipt<()> = world.transact(id, "set", 18)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(18));
    let snapshot2 = world.persist()?;

    let _: Receipt<()> = world.transact(id, "set", 19)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(19));
    let snapshot3 = world.persist()?;

    world.restore(&snapshot1)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(17));

    world.restore(&snapshot2)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(18));

    world.restore(&snapshot3)?;
    let value: Receipt<Option<i16>> = world.query(id, "get", ())?;
    assert_eq!(*value, Some(19));

    Ok(())
}
