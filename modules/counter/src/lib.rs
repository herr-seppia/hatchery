// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
#![no_std]
#![no_main]

use rkyv::rend::LittleEndian;
use rkyv::ser::serializers::BufferSerializer;
use rkyv::ser::Serializer;
use rkyv::{archived_root, Deserialize};

#[allow(unused)]
use dallo::snap;

#[derive(Default)]
pub struct Counter {
    value: i32,
}

#[no_mangle]
static mut AR: [u8; 4] = [0u8; 4];

static mut SELF: Counter = Counter { value: 0xfc };

impl Counter {
    pub fn read_value(&self) -> i32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn mogrify(&mut self, x: i32) {
        self.value -= x;
    }
}

#[no_mangle]
fn read_value(_: i32) -> i32 {
    let ret = unsafe { SELF.read_value() };

    let mut ser = unsafe { BufferSerializer::new(&mut AR) };
    let ofs = ser.serialize_value(&ret).unwrap() as i32;
    ofs
}

#[no_mangle]
fn increment(_: i32) -> i32 {
    unsafe { SELF.increment() }
    snap();
    0
}

#[no_mangle]
fn mogrify(arg: i32) -> i32 {
    let ret = {
        let argret = unsafe { &AR.get_unchecked(arg as usize..) };

        let i = unsafe { archived_root::<LittleEndian<i32>>(argret) };

        let i: LittleEndian<i32> =
            i.deserialize(&mut rkyv::Infallible).unwrap();
        let i = i.into();
        unsafe { SELF.mogrify(i) }
    };

    let mut ser = unsafe { BufferSerializer::new(&mut AR) };
    let ofs = ser.serialize_value(&ret).unwrap() as i32;
    ofs
}
