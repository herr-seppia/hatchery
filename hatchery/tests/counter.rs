// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use hatchery::{module_bytecode, Error, Receipt, World};

#[test]
pub fn counter_trivial() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("counter"))?;

    let session = world.session();

    let value: Receipt<i64> = session.query(id, "read_value", ())?;

    assert_eq!(*value, 0xfc);

    Ok(())
}

#[test]
pub fn counter_increment() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("counter"))?;

    let mut session = world.session();

    let _: Receipt<()> = session.transact(id, "increment", ())?;

    let value: Receipt<i64> = session.query(id, "read_value", ())?;
    assert_eq!(*value, 0xfd);

    let _: Receipt<()> = session.transact(id, "increment", ())?;

    let value: Receipt<i64> = session.query(id, "read_value", ())?;
    assert_eq!(*value, 0xfe);

    Ok(())
}

#[test]
pub fn counter_mogrify() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("counter"))?;

    let mut session = world.session();

    let value = session.transact::<i64, i64>(id, "mogrify", 32)?;

    assert_eq!(*value, 0xfc);

    let value: Receipt<i64> = session.query(id, "read_value", ())?;
    assert_eq!(*value, 0xfc - 32);

    Ok(())
}
