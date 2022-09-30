// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use wasmer::{imports, Function, FunctionEnv, FunctionEnvMut};

use uplink::{ModuleId, ARGBUF_LEN};

use crate::instance::Env;

pub(crate) struct DefaultImports;

impl DefaultImports {
    pub fn default(store: &mut wasmer::Store, env: Env) -> wasmer::Imports {
        let fenv = FunctionEnv::new(store, env);

        imports! {
            "env" => {
                "caller" => Function::new_typed_with_env(store, &fenv, caller),
                "q" => Function::new_typed_with_env(store, &fenv, q),
                "t" => Function::new_typed_with_env(store, &fenv, t),
                "nq" => Function::new_typed_with_env(store, &fenv, nq),
                "host_debug" => Function::new_typed_with_env(store, &fenv, host_debug),
                "emit" => Function::new_typed_with_env(store, &fenv, emit),
            }
        }
    }
}

fn caller(env: FunctionEnvMut<Env>) {
    let env = env.data();

    let mod_id = env.nth_from_top(1);

    env.self_instance().with_arg_buffer(|arg| {
        arg[..std::mem::size_of::<ModuleId>()]
            .copy_from_slice(mod_id.as_bytes())
    })
}

fn q(
    mut fenv: FunctionEnvMut<Env>,
    mod_id_ofs: i32,
    name_ofs: i32,
    name_len: u32,
    arg_len: u32,
) -> u32 {
    let env = fenv.data_mut();

    let instance = env.self_instance();
    let argbuf_ofs = instance.arg_buffer_offset();

    instance.with_memory_mut(|memory| {
        let (ret_len, mut callee) = {
            let name = core::str::from_utf8(
                &memory[name_ofs as usize..][..name_len as usize],
            )
            .expect("TODO error handling");

            let arg_buf = &memory[argbuf_ofs..][..ARGBUF_LEN];
            let mut mod_id = ModuleId::uninitialized();
            mod_id.as_bytes_mut().copy_from_slice(
                &memory[mod_id_ofs as usize..]
                    [..std::mem::size_of::<ModuleId>()],
            );

            let mut callee = env.instance(mod_id);
            let arg = &arg_buf[..arg_len as usize];

            env.push_callstack(mod_id);

            callee.write_argument(arg);
            let ret_len =
                callee.query(name, arg.len() as u32).expect("invalid query");
            (ret_len, callee)
        };

        env.pop_callstack();

        // copy back result
        callee.read_argument(&mut memory[argbuf_ofs..][..ret_len as usize]);
        ret_len
    })
}

fn t(
    mut fenv: FunctionEnvMut<Env>,
    mod_id_ofs: i32,
    name_ofs: i32,
    name_len: u32,
    arg_len: u32,
) -> u32 {
    let env = fenv.data_mut();

    let instance = env.self_instance();
    let argbuf_ofs = instance.arg_buffer_offset();

    instance.with_memory_mut(|memory| {
        let (ret_len, mut callee) = {
            let name = core::str::from_utf8(
                &memory[name_ofs as usize..][..name_len as usize],
            )
            .expect("TODO error handling");

            let arg_buf = &memory[argbuf_ofs..][..ARGBUF_LEN];

            let mut mod_id = ModuleId::uninitialized();
            mod_id.as_bytes_mut().copy_from_slice(
                &memory[mod_id_ofs as usize..]
                    [..std::mem::size_of::<ModuleId>()],
            );

            let mut callee = env.instance(mod_id);

            let arg = &arg_buf[..arg_len as usize];

            env.push_callstack(mod_id);

            callee.write_argument(arg);
            let ret_len = callee
                .transact(name, arg.len() as u32)
                .expect("invalid transaction");
            (ret_len, callee)
        };

        env.pop_callstack();

        // copy back result
        callee.read_argument(&mut memory[argbuf_ofs..][..ret_len as usize]);
        ret_len
    })
}

fn nq(
    mut fenv: FunctionEnvMut<Env>,
    name_ofs: i32,
    name_len: u32,
    arg_len: u32,
) -> u32 {
    let env = fenv.data_mut();

    let instance = env.self_instance();

    let name_ofs = name_ofs as usize;
    let name_len = name_len as usize;

    let name = instance.with_memory(|buf| {
        // performance: use a dedicated buffer here?
        core::str::from_utf8(&buf[name_ofs..][..name_len])
            .expect("TODO, error out cleaner")
            .to_owned()
    });

    instance
        .with_arg_buffer(|buf| env.host_query(&name, buf, arg_len))
        .expect("TODO: error handling")
}

fn emit(mut fenv: FunctionEnvMut<Env>, arg_len: u32) {
    let env = fenv.data_mut();
    env.emit(arg_len)
}

fn host_debug(fenv: FunctionEnvMut<Env>, msg_ofs: i32, msg_len: u32) {
    let env = fenv.data();

    env.self_instance().with_memory(|mem| {
        let slice = &mem[msg_ofs as usize..][..msg_len as usize];

        let msg = std::str::from_utf8(slice).expect("Invalid debug string");

        env.register_debug(msg);

        println!("MODULE DEBUG {:?}", msg)
    })
}
