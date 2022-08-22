// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use hatchery::{module_bytecode, Error, Receipt, World};
use std::path::PathBuf;

#[test]
pub fn box_set_get() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("box"))?;

    let mut session = world.session();

    let value: Receipt<Option<i16>> = session.query(id, "get", ())?;

    assert_eq!(*value, None);

    session.transact::<i16, ()>(id, "set", 0x11)?;

    let value = session.query::<_, Option<i16>>(id, "get", ())?;

    assert_eq!(*value, Some(0x11));

    Ok(())
}

#[test]
pub fn box_set_store_restore_get() -> Result<(), Error> {
    let mut storage_path = PathBuf::new();

    let first_id = {
        let mut first_world = World::ephemeral()?;

        let id = first_world.deploy(module_bytecode!("box"))?;

        let mut first_session = first_world.session();

        first_session.transact::<i16, ()>(id, "set", 0x23)?;

        first_world.storage_path().clone_into(&mut storage_path);

        id
    };

    let mut second_world = World::restore_or_create(storage_path)?;

    let second_id = second_world.deploy(module_bytecode!("box"))?;

    let second_session = second_world.session();

    assert_eq!(first_id, second_id);

    let value = second_session.query::<_, Option<i16>>(second_id, "get", ())?;

    assert_eq!(*value, Some(0x23));

    Ok(())
}

#[test]
pub fn world_persist_restore() -> Result<(), Error> {
    let mut world = World::ephemeral()?;
    let id = world.deploy(module_bytecode!("box"))?;

    let mut session = world.session();

    session.transact::<i16, ()>(id, "set", 17)?;

    let value = session.query::<_, Option<i16>>(id, "get", ())?;

    assert_eq!(*value, Some(17));

    world.persist()?;

    session.transact::<i16, ()>(id, "set", 18)?;
    let value = session.query::<_, Option<i16>>(id, "get", ())?;
    assert_eq!(*value, Some(18));

    world.restore()?;
    let value: Receipt<Option<i16>> = session.query(id, "get", ())?;
    assert_eq!(*value, Some(17));

    Ok(())
}
