// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::BTreeMap;
use std::mem::{size_of, transmute};
use std::path::{Path, PathBuf};

use bytecheck::CheckBytes;
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Infallible,
    Serialize,
};
use tempfile::tempdir;

use crate::module::WrappedModule;
use crate::session::{Session, SessionId, SessionMut};
use crate::store::new_store_for_compilation;
use crate::types::StandardBufSerializer;
use crate::util::{module_id_to_name, session_id_to_name};
use crate::Error::{self, PersistenceError};

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct ModuleId(usize);

impl ModuleId {
    pub fn to_bytes(&self) -> Box<[u8]> {
        let bytes: [u8; size_of::<usize>()] =
            unsafe { transmute(self.0.to_le()) };
        Box::from(bytes)
    }
}

#[derive(Debug)]
pub struct MemoryPath {
    path: PathBuf,
}

impl MemoryPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        MemoryPath { path: path.into() }
    }
}

impl AsRef<Path> for MemoryPath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

// PreimagePath may be removed once we are able to disable memory initialization
#[derive(Debug)]
pub struct PreimagePath {
    path: PathBuf,
}

impl PreimagePath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        PreimagePath { path: path.into() }
    }
}

impl AsRef<Path> for PreimagePath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

#[derive(Default)]
pub struct VM {
    modules: BTreeMap<ModuleId, WrappedModule>,
    base_memory_path: PathBuf,
    preimage_path: Option<PreimagePath>, /* workaround until we are able to
                                          * disable memory initialization */
}

impl VM {
    pub fn new<P: AsRef<Path>>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        VM {
            modules: BTreeMap::default(),
            base_memory_path: path.into(),
            preimage_path: None,
        }
    }

    pub fn ephemeral() -> Result<Self, Error> {
        let vm = VM {
            modules: BTreeMap::default(),
            base_memory_path: tempdir()
                .map_err(|e| PersistenceError(e))?
                .path()
                .into(),
            preimage_path: None,
        };
        Ok(vm)
    }

    pub fn module_memory_path(&self, module_id: &ModuleId) -> MemoryPath {
        MemoryPath::new(
            self.base_memory_path.join(module_id_to_name(*module_id)),
        )
    }

    pub fn session_memory_path(
        &self,
        module_id: &ModuleId,
        session_id: &SessionId,
    ) -> MemoryPath {
        let session_id_name = &*session_id_to_name(*session_id);
        let mut name = module_id_to_name(*module_id);
        name.push_str(session_id_name);
        MemoryPath::new(self.base_memory_path.join(name))
    }

    pub fn preimage_path(&self) -> Option<PreimagePath> {
        self.preimage_path
            .as_ref()
            .map(|p| PreimagePath::new(p.as_ref()))
    }

    pub fn set_preimage_path(&mut self, preimage_path: Option<PreimagePath>) {
        self.preimage_path = preimage_path;
    }

    pub fn deploy(&mut self, bytecode: &[u8]) -> Result<ModuleId, Error> {
        println!("acquiring new store for compilation");
        let store = new_store_for_compilation();
        let module = WrappedModule::new(&store, bytecode)?;
        let id = ModuleId(self.modules.len());
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

        let tunables = VMLinearTunables::new(tempdir()?.path());
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
        let mut vm = VM::ephemeral()?;
        let id = vm.deploy(module_bytecode!("counter"))?;

        assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);

        Ok(())
    }

    #[test]
    fn counter_read_write() -> Result<(), Error> {
        let mut vm = VM::new("/tmp/b00");
        let id = vm.deploy(module_bytecode!("counter"))?;

        {
            let mut session = vm.session_mut();

            println!("read_value FC");
            assert_eq!(session.query::<(), i64>(id, "read_value", ())?, 0xfc);

            println!("increment");
            session.transact::<(), ()>(id, "increment", ())?;
            session.commit(&id)?; // workaround

            println!("read_value FD");
            assert_eq!(session.query::<(), i64>(id, "read_value", ())?, 0xfd);
        }

        // mutable session dropped without committing.
        // old counter value still accessible.

        println!("read_value FC");
        assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfc);

        let mut other_session = vm.session_mut();

        println!("increment");
        other_session.transact::<(), ()>(id, "increment", ())?;
        other_session.commit(&id)?; // workaround
        // let commit_id = other_session.commit();

        // session committed, new value accessible

        println!("read_value FD");
        assert_eq!(vm.query::<(), i64>(id, "read_value", ())?, 0xfd);

        Ok(())
    }
}
