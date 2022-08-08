// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod event;
mod stack;

pub use event::{Event, Receipt};

use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dallo::{ModuleId, Ser, MODULE_ID_BYTES};
use parking_lot::ReentrantMutex;
use rkyv::{archived_value, Archive, Deserialize, Infallible, Serialize};
use stack::CallStack;
use tempfile::tempdir;
use wasmer::{imports, Exports, Function, Val};

use crate::env::Env;
use crate::error::Error;
use crate::instance::Instance;
use crate::memory::MemHandler;
use crate::snapshot::{MemoryPath, Snapshot, SnapshotLike};
use crate::storage_helpers::module_id_to_name;
use crate::Error::PersistenceError;
use crate::world_snapshot::WorldSnapshotId;

#[derive(Debug)]
pub struct WorldInner {
    environments: BTreeMap<ModuleId, Env>,
    storage_path: PathBuf,
    events: Vec<Event>,
    call_stack: CallStack,
}

impl Deref for WorldInner {
    type Target = BTreeMap<ModuleId, Env>;

    fn deref(&self) -> &Self::Target {
        &self.environments
    }
}

impl DerefMut for WorldInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.environments
    }
}

#[derive(Debug, Clone)]
pub struct World(Arc<ReentrantMutex<UnsafeCell<WorldInner>>>);

impl World {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        World(Arc::new(ReentrantMutex::new(UnsafeCell::new(WorldInner {
            environments: BTreeMap::new(),
            storage_path: path.into(),
            events: vec![],
            call_stack: CallStack::new(ModuleId::uninitialized()),
        }))))
    }

    pub fn ephemeral() -> Result<Self, Error> {
        Ok(World(Arc::new(ReentrantMutex::new(UnsafeCell::new(
            WorldInner {
                environments: BTreeMap::new(),
                storage_path: tempdir()
                    .map_err(PersistenceError)?
                    .path()
                    .into(),
                events: vec![],
                call_stack: CallStack::new(ModuleId::uninitialized()),
            },
        )))))
    }

    pub fn persist(&self) -> Result<WorldSnapshotId, Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };
        let mut world_snapshot_id = WorldSnapshotId::uninitialized();
        for (module_id, environment) in w.environments.iter() {
            let memory_path = MemoryPath::new(self.memory_path(module_id));
            let snapshot = Snapshot::new(&memory_path)?;
            environment.inner_mut().add_snapshot_id(snapshot.id());
            world_snapshot_id.xor(&snapshot.id());
            snapshot.save(&memory_path)?;
            environment.inner_mut().set_dirty(false);
            println!(
                "persisted state of module: {:?} to file: {:?}",
                module_id_to_name(*module_id),
                snapshot.path()
            );
        }
        Ok(world_snapshot_id)
    }

    pub fn restore(&self) -> Result<(), Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };
        for (module_id, environment) in w.environments.iter() {
            let memory_path = MemoryPath::new(self.memory_path(module_id));
            if let Some(snapshot_id) = environment.inner().last_snapshot_id() {
                let snapshot = Snapshot::from_id(*snapshot_id, &memory_path)?;
                snapshot.load(&memory_path)?;
                println!(
                    "restored state of module: {:?} from file: {:?}",
                    module_id_to_name(*module_id),
                    snapshot.path()
                );
            }
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };
        for environment in w.environments.values() {
            if environment.inner().is_dirty() {
                return true;
            }
        }
        false
    }

    pub fn memory_path(&self, module_id: &ModuleId) -> PathBuf {
        self.storage_path().join(module_id_to_name(*module_id))
    }

    pub fn deploy(&mut self, bytecode: &[u8]) -> Result<ModuleId, Error> {
        let id_bytes: [u8; MODULE_ID_BYTES] = blake3::hash(bytecode).into();
        let id = ModuleId::from(id_bytes);
        let store = wasmer::Store::new_with_path(
            self.storage_path().join(module_id_to_name(id)).as_path(),
        );
        let module = wasmer::Module::new(&store, bytecode)?;

        let mut env = Env::uninitialized();

        #[rustfmt::skip]
        let imports = imports! {
            "env" => {
                "alloc" => Function::new_native_with_env(&store, env.clone(), host_alloc),
		        "dealloc" => Function::new_native_with_env(&store, env.clone(), host_dealloc),

                "snap" => Function::new_native_with_env(&store, env.clone(), host_snapshot),

                "q" => Function::new_native_with_env(&store, env.clone(), host_query),
		        "t" => Function::new_native_with_env(&store, env.clone(), host_transact),

                "emit" => Function::new_native_with_env(&store, env.clone(), host_emit),
                "caller" => Function::new_native_with_env(&store, env.clone(), host_caller),
            }
        };

        let instance = wasmer::Instance::new(&module, &imports)?;

        let arg_buf_ofs = global_i32(&instance.exports, "A")?;
        let arg_buf_len_pos = global_i32(&instance.exports, "AL")?;

        // TODO: We should check these buffers have the correct length.
        let self_id_ofs = global_i32(&instance.exports, "SELF_ID")?;

        let heap_base = global_i32(&instance.exports, "__heap_base")?;

        // check buffer alignment
        // debug_assert_eq!(arg_buf_ofs % 8, 0);

        // We need to read the actual value of AL from the offset into memory

        let mem = instance.exports.get_memory("memory")?;
        let data =
            &unsafe { mem.data_unchecked() }[arg_buf_len_pos as usize..][..4];

        let arg_buf_len: i32 = unsafe { archived_value::<i32>(data, 0) }
            .deserialize(&mut Infallible)
            .expect("infallible");

        let instance = Instance::new(
            id,
            instance,
            self.clone(),
            MemHandler::new(heap_base as usize),
            arg_buf_ofs,
            arg_buf_len,
            heap_base,
            self_id_ofs,
        );
        instance.write_self_id(id);

        env.initialize(instance);

        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };
        w.insert(id, env);

        Ok(id)
    }

    pub fn query<Arg, Ret>(
        &self,
        m_id: ModuleId,
        name: &str,
        arg: Arg,
    ) -> Result<Receipt<Ret>, Error>
    where
        Arg: for<'a> Serialize<Ser<'a>>,
        Ret: Archive,
        Ret::Archived: Deserialize<Ret, Infallible>,
    {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.call_stack = CallStack::new(m_id);

        let ret = w
            .get(&m_id)
            .expect("invalid module id")
            .inner()
            .query(name, arg)?;

        let events = mem::take(&mut w.events);

        Ok(Receipt::new(ret, events))
    }

    pub fn transact<Arg, Ret>(
        &mut self,
        m_id: ModuleId,
        name: &str,
        arg: Arg,
    ) -> Result<Receipt<Ret>, Error>
    where
        Arg: for<'a> Serialize<Ser<'a>>,
        Ret: Archive,
        Ret::Archived: Deserialize<Ret, Infallible>,
    {
        let w = self.0.lock();
        let w = unsafe { &mut *w.get() };

        w.call_stack = CallStack::new(m_id);

        let ret = w
            .get_mut(&m_id)
            .expect("invalid module id")
            .inner_mut()
            .transact(name, arg)?;

        let events = mem::take(&mut w.events);

        Ok(Receipt::new(ret, events))
    }

    fn perform_query(
        &self,
        name: &str,
        caller: ModuleId,
        callee: ModuleId,
        arg_ofs: i32,
    ) -> Result<i32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.call_stack.push(callee);

        let caller = w.get(&caller).expect("oh no").inner();
        let callee = w.get(&callee).expect("no oh").inner();

        let mut min_len = 0;

        caller.with_arg_buffer(|buf_caller| {
            callee.with_arg_buffer(|buf_callee| {
                min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_callee[..min_len].copy_from_slice(&buf_caller[..min_len]);
            })
        });

        let ret_ofs = callee.perform_query(name, arg_ofs)?;

        callee.with_arg_buffer(|buf_callee| {
            caller.with_arg_buffer(|buf_caller| {
                buf_caller[..min_len].copy_from_slice(&buf_callee[..min_len]);
            })
        });

        w.call_stack.pop();

        Ok(ret_ofs)
    }

    fn perform_transaction(
        &self,
        name: &str,
        caller: ModuleId,
        callee: ModuleId,
        arg_ofs: i32,
    ) -> Result<i32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.call_stack.push(callee);

        let caller = w.get(&caller).expect("oh no").inner();
        let callee = w.get(&callee).expect("no oh").inner();

        caller.with_arg_buffer(|buf_caller| {
            callee.with_arg_buffer(|buf_callee| {
                let min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_callee[..min_len].copy_from_slice(&buf_caller[..min_len]);
            })
        });

        let ret_ofs = callee.perform_transaction(name, arg_ofs)?;

        callee.with_arg_buffer(|buf_callee| {
            caller.with_arg_buffer(|buf_caller| {
                let min_len = std::cmp::min(buf_caller.len(), buf_callee.len());
                buf_caller[..min_len].copy_from_slice(&buf_callee[..min_len]);
            })
        });

        w.call_stack.pop();

        Ok(ret_ofs)
    }

    fn perform_emit(&self, module_id: ModuleId, data: Vec<u8>) {
        let guard = self.0.lock();
        let w = unsafe { &mut *guard.get() };

        w.events.push(Event::new(module_id, data));
    }

    fn perform_caller(&self, instance: &Instance) -> Result<i32, Error> {
        let guard = self.0.lock();
        let w = unsafe { &*guard.get() };

        let caller = w.call_stack.caller().unwrap_or(ModuleId::uninitialized());

        instance.write_to_arg_buffer(caller)
    }

    pub fn storage_path(&self) -> &Path {
        let guard = self.0.lock();
        let world_inner = unsafe { &*guard.get() };
        world_inner.storage_path.as_path()
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
    method_name_len: i32,
    arg_ofs: i32,
) -> i32 {
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
        .world()
        .perform_query(&name, instance.id(), mod_id, arg_ofs)
        .expect("TODO: error handling")
}

fn host_transact(
    env: &Env,
    module_id_adr: i32,
    method_name_adr: i32,
    method_name_len: i32,
    arg_ofs: i32,
) -> i32 {
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
        .world()
        .perform_transaction(&name, instance.id(), mod_id, arg_ofs)
        .expect("TODO: error handling")
}

fn host_emit(env: &Env, arg_ofs: i32, arg_len: i32) {
    let instance = env.inner();
    let module_id = instance.id();

    let arg_ofs = arg_ofs as usize;
    let arg_len = arg_len as usize;

    let data = instance.with_arg_buffer(|buf| buf[arg_ofs..arg_len].to_vec());

    instance.world().perform_emit(module_id, data);
}

fn host_caller(env: &Env) -> i32 {
    let instance = env.inner();
    instance
        .world()
        .perform_caller(instance)
        .expect("TODO: error handling")
}
