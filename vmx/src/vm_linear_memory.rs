// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use memmap2::{MmapMut, MmapOptions};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use wasmer::Tunables;
use wasmer_types::{MemoryType, Pages, TableType, WASM_PAGE_SIZE};
use wasmer_vm::{
    LinearMemory, MemoryError, MemoryStyle, TableStyle, VMMemory,
    VMMemoryDefinition, VMTable, VMTableDefinition,
};

const PAGE_SIZE: usize = 65536;
const ZERO_HASH: [u8; 32] = [0u8; 32];
const ZEROED_PAGE: [u8; PAGE_SIZE] = [0u8; PAGE_SIZE];

/// A WASM linear memory.
#[derive(Debug)]
pub struct VMLinearMemory {
    // mmap: MmapMut,
    // ptr: MmapPtr,
    mem: [u8; PAGE_SIZE*20],
}

// This allows `wasmer_vm::LinearMemory::vmmemory` to be implemented at the
// cost of a small overhead of two pointer lengths.
// #[derive(Debug, Clone, Copy)]
// #[repr(C)]
// struct MmapPtr {
//     base: *const u8,
//     len: usize,
// }

// this is safe because it always points to the base of the mmap, rather than to
// the `Memory` struct itself.
// unsafe impl Send for MmapPtr {}
// unsafe impl Sync for MmapPtr {}

// impl<'a> From<&'a MmapMut> for MmapPtr {
//     fn from(mmap: &'a MmapMut) -> Self {
//         Self {
//             base: mmap.as_ptr(),
//             len: mmap.len(),
//         }
//     }
// }

impl VMLinearMemory {
    // Creates a new copy-on-write WASM linear memory backed by a file at the
    // given `path`.
    //
    // This will create the file if it doesn't exist. If the file is smaller
    // than a WASM page it will extended and its contents zeroed.
    // pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    //     let mut file = OpenOptions::new()
    //         .create(true)
    //         .read(true)
    //         .write(true)
    //         .open(path)?;
    //
    //     if file.metadata()?.len() < PAGE_SIZE as u64 {
    //         file.set_len(PAGE_SIZE as u64)?;
    //         file.write_all(&ZEROED_PAGE)?;
    //     }
    //
    //     let mmap = unsafe { MmapOptions::new().map_copy(&file)? };
    //     let ptr = MmapPtr::from(&mmap);
    //
    //     Ok(Self { /*mmap, ptr,*/ mem: [0; WASM_PAGE_SIZE] })
    // }

    /// Creates a new anonymous WASM linear memory with an initial size of a
    /// WASM page.
    pub fn ephemeral() -> io::Result<Self> {
        // let mmap = MmapMut::map_anon(PAGE_SIZE)?;
        // let ptr = MmapPtr::from(&mmap);
        Ok(Self { /*mmap, ptr,*/ mem: [0; PAGE_SIZE*20] })
    }

    // Copies the current contents onto the file at the given `path`, replacing
    // the internal mmap by a new copy-on-write WASM backed by said file.
    //
    // The file will be truncated if it exists.
    // pub fn copy_to<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
    //     let mut file = OpenOptions::new()
    //         .create(true)
    //         .read(true)
    //         .write(true)
    //         .truncate(true)
    //         .open(path)?;
    //
    //     file.set_len(self.mmap.len() as u64)?;
    //     file.write_all(&self.mmap)?;
    //
    //     let new_mmap = unsafe { MmapOptions::new().map_copy(&file)? };
    //     let new_ptr = MmapPtr::from(&new_mmap);
    //
    //     self.mmap = new_mmap;
    //     self.ptr = new_ptr;
    //
    //     Ok(())
    // }
}

impl LinearMemory for VMLinearMemory {
    fn ty(&self) -> MemoryType {
        println!("**VMLinearMemory ty called");
        MemoryType {
            minimum: Pages::from(20u32),
            maximum: Some(Pages::from(20u32)),
            shared: false,
        }
    }

    fn size(&self) -> Pages {
        println!("**VMLinearMemory size called");
        // Pages((self.mmap.len() / WASM_PAGE_SIZE) as u32)
        Pages::from(20u32)
    }

    fn style(&self) -> MemoryStyle {
        println!("**VMLinearMemory style called");
        // MemoryStyle::Dynamic {
        //     offset_guard_size: 0,
        // }
        MemoryStyle::Static {
            bound: Pages::from(20u32),
            offset_guard_size: 0,
        }
    }

    fn grow(&mut self, delta: Pages) -> Result<Pages, MemoryError> {
        println!("**VMLinearMemory grow called");
        Err(MemoryError::CouldNotGrow {
            current: Pages::from(100u32),
            attempted_delta: delta,
        })
    }

    fn vmmemory(&self) -> NonNull<VMMemoryDefinition> {
        use std::cell::UnsafeCell;
        use wasmer_vm::MaybeInstanceOwned;
        println!("**VMLinearMemory vmmemory called");
        // let ptr = &self.ptr as *const MmapPtr;
        // let ptr = ptr as *mut VMMemoryDefinition;
        // NonNull::new(ptr).unwrap()
        MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(VMMemoryDefinition {
            base: self.mem.as_ptr() as _,
            current_length: PAGE_SIZE * 20,
        })))
        .as_ptr()
    }

    fn try_clone(&self) -> Option<Box<dyn LinearMemory + 'static>> {
        println!("**VMLinearMemory try_clone called");
        // TODO this could actually be implemented
        None
    }
}

// impl Deref for VMLinearMemory {
//     type Target = [u8];
//
//     fn deref(&self) -> &Self::Target {
//         &self.mmap
//     }
// }

// impl DerefMut for VMLinearMemory {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.mmap
//     }
// }

// impl AsRef<[u8]> for VMLinearMemory {
//     fn as_ref(&self) -> &[u8] {
//         &self.mmap
//     }
// }

// impl AsMut<[u8]> for VMLinearMemory {
//     fn as_mut(&mut self) -> &mut [u8] {
//         &mut self.mmap
//     }
// }

impl From<VMLinearMemory> for wasmer_vm::VMMemory {
    fn from(mem: VMLinearMemory) -> Self {
        Self(Box::new(mem))
    }
}

pub struct VMLinearTunables;
impl Tunables for VMLinearTunables {
    fn memory_style(&self, _memory: &MemoryType) -> MemoryStyle {
        println!("**VMLinearTunables memory_style called");
        MemoryStyle::Static {
            bound: Pages::from(20u32),
            offset_guard_size: 0,
        }
        // MemoryStyle::Dynamic {
        //     offset_guard_size: 0,
        // }
    }

    /// Construct a `TableStyle` for the provided `TableType`
    fn table_style(&self, _table: &TableType) -> TableStyle {
        println!("**VMLinearTunables table_style called");
        TableStyle::CallerChecksSignature
    }
    fn create_host_memory(
        &self,
        _ty: &MemoryType,
        _style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError> {
        println!("**VMLinearTunables create_host_memory called");
        let memory = VMLinearMemory::ephemeral().unwrap();
        Ok(VMMemory::from_custom(memory))
    }
    unsafe fn create_vm_memory(
        &self,
        _ty: &MemoryType,
        _style: &MemoryStyle,
        _vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError> {
        println!("**VMLinearTunables create_vm_memory called");
        let memory = VMLinearMemory::ephemeral().unwrap();
        Ok(memory.into())
    }

    /// Create a table owned by the host given a [`TableType`] and a
    /// [`TableStyle`].
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<VMTable, String> {
        println!("**VMLinearTunables create_host_table called");
        VMTable::new(ty, style)
    }

    /// Create a table owned by the VM given a [`TableType`] and a
    /// [`TableStyle`].
    ///
    /// # Safety
    /// - `vm_definition_location` must point to a valid location in VM memory.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<VMTable, String> {
        println!("**VMLinearTunables create_vm_table called");
        VMTable::from_definition(ty, style, vm_definition_location)
    }
}
