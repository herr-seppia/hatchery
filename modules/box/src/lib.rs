// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
#![no_std]
#![no_main]

use dallo::Box;

pub struct Boxen {
    a: Option<Box<i16>>,
}

static mut SELF: Boxen = Boxen { a: None };

impl Boxen {
    pub fn set(&mut self, x: i16) {
        match self.a.as_mut() {
            Some(o) => **o = x,
            None => self.a = Some(Box::new(x)),
        }
    }

    pub fn get(&mut self) -> Option<i16> {
        self.a.as_ref().map(|i| **i)
    }
}

#[no_mangle]
fn set(x: i16) {
    unsafe { SELF.set(x) }
}

#[no_mangle]
fn get() -> Option<i16> {
    unsafe { SELF.get() }
}
