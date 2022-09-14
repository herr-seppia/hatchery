// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[macro_use]
mod bytecode_macro;
mod error;
mod instance;
mod module;
mod session;
mod store;
mod types;
mod vm;
mod vm_linear_memory;

pub use error::Error;
pub use vm::VM;
