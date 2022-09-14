// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::Path;
// use std::sync::Arc;

// use wasmer::wasmparser::Operator;
use wasmer::Store;
use wasmer_compiler_singlepass::Singlepass;
// use wasmer_middlewares::Metering;

use crate::vm_linear_memory::VMLinearTunables;
// use lazy_static::lazy_static;

// fn cost_function(_: &Operator) -> u64 {
//     1
// }

// lazy_static! {
//     static ref CENTRAL_STORE: Store = Store::new_with_tunables(
//         Singlepass::default(),
//         VMLinearTunables {},
//     );
// static ref CENTRAL_STORE: Store = Store::default();
// }

// unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
//     let const_ptr = reference as *const T;
//     let mut_ptr = const_ptr as *mut T;
//     &mut *mut_ptr
// }

/// Creates a new store using the singlepass compiler configured to meter using
/// the default cost function.
pub fn new_store<P: AsRef<Path>>(path: P) -> Store {
    let engine = Singlepass::default();
    // let metering = Arc::new(Metering::new(0, cost_function));

    // compiler_config.push_middleware(metering);

    Store::new_with_tunables(engine, VMLinearTunables::new(path))

    // Store::default()

    // unsafe { very_bad_function(&CENTRAL_STORE) }
}

pub fn new_store_for_compilation() -> Store {
    let engine = Singlepass::default();
    Store::new(engine)
}
