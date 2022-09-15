// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::session::SessionId;
use crate::vm::ModuleId;
use rand::prelude::*;

pub fn module_id_to_name(module_id: ModuleId) -> String {
    format!("{}", ByteArrayWrapper(module_id.to_bytes().as_ref()))
}

pub fn session_id_to_name(session_id: SessionId) -> String {
    format!("{}", ByteArrayWrapper(session_id.as_bytes()))
}

pub struct ByteArrayWrapper<'a>(pub &'a [u8]);

impl<'a> core::fmt::UpperHex for ByteArrayWrapper<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "0x")?
        }
        for byte in self.0 {
            write!(f, "{:02X}", &byte)?
        }
        Ok(())
    }
}

impl<'a> core::fmt::Display for ByteArrayWrapper<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(self, f)
    }
}
