// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(arbitrary_self_types)]
#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: dallo::HostAlloc = dallo::HostAlloc;

#[derive(Default)]
pub struct Eventer;

use dallo::{ModuleId, State};

#[no_mangle]
static SELF_ID: ModuleId = ModuleId::uninitialized();

static mut STATE: State<Eventer> = State::new(Eventer);

impl Eventer {
    pub fn emit_num(&self, num: u32) {
        for i in 0..num {
            dallo::emit(i);
        }
    }
}

#[no_mangle]
unsafe fn emit_events(arg_len: u32) -> u32 {
    dallo::wrap_query(arg_len, |num| STATE.emit_num(num))
}
