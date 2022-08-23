// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[cfg(not(feature = "std"))]
const _: () = {
    /// Macro to format and send debug output to the host
    #[macro_export]
    macro_rules! debug {
        ($($tt:tt)*) => {
            extern "C" {
                fn host_debug(len: u32);
            }

	    use core::fmt::Write;

            let len = $crate::with_arg_buf(|b| {
                let mut w = $crate::bufwriter::BufWriter::new(b);
                write!(&mut w, $($tt)*).unwrap();
                w.ofs() as u32
            });

            unsafe { host_debug(len) }
        };
    }
};

#[cfg(feature = "std")]
const _: () = {
    #[macro_export]
    macro_rules! debug {
        ($($tt:tt)*) => {
            println!("DEBUG: {}", &format!($($tt)*))
        };
    }
};
