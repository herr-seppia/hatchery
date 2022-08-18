// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use dallo::{HostAlloc, ModuleId, State};
#[global_allocator]
static ALLOCATOR: HostAlloc = HostAlloc;

// One Box, many `Boxen`
pub struct Boxen {
    a: Option<Box<i16>>,
    #[allow(unused)]
    b: i16,
}

#[no_mangle]
static SELF_ID: ModuleId = ModuleId::uninitialized();

static mut STATE: State<Boxen> = State::new(Boxen { a: None, b: 0xbb });

impl Boxen {
    pub fn set(&mut self, x: i16) {
        match self.a.as_mut() {
            Some(o) => **o = x,
            None => self.a = Some(Box::new(x)),
        }
    }

    pub fn noop_query_with_arg(&mut self, _x: i16) {
    }

    pub fn get(&self) -> Option<i16> {
        self.a.as_ref().map(|i| **i)
    }
}

#[no_mangle]
unsafe fn set(arg_len: u32) -> u32 {
    dallo::wrap_transaction(arg_len, |to| STATE.set(to))
}

#[no_mangle]
unsafe fn get(arg_len: u32) -> u32 {
    dallo::wrap_transaction(arg_len, |_: ()| STATE.get())
}

#[no_mangle]
unsafe fn noop_query_with_arg(arg_len: u32) -> u32 {
    dallo::wrap_transaction(arg_len, |to| STATE.noop_query_with_arg(to))
}

#[no_mangle]
unsafe fn mem_snap(_arg_len: u32) -> u32 {
    dallo::snap();
    0
}
