// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use vmx::VMLinearTunables;
use tempfile::tempdir;

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
