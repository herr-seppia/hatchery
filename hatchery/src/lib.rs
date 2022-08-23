// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod env;
mod error;
mod event;
mod instance;
mod memory;
mod session;
mod snapshot;
mod stack;
mod storage_helpers;
mod store;
mod world;

pub use error::Error;
pub use event::{Event, Receipt};
pub use session::Session;
pub use snapshot::Snapshot;
pub use world::World;

#[macro_export]
macro_rules! module_bytecode {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../modules/",
            $name,
            "/target/wasm32-unknown-unknown/release/",
            $name,
            ".wasm"
        ))
    };
}
