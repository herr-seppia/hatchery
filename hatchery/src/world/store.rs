// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::Path;
use std::ptr::NonNull;
use std::sync::Arc;

use wasmer::vm::{MemoryStyle, TableStyle};
use wasmer::wasmparser::Operator;
use wasmer::{
    BaseTunables, CompilerConfig, MemoryType, Store, TableType, Target,
    Tunables,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;
use wasmer_vm::{
    MemoryError, VMMemory, VMMemoryDefinition, VMTable, VMTableDefinition,
};

fn cost_function(_: &Operator) -> u64 {
    1
}

/// Creates a new store using the singlepass compiler configured to meter using
/// the default cost function.
pub fn new_store<P: AsRef<Path>>(path: P) -> Store {
    let mut compiler_config = Singlepass::default();
    let metering = Arc::new(Metering::new(0, cost_function));

    compiler_config.push_middleware(metering);

    Store::new_with_tunables_and_path(
        &Universal::new(compiler_config).engine(),
        BaseTunables::for_target(&Target::default()),
        path.as_ref().into(),
    )
}

struct CustomTunables {
    base: BaseTunables,
}

// TODO all impls should probably pass to base straight ahead besides
// `create_vm_memory`
impl Tunables for CustomTunables {
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        todo!()
    }

    fn table_style(&self, table: &TableType) -> TableStyle {
        todo!()
    }

    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError> {
        todo!()
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError> {
        todo!()
    }

    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<VMTable, String> {
        todo!()
    }

    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<VMTable, String> {
        todo!()
    }
}
