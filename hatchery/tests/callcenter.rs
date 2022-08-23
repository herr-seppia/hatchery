// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dallo::{RawQuery, RawResult, RawTransaction};
use hatchery::{module_bytecode, Error, Receipt, World};

#[test]
#[ignore]
pub fn world_center_counter_read() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let counter_id = world.deploy(module_bytecode!("counter"))?;
    let center_id = world.deploy(module_bytecode!("callcenter"))?;

    let session = world.session();

    let value: Receipt<i64> = session.query(counter_id, "read_value", ())?;
    assert_eq!(*value, 0xfc);

    // read value through callcenter
    let value: Receipt<i64> =
        session.query(center_id, "query_counter", counter_id)?;
    assert_eq!(*value, 0xfc);

    Ok(())
}

#[test]
#[ignore]
pub fn world_center_counter_direct() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let counter_id = world.deploy(module_bytecode!("counter"))?;
    let center_id = world.deploy(module_bytecode!("callcenter"))?;

    let mut session = world.session();

    // read value directly
    let value: Receipt<i64> = session.query(counter_id, "read_value", ())?;
    assert_eq!(*value, 0xfc);

    // read value through callcenter
    let value: Receipt<i64> =
        session.query(center_id, "query_counter", counter_id)?;
    assert_eq!(*value, 0xfc);

    // increment through call center
    let _: Receipt<()> =
        session.transact(center_id, "increment_counter", counter_id)?;

    // read value directly
    let value: Receipt<i64> = session.query(counter_id, "read_value", ())?;
    assert_eq!(*value, 0xfd);

    // read value through callcenter
    let value: Receipt<i64> =
        session.query(center_id, "query_counter", counter_id)?;
    assert_eq!(*value, 0xfd);

    Ok(())
}

#[test]
pub fn query_passthrough() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let center_id = world.deploy(module_bytecode!("callcenter"))?;
    let session = world.session();

    let rq = RawQuery::new("read_value", ());

    let res: Receipt<RawQuery> =
        session.query(center_id, "query_passthrough", rq.clone())?;

    assert_eq!(rq, res.into_inner());

    Ok(())
}

#[test]
pub fn world_center_counter_delegated() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let counter_id = world.deploy(module_bytecode!("counter"))?;
    let center_id = world.deploy(module_bytecode!("callcenter"))?;

    let mut session = world.session();

    let rq = RawQuery::new("read_value", ());

    // read value through callcenter
    let res = session.query::<_, RawResult>(
        center_id,
        "delegate_query",
        (counter_id, rq),
    )?;

    let value: i64 = res.cast();

    assert_eq!(value, 0xfc);

    // increment through delegated transaction

    let rt = RawTransaction::new("increment", ());

    let _: Receipt<()> = session.transact(
        center_id,
        "delegate_transaction",
        (counter_id, rt),
    )?;

    // read value directly
    let value: Receipt<i64> = session.query(counter_id, "read_value", ())?;
    assert_eq!(*value, 0xfd);

    Ok(())
}

#[test]
pub fn world_center_calls_self() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let center_id = world.deploy(module_bytecode!("callcenter"))?;

    let session = world.session();

    // am i calling myself
    let calling_self: Receipt<bool> =
        session.query(center_id, "calling_self", center_id)?;
    assert!(*calling_self);

    Ok(())
}

#[test]
pub fn world_center_caller() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let center_id = world.deploy(module_bytecode!("callcenter"))?;

    let session = world.session();

    let value: Receipt<bool> = session.query(center_id, "call_self", ())?;
    assert!(*value);

    Ok(())
}
