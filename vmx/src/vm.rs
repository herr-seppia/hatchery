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

// todo: PreimagePath may be removed once we are able to disable memory
// initialization
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
    preimage_path: Option<PreimagePath>, /* todo: workaround until we are
                                          * able to disable memory
                                          * initialization */
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
