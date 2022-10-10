// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

use parking_lot::RwLock;

use uplink::ModuleId;

use crate::error::Error;
use crate::linear::{Linear, MAX_MEMORY_BYTES, MEMORY_PAGES, WASM_PAGE_SIZE};
use crate::vm::VM;

#[derive(Clone)]
pub struct MemoryHandler {
    memories: Arc<RwLock<BTreeMap<ModuleId, Linear>>>,
    #[allow(unused)]
    vm: VM,
}

impl MemoryHandler {
    pub fn new(vm: VM) -> Self {
        MemoryHandler {
            memories: Arc::new(RwLock::new(BTreeMap::new())),
            vm,
        }
    }

    pub fn get_memory(&self, mod_id: ModuleId) -> Result<Linear, Error> {
        {
            let rg = self.memories.read();
            if let Some(mem) = rg.get(&mod_id) {
                return Ok(mem.clone());
            }
        }

        self.vm.with_module(mod_id, |module| {
            let (path, fresh) = self.vm.memory_path(&mod_id);
            if path.as_ref().exists() {
                fs::remove_file(path.as_ref()).expect("file removed if exists");
            }
            let result = Linear::new(
                Some(path),
                MEMORY_PAGES * WASM_PAGE_SIZE,
                MAX_MEMORY_BYTES,
                fresh,
                module.volatile().clone(),
            );
            result.map(|mem| {
                self.memories.write().insert(mod_id, mem.clone());
                mem
            })
        })
    }

    pub fn with_every_module_id<F>(&self, mut closure: F) -> Result<(), Error>
    where
        F: FnMut(&ModuleId, &[u8]) -> Result<(), Error>,
    {
        let guard = self.memories.read();
        for module_id in guard.keys() {
            let linear = guard.get(module_id).expect("linear memory exists");
            closure(module_id, linear.as_slice())?
        }
        Ok(())
    }
}
