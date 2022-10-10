// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct MemoryPath {
    path: PathBuf,
}

impl MemoryPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        MemoryPath { path: path.into() }
    }
}

impl AsRef<Path> for MemoryPath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}
