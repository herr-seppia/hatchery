// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::panic::PanicInfo;

use crate::bufwriter::BufWriter;

#[alloc_error_handler]
#[allow(clippy::empty_loop)]
fn foo(_: core::alloc::Layout) -> ! {
    loop {}
}

extern "C" {
    pub(crate) fn host_panic(len: u32);
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(msg) = panic_info.message() {
        let len = crate::state::with_arg_buf(|b| {
            let mut w = BufWriter::new(b);
            core::fmt::write(&mut w, *msg).unwrap();
            w.ofs() as u32
        });
        unsafe { host_panic(len) }
    } else {
        unsafe { host_panic(0) }
    }
    unreachable!()
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
