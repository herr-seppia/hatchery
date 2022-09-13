// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::types::Error;
use wasmer::{Module, Store};

pub struct WrappedModule {
    serialized: Vec<u8>,
    bytecode: Vec<u8>,
    module: Module,
    store: Store,
}

impl WrappedModule {
    pub fn new(store: Store, bytecode: &[u8]) -> Result<Self, Error> {
        let module = wasmer::Module::new(&store, bytecode)?;
        let serialized = module.serialize()?;

        Ok(WrappedModule {
            serialized,
            bytecode: bytecode.to_vec(),
            module,
            store,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.serialized
    }

    pub fn as_bytecode(&self) -> &[u8] {
        &self.bytecode.as_slice()
    }

    pub fn as_module(&self) -> &Module {
        &self.module
    }

    pub fn as_module_mut(&mut self) -> &mut Module {
        &mut self.module
    }

    pub fn as_store(&self) -> &Store {
        &self.store
    }

    pub fn as_store_mut(&mut self) -> &mut Store {
        &mut self.store
    }
}
