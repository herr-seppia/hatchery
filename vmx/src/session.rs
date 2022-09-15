// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rand::prelude::*;
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Infallible,
    Serialize,
};
use std::collections::BTreeMap;
use std::mem::{size_of, transmute};

use crate::error::Error;
use crate::instance::WrappedInstance;
use crate::types::StandardBufSerializer;
use crate::vm::{ModuleId, VM};
use crate::Error::SnapshotError;

pub const SESSION_ID_BYTES: usize = 4;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct SessionId([u8; SESSION_ID_BYTES]);

impl SessionId {
    pub fn new() -> SessionId {
        SessionId(thread_rng().gen::<[u8; SESSION_ID_BYTES]>())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

pub struct Session<'a> {
    vm: &'a mut VM,
    instances: BTreeMap<ModuleId, WrappedInstance>,
    id: SessionId,
}

impl<'a> Session<'a> {
    pub fn new(vm: &'a mut VM) -> Self {
        Session {
            vm,
            instances: BTreeMap::new(),
            id: SessionId::new(),
        }
    }

    fn initialize_module(&mut self, id: ModuleId) -> Result<(), Error> {
        if self.instances.get(&id).is_some() {
            return Ok(());
        }
        let memory_path = self.vm.module_memory_path(&id);
        let module = self.vm.module_mut(id);
        let wrapped = WrappedInstance::new(module, memory_path)?;
        self.instances.insert(id, wrapped);
        Ok(())
    }

    fn get_instance(
        &mut self,
        id: ModuleId,
    ) -> Result<&mut WrappedInstance, Error> {
        self.initialize_module(id)?;
        Ok(self.instances.get_mut(&id).expect("initialized above"))
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
        let i = self.get_instance(id)?;
        i.query(method_name, arg)
    }

    pub fn capture(&self, id: &ModuleId) -> Result<(), Error> {
        let source_path = self.vm.module_memory_path(id);
        let target_path = self.vm.session_memory_path(id, &self.id);
        println!("capture from {:?} to {:?}", source_path, target_path);
        std::fs::copy(source_path, target_path).map_err(SnapshotError)?;
        Ok(())
    }

    pub fn restore(&self, id: &ModuleId) -> Result<(), Error> {
        let source_path = self.vm.session_memory_path(id, &self.id);
        let target_path = self.vm.module_memory_path(id);
        println!("restore from {:?} to {:?}", source_path, target_path);
        std::fs::copy(source_path, target_path).map_err(SnapshotError)?;
        Ok(())
    }
}

type CommitId = usize;

pub struct SessionMut<'a> {
    vm: &'a mut VM,
    instances: BTreeMap<ModuleId, WrappedInstance>,
    id: SessionId,
}

impl<'a> SessionMut<'a> {
    pub fn new(vm: &'a mut VM) -> Self {
        SessionMut {
            vm,
            instances: BTreeMap::new(),
            id: SessionId::new(),
        }
    }

    fn initialize_module(&mut self, id: ModuleId) -> Result<(), Error> {
        if self.instances.get(&id).is_some() {
            return Ok(());
        }
        let memory_path = self.vm.module_memory_path(&id);
        let module = self.vm.module_mut(id);
        let wrapped = WrappedInstance::new(module, memory_path)?;
        self.instances.insert(id, wrapped);
        Ok(())
    }

    fn get_instance(
        &mut self,
        id: ModuleId,
    ) -> Result<&mut WrappedInstance, Error> {
        self.initialize_module(id)?;
        Ok(self.instances.get_mut(&id).expect("initialized above"))
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
        let mut session = Session::new(self.vm);
        println!("before query capture");
        session.capture(&id)?;
        println!("after query capture");
        let ret = session.query(id, method_name, arg);
        session.restore(&id)?;
        ret
    }

    pub fn transact<Arg, Ret>(
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
        let i = self.get_instance(id)?;
        i.transact(method_name, arg)
    }

    pub fn capture(&self, id: &ModuleId) -> Result<(), Error> {
        let source_path = self.vm.module_memory_path(id);
        let target_path = self.vm.session_memory_path(id, &self.id);
        println!("capture mut from {:?} to {:?}", source_path, target_path);
        std::fs::copy(source_path, target_path).map_err(SnapshotError)?;
        Ok(())
    }

    pub fn restore(&self, id: &ModuleId) -> Result<(), Error> {
        let source_path = self.vm.session_memory_path(id, &self.id);
        let target_path = self.vm.module_memory_path(id);
        println!("restore mut from {:?} to {:?}", source_path, target_path);
        std::fs::copy(source_path, target_path).map_err(SnapshotError)?;
        Ok(())
    }

    pub fn commit(self) -> CommitId {
        todo!()
    }
}
