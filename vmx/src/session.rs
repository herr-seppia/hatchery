// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Infallible,
    Serialize,
};
use std::collections::BTreeMap;

use crate::instance::WrappedInstance;
use crate::types::{Error, StandardBufSerializer};
use crate::vm::{ModuleId, VM};

pub struct Session<'a> {
    vm: &'a VM,
    instances: BTreeMap<ModuleId, WrappedInstance<'a>>,
}

impl<'a> Session<'a> {
    pub fn new(vm: &'a VM) -> Self {
        Session {
            vm,
            instances: BTreeMap::new(),
        }
    }

    fn initialize_module(&mut self, id: ModuleId) -> Result<(), Error> {
        println!("ini module0");
        if self.instances.get(&id).is_some() {
            return Ok(());
        }
        println!("ini module1");
        let module = self.vm.module(id);
        println!("ini module2");
        let wrapped = WrappedInstance::new(module)?;
        println!("ini module3");
        self.instances.insert(id, wrapped);
        println!("ini module4");
        Ok(())
    }

    fn get_instance(
        &mut self,
        id: ModuleId,
    ) -> Result<&mut WrappedInstance<'a>, Error> {
        println!("get instance1");
        self.initialize_module(id)?;
        println!("get instance1.1");
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
}

type CommitId = usize;

pub struct SessionMut<'a> {
    vm: &'a mut VM,
    instances: BTreeMap<ModuleId, WrappedInstance<'a>>,
}

impl<'a> SessionMut<'a> {
    pub fn new(vm: &'a mut VM) -> Self {
        SessionMut {
            vm,
            instances: BTreeMap::new(),
        }
    }

    fn initialize_module(&mut self, id: ModuleId) -> Result<(), Error> {
        if self.instances.get(&id).is_some() {
            return Ok(());
        }
        let module = self.vm.module(id);
        let wrapped = WrappedInstance::new(module)?;
        self.instances.insert(id, wrapped);
        Ok(())
    }

    fn get_instance(
        &mut self,
        id: ModuleId,
    ) -> Result<&mut WrappedInstance<'a>, Error> {
        println!("get instance2");
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
        session.query(id, method_name, arg)
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

    pub fn commit(self) -> CommitId {
        todo!()
    }
}
