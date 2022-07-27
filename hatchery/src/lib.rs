// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod env;
mod error;
mod instance;
mod memory;
mod storage_helpers;
mod world;

pub use error::Error;
pub use storage_helpers::create_snapshot_id;
pub use world::World;

#[macro_export]
macro_rules! module_bytecode {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../target/wasm32-unknown-unknown/release/",
            $name,
            ".wasm"
        ))
    };
}
