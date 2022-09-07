// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::Path;
use std::sync::Arc;

use wasmer::wasmparser::Operator;
use wasmer::{BaseTunables, CompilerConfig, Store, Target};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

use crate::vm_linear_memory::VMLinearTunables;

fn cost_function(_: &Operator) -> u64 {
    1
}

/// Creates a new store using the singlepass compiler configured to meter using
/// the default cost function.
// pub fn new_store<P: AsRef<Path>>(path: P) -> Store {
pub fn new_store() -> Store {
    let mut compiler_config = Singlepass::default();
    let metering = Arc::new(Metering::new(0, cost_function));

    compiler_config.push_middleware(metering);

    Store::new_with_tunables(
        compiler_config,
        VMLinearTunables {},
    )
}
