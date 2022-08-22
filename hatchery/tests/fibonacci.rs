// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use hatchery::{module_bytecode, Error, World};

#[test]
pub fn fibo() -> Result<(), Error> {
    let mut world = World::ephemeral()?;

    let id = world.deploy(module_bytecode!("fibonacci"))?;

    let mut session = world.session();

    assert_eq!(*session.query::<u32, u64>(id, "nth", 0)?, 1);
    assert_eq!(*session.query::<u32, u64>(id, "nth", 1)?, 1);
    assert_eq!(*session.query::<u32, u64>(id, "nth", 2)?, 2);
    assert_eq!(*session.query::<u32, u64>(id, "nth", 3)?, 3);
    assert_eq!(*session.query::<u32, u64>(id, "nth", 4)?, 5);

    Ok(())
}
