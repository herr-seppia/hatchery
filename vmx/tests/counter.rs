// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use vmx::{module_bytecode, Error, VM};

#[test]
fn counter_read() -> Result<(), Error> {
    let mut vm = VM::ephemeral()?;
    let id = vm.deploy(module_bytecode!("counter"))?;

    assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);

    Ok(())
}

#[test]
fn counter_read_write() -> Result<(), Error> {
    let mut vm = VM::ephemeral()?;
    let id = vm.deploy(module_bytecode!("counter"))?;

    {
        let mut session = vm.session_mut();

        println!("read_value FC");
        assert_eq!(session.query::<(), i64>(id, "read_value", ())?, 0xfc);

        println!("increment");
        session.transact::<(), ()>(id, "increment", ())?;
        session.commit(&id)?; // todo: workaround

        println!("read_value FD");
        assert_eq!(session.query::<(), i64>(id, "read_value", ())?, 0xfd);
    }

    // mutable session dropped without committing.
    // old counter value still accessible.

    println!("read_value FC");
    assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);

    let mut other_session = vm.session_mut();

    println!("increment");
    other_session.transact::<(), ()>(id, "increment", ())?;
    other_session.commit(&id)?; // todo: workaround
    // let commit_id = other_session.commit();

    // session committed, new value accessible

    println!("read_value FD");
    assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfd);

    Ok(())
}
