// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::BTreeMap;

use bytecheck::CheckBytes;
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Infallible,
    Serialize,
};

use crate::module::WrappedModule;
use crate::session::{Session, SessionMut};
use crate::store::new_store_for_compilation;
use crate::types::{Error, StandardBufSerializer};

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct ModuleId(usize);

#[derive(Default)]
pub struct VM {
    modules: BTreeMap<ModuleId, WrappedModule>,
}

impl VM {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn deploy(&mut self, bytecode: &[u8]) -> Result<ModuleId, Error> {
        println!("acquiring new store");
        let store = new_store_for_compilation();
        let id = ModuleId(self.modules.len());
        let module = WrappedModule::new(store, bytecode)?;
        self.modules.insert(id, module);
        Ok(id)
    }

    pub fn module(&self, id: ModuleId) -> &WrappedModule {
        self.modules.get(&id).expect("Invalid ModuleId")
    }

    pub fn module_mut(&mut self, id: ModuleId) -> &mut WrappedModule {
        self.modules.get_mut(&id).expect("Invalid ModuleId")
    }

    pub fn query<Arg, Ret>(
        &mut self,
        id: ModuleId,
        method_name: &str,
        arg: Arg,
    ) -> Result<Ret, Error>
    where
        Arg: for<'b> Serialize<StandardBufSerializer<'b>>,
        Ret: Archive,
        Ret::Archived: Deserialize<Ret, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let mut session = Session::new(self);
        session.query(id, method_name, arg)
    }

    pub fn session_mut(&mut self) -> SessionMut {
        SessionMut::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm_linear_memory::VMLinearTunables;

    #[test]
    fn check_customtunables() -> Result<(), Box<dyn std::error::Error>> {
        use wasmer::{imports, wat2wasm, Instance, Memory, Module, Store};
        use wasmer_compiler_cranelift::Cranelift;
        use wasmer_types::Pages;

        // let wasm_bytes = wat2wasm(br#"(module (memory 3) (export "memory"
        // (memory 0)))"#)?;
        let wasm_bytes = wat2wasm(
            br#"(module
                (type (;0;) (func (param i32) (result i32)))
        (func (;0;) (type 0) (param i32) (result i32)
        (local i32)
        i32.const 0
        i32.load offset=1048576
        local.set 1
        i32.const 0
        local.get 0
        i32.store offset=1048576
        local.get 1
        )
        (table (;0;) 1 1 funcref)
        (memory (;0;) 18)
        (global (;0;) (mut i32) i32.const 1048576)
        (global (;1;) i32 i32.const 1048584)
        (global (;2;) i32 i32.const 1114120)
        (global (;3;) i32 i32.const 1114128)
        (export "memory" (memory 0))
        (export "change" (func 0))
        (export "A" (global 1))
        (export "__data_end" (global 2))
        (export "__heap_base" (global 3))
        (data (;0;) (i32.const 1048576) "*\00\00\00")
        )"#,
        )?;
        let compiler = Cranelift::default();

        let tunables = VMLinearTunables {};
        let mut store = Store::new_with_tunables(compiler, tunables);
        let module = Module::new(&store, wasm_bytes)?;
        let import_object = imports! {};
        println!("about to create instance");
        let instance = Instance::new(&mut store, &module, &import_object)?;
        println!("after instance creation");

        let mut memories: Vec<Memory> = instance
            .exports
            .iter()
            .memories()
            .map(|pair| pair.1.clone())
            .collect();
        assert_eq!(memories.len(), 1);
        let first_memory = memories.pop().unwrap();
        assert_eq!(first_memory.ty(&store).maximum.unwrap(), Pages(18));
        let view = first_memory.view(&store);
        let x = unsafe { view.data_unchecked_mut() }[0];
        assert_eq!(x, 0);

        Ok(())
    }

    #[test]
    fn counter_read() -> Result<(), Error> {
        let mut vm = VM::new();
        let id = vm.deploy(module_bytecode!("counter"))?;
        println!("after deploy");

        assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);

        Ok(())
    }

    #[test]
    fn counter_read_write() -> Result<(), Error> {
        let mut vm = VM::new();
        let id = vm.deploy(module_bytecode!("counter"))?;

        {
            let mut session = vm.session_mut();

            assert_eq!(session.query::<(), i64>(id, "read_value", ())?, 0xfc);

            session.transact::<(), ()>(id, "increment", ())?;

            // assert_eq!(session.query::<(), i64>(id, "read_value", ())?,
            // 0xfd);
        }

        // mutable session dropped without commiting.
        // old counter value still accessible.

        // assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);
        //
        // let mut other_session = vm.session_mut();
        //
        // other_session.transact::<(), ()>(id, "increment", ())?;
        //
        // let commit_id = other_session.commit();

        // session committed, new value accessible

        // assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfd);

        Ok(())
    }
}
