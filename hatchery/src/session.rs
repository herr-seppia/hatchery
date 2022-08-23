// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;

use dallo::{ModuleId, StandardBufSerializer, StandardDeserialize};
use parking_lot::ReentrantMutex;
use rkyv::{Archive, Serialize};
use wasmer::{imports, Exports, Function, Val};

use crate::env::Env;
use crate::error::Error;
use crate::event::{Event, Receipt};
use crate::instance::Instance;
use crate::memory::MemHandler;
use crate::stack::CallStack;
use crate::store::new_store;
use crate::world::World;

const DEFAULT_POINT_LIMIT: u64 = 4096 * 1024;
const POINT_PASS_PERCENTAGE: u64 = 93;

#[derive(Debug)]
pub struct SessionInner {
    environments: BTreeMap<ModuleId, Env>,
    events: Vec<Event>,
    call_stack: CallStack,
    world: World,
    height: u64,
    limit: u64,
}

#[derive(Debug, Clone)]
pub struct Session(Arc<ReentrantMutex<UnsafeCell<SessionInner>>>);

impl Session {
    pub fn new(world: World) -> Self {
        Session(Arc::new(ReentrantMutex::new(UnsafeCell::new(
            SessionInner {
                environments: BTreeMap::new(),
                events: vec![],
                world,
                call_stack: CallStack::default(),
                height: 0,
                limit: DEFAULT_POINT_LIMIT,
            },
        ))))
    }

    pub fn world(&self) -> &World {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        &w.world
    }

    fn initialize_instance(&self, module_id: ModuleId) -> Result<(), Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        if w.environments.get(&module_id).is_some() {
            return Ok(());
        }

        let mut env = Env::uninitialized();

        let store = new_store(self.world().storage_path());

        let imports = imports! {
            "env" => {
                "alloc" => Function::new_native_with_env(&store, env.clone(), host_alloc),
                "dealloc" => Function::new_native_with_env(&store, env.clone(), host_dealloc),

                "snap" => Function::new_native_with_env(&store, env.clone(), host_snapshot),

                "q" => Function::new_native_with_env(&store, env.clone(), host_query),
                "t" => Function::new_native_with_env(&store, env.clone(), host_transact),
        "host_panic" => Function::new_native_with_env(&store, env.clone(), host_panic),
        "host_debug" => Function::new_native_with_env(&store, env.clone(), host_debug),

                "height" => Function::new_native_with_env(&store, env.clone(), host_height),
                "emit" => Function::new_native_with_env(&store, env.clone(), host_emit),
                "caller" => Function::new_native_with_env(&store, env.clone(), host_caller),
                "limit" => Function::new_native_with_env(&store, env.clone(), host_limit),
                "spent" => Function::new_native_with_env(&store, env.clone(), host_spent),
            }
        };

        let module = self.world().get_module(module_id);

        let instance = wasmer::Instance::new(&module, &imports)?;

        let arg_buf_ofs = global_i32(&instance.exports, "A")?;

        // TODO: We should check these buffers have the correct length.
        let self_id_ofs = global_i32(&instance.exports, "SELF_ID")?;

        let heap_base = global_i32(&instance.exports, "__heap_base")?;

        assert_eq!(arg_buf_ofs % 8, 0);

        let instance = Instance::new(
            module_id,
            instance,
            self.clone(),
            MemHandler::new(heap_base as usize),
            arg_buf_ofs,
            heap_base,
            self_id_ofs,
        );
        instance.write_self_id(module_id);

        env.initialize(instance);

        w.environments.insert(module_id, env);

        Ok(())
    }

    pub fn query<Arg, Ret>(
        &self,
        m_id: ModuleId,
        name: &str,
        arg: Arg,
    ) -> Result<Receipt<Ret>, Error>
    where
        Arg: for<'a> Serialize<StandardBufSerializer<'a>>,
        Ret: Archive,
        Ret::Archived: StandardDeserialize<Ret>,
    {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.call_stack = CallStack::new(m_id, w.limit);

        self.initialize_instance(m_id)?;

        let instance = w
            .environments
            .get(&m_id)
            .expect("invalid module id")
            .inner();
        instance.set_remaining_points(w.limit);

        let ret = instance.query(name, arg)?;
        let remaining = instance.remaining_points();

        let events = mem::take(&mut w.events);

        Ok(Receipt::new(ret, events, w.limit - remaining))
    }

    pub fn transact<Arg, Ret>(
        &mut self,
        m_id: ModuleId,
        name: &str,
        arg: Arg,
    ) -> Result<Receipt<Ret>, Error>
    where
        Arg: for<'a> Serialize<StandardBufSerializer<'a>> + core::fmt::Debug,
        Ret: Archive,
        Ret::Archived: StandardDeserialize<Ret>,
    {
        let w = self.0.lock();
        let w = unsafe { &mut *w.get() };

        w.call_stack = CallStack::new(m_id, w.limit);

        self.initialize_instance(m_id)?;

        let instance = w
            .environments
            .get(&m_id)
            .expect("invalid module id")
            .inner_mut();
        instance.set_remaining_points(w.limit);

        let ret = instance.transact(name, arg)?;
        let remaining = instance.remaining_points();

        let events = mem::take(&mut w.events);

        Ok(Receipt::new(ret, events, w.limit - remaining))
    }

    /// Set the height available to modules.
    pub fn set_height(&mut self, height: u64) {
        let w = self.0.lock();
        let w = unsafe { &mut *w.get() };

        w.height = height;
    }

    /// Set the point limit for the next call.
    pub fn set_point_limit(&mut self, limit: u64) {
        let w = self.0.lock();
        let w = unsafe { &mut *w.get() };

        w.limit = limit;
    }

    fn perform_query(
        &self,
        name: &str,
        caller_id: ModuleId,
        callee_id: ModuleId,
        arg_len: u32,
    ) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        self.initialize_instance(caller_id)?;
        self.initialize_instance(callee_id)?;

        let caller = w.environments.get(&caller_id).expect("oh no").inner();
        let callee = w.environments.get(&callee_id).expect("oh no").inner();

        let remaining = caller.remaining_points();
        let limit = remaining * POINT_PASS_PERCENTAGE / 100;

        println!("a");

        w.call_stack.push(callee_id, limit);

        callee.set_remaining_points(limit);

        println!("b");

        let mut min_len = 0;

        caller.with_arg_buffer(|buf_caller| {
            callee.with_arg_buffer(|buf_callee| {
                min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_callee[..min_len].copy_from_slice(&buf_caller[..min_len]);
            })
        });

        println!("c");

        let ret_ofs = callee.perform_query(name, arg_len)?;

        callee.with_arg_buffer(|buf_callee| {
            caller.with_arg_buffer(|buf_caller| {
                buf_caller[..min_len].copy_from_slice(&buf_callee[..min_len]);
            })
        });

        println!("d");

        let callee_used = limit - callee.remaining_points();
        caller.set_remaining_points(remaining - callee_used);

        w.call_stack.pop();

        println!("e");

        Ok(ret_ofs)
    }

    fn perform_transaction(
        &self,
        name: &str,
        caller_id: ModuleId,
        callee_id: ModuleId,
        arg_len: u32,
    ) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        self.initialize_instance(caller_id)?;
        self.initialize_instance(callee_id)?;

        let caller = w.environments.get(&caller_id).expect("oh no").inner();
        let callee = w.environments.get(&callee_id).expect("no oh").inner();

        let remaining = caller.remaining_points();
        let limit = remaining * POINT_PASS_PERCENTAGE / 100;

        w.call_stack.push(callee_id, limit);

        callee.set_remaining_points(limit);

        caller.with_arg_buffer(|buf_caller| {
            callee.with_arg_buffer(|buf_callee| {
                let min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_callee[..min_len].copy_from_slice(&buf_caller[..min_len]);
            })
        });

        let ret_len = callee.perform_transaction(name, arg_len)?;

        callee.with_arg_buffer(|buf_callee| {
            caller.with_arg_buffer(|buf_caller| {
                let min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_caller[..min_len].copy_from_slice(&buf_callee[..min_len]);
            })
        });

        let callee_used = limit - callee.remaining_points();
        caller.set_remaining_points(remaining - callee_used);

        w.call_stack.pop();

        Ok(ret_len)
    }

    fn perform_height(&self, instance: &Instance) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &*guard.get() };

        instance.write_to_arg_buffer(w.height)
    }

    fn perform_emit(&self, module_id: ModuleId, data: Vec<u8>) {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.events.push(Event::new(module_id, data));
    }

    fn perform_panic(&self, ofs: i32, len: u32) {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        todo!()
    }

    fn perform_limit(&self, instance: &Instance) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &*guard.get() };

        let limit = w.call_stack.limit();
        instance.write_to_arg_buffer(limit)
    }

    fn perform_spent(&self, instance: &Instance) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &*guard.get() };

        let limit = w.call_stack.limit();
        let remaining = instance.remaining_points();

        instance.write_to_arg_buffer(limit - remaining)
    }

    fn perform_caller(&self, instance: &Instance) -> Result<u32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &*guard.get() };
        let caller = w.call_stack.caller();

        instance.write_to_arg_buffer(caller)
    }
}

fn global_i32(exports: &Exports, name: &str) -> Result<i32, Error> {
    if let Val::I32(i) = exports.get_global(name)?.get() {
        Ok(i)
    } else {
        Err(Error::MissingModuleExport)
    }
}

fn host_alloc(env: &Env, amount: i32, align: i32) -> i32 {
    env.inner_mut()
        .alloc(amount as usize, align as usize)
        .try_into()
        .expect("i32 overflow")
}

fn host_dealloc(env: &Env, addr: i32) {
    env.inner_mut().dealloc(addr as usize)
}

// Debug helper to take a snapshot of the memory of the running process.
fn host_snapshot(env: &Env) {
    env.inner().snap()
}

fn host_query(
    env: &Env,
    module_id_adr: i32,
    method_name_adr: i32,
    method_name_len: u32,
    arg_len: u32,
) -> u32 {
    let module_id_adr = module_id_adr as usize;
    let method_name_adr = method_name_adr as usize;
    let method_name_len = method_name_len as usize;

    let instance = env.inner();
    let mut mod_id = ModuleId::uninitialized();
    // performance: use a dedicated buffer here?
    let mut name = String::new();

    instance.with_memory(|buf| {
        mod_id.as_bytes_mut()[..].copy_from_slice(
            &buf[module_id_adr..][..core::mem::size_of::<ModuleId>()],
        );
        let utf =
            core::str::from_utf8(&buf[method_name_adr..][..method_name_len])
                .expect("TODO, error out cleaner");
        name.push_str(utf)
    });

    instance
        .session()
        .perform_query(&name, instance.id(), mod_id, arg_len)
        .expect("TODO: error handling")
}

fn host_transact(
    env: &Env,
    module_id_adr: i32,
    method_name_adr: i32,
    method_name_len: u32,
    arg_len: u32,
) -> u32 {
    let module_id_adr = module_id_adr as usize;
    let method_name_adr = method_name_adr as usize;
    let method_name_len = method_name_len as usize;

    let instance = env.inner();
    let mut mod_id = ModuleId::uninitialized();
    // performance: use a dedicated buffer here?
    let mut name = String::new();

    instance.with_memory(|buf| {
        mod_id.as_bytes_mut()[..].copy_from_slice(
            &buf[module_id_adr..][..core::mem::size_of::<ModuleId>()],
        );
        let utf =
            core::str::from_utf8(&buf[method_name_adr..][..method_name_len])
                .expect("TODO, error out cleaner");
        name.push_str(utf)
    });

    instance
        .session()
        .perform_transaction(&name, instance.id(), mod_id, arg_len)
        .expect("TODO: error handling")
}

fn host_height(env: &Env) -> u32 {
    let instance = env.inner();
    instance
        .session()
        .perform_height(instance)
        .expect("TODO: error handling")
}

fn host_emit(env: &Env, arg_len: u32) {
    let instance = env.inner();
    let module_id = instance.id();

    let arg_len = arg_len as usize;

    let data = instance.with_arg_buffer(|buf| buf[..arg_len].to_vec());

    instance.session().perform_emit(module_id, data);
}

fn host_spent(env: &Env) -> u32 {
    let instance = env.inner();
    instance
        .session()
        .perform_spent(instance)
        .expect("TODO: error handling")
}

fn host_limit(env: &Env) -> u32 {
    let instance = env.inner();
    instance
        .session()
        .perform_limit(instance)
        .expect("TODO: error handling")
}

fn host_caller(env: &Env) -> u32 {
    let instance = env.inner();
    instance
        .session()
        .perform_caller(instance)
        .expect("TODO: error handling")
}

fn host_panic(env: &Env, len: u32) {
    let instance = env.inner();
    instance.panic(len)
}

fn host_debug(env: &Env, len: u32) {
    let instance = env.inner();
    instance.debug(len)
}
