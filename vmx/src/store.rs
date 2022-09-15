// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::Path;
use wasmer::Store;
use wasmer_compiler_singlepass::Singlepass;

use crate::vm_linear_memory::VMLinearTunables;
/// Creates a new store using the singlepass compiler configured to meter using
/// the default cost function.
pub fn new_store<P: AsRef<Path>>(path: P) -> Store {
    let engine = Singlepass::default();
    println!(
        "acquiring new store for instances at path {:?}",
        path.as_ref()
    );
    Store::new_with_tunables(engine, VMLinearTunables::new(path))
}

pub fn new_store_for_compilation() -> Store {
    let engine = Singlepass::default();
    Store::new(engine)
}
