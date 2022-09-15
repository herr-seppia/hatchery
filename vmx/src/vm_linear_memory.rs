// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cell::UnsafeCell;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::ptr;
use std::ptr::NonNull;
use wasmer::Tunables;
use wasmer_types::{MemoryType, Pages, TableType};
use wasmer_vm::{
    LinearMemory, MemoryError, MemoryStyle, TableStyle, VMMemory,
    VMMemoryDefinition, VMTable, VMTableDefinition,
};

const PAGE_SIZE: usize = 65536;
const TOTAL_PAGES: usize = 18;

/// A WASM linear memory.
#[derive(Debug)]
pub struct VMLinearMemory {
    file_opt: Option<File>,
    memory_definition: Option<UnsafeCell<VMMemoryDefinition>>,
}

unsafe impl Send for VMLinearMemory {} // todo: make sure this is acceptable
unsafe impl Sync for VMLinearMemory {}

impl VMLinearMemory {
    /// Creates a new copy-on-write WASM linear memory backed by a file at the
    /// given `path`.
    pub fn new<P: AsRef<Path>>(
        path: Option<P>,
        accessible_size: usize,
        mapping_size: usize,
    ) -> io::Result<Self> {
        let ptr: *mut std::ffi::c_void;
        let file_opt = match path {
            Some(file_path) => {
                if let Some(p) = file_path.as_ref().parent() {
                    std::fs::create_dir_all(p)?;
                }
                let file_path_exists = file_path.as_ref().exists();
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(!file_path_exists)
                    .open(file_path)?;
                if !file_path_exists {
                    file.set_len(accessible_size as u64)?;
                };
                ptr = unsafe {
                    libc::mmap(
                        ptr::null_mut(),
                        mapping_size,
                        libc::PROT_NONE,
                        libc::MAP_SHARED,
                        file.as_raw_fd(),
                        0,
                    )
                };
                Some(file)
            }
            None => {
                ptr = unsafe {
                    libc::mmap(
                        ptr::null_mut(),
                        mapping_size,
                        libc::PROT_NONE,
                        libc::MAP_PRIVATE | libc::MAP_ANON,
                        -1,
                        0,
                    )
                };
                None
            }
        };
        if ptr as isize == -1_isize {
            return Err(io::Error::last_os_error());
        }

        let mut ret = VMLinearMemory {
            memory_definition: None,
            file_opt,
        };
        ret.memory_definition = Some(UnsafeCell::new(VMMemoryDefinition {
            base: ptr as _,
            current_length: accessible_size,
        }));

        if accessible_size != 0 {
            // Commit the accessible size.
            ret.make_accessible(0, accessible_size)?;
        }

        Ok(ret)
    }

    /// Make the memory starting at `start` and extending for `len` bytes
    /// accessible. `start` and `len` must be native page-size multiples and
    /// describe a range within `self`'s reserved memory.
    #[cfg(not(target_os = "windows"))]
    pub fn make_accessible(
        &mut self,
        start: usize,
        len: usize,
    ) -> io::Result<()> {
        let page_size = region::page::size();
        assert_eq!(start & (page_size - 1), 0);
        assert_eq!(len & (page_size - 1), 0);

        if let Some(file) = &self.file_opt {
            if start > 0 {
                let new_len = (start + len) as u64;
                file.set_len(new_len)?;
            }
        }
        // Commit the accessible size.
        let vm_def_ptr = self.memory_definition.as_ref().unwrap().get(); //.base as *const u8;
        let ptr = unsafe { (*vm_def_ptr).base };
        let result = unsafe {
            region::protect(ptr.add(start), len, region::Protection::READ_WRITE)
        }
        .expect("region protection");
        // todo: make proper conversion from region error
        // to io error or use common error
        Ok(result)
    }
}

impl LinearMemory for VMLinearMemory {
    fn ty(&self) -> MemoryType {
        MemoryType {
            minimum: Pages::from(TOTAL_PAGES as u32),
            maximum: Some(Pages::from(TOTAL_PAGES as u32)),
            shared: false,
        }
    }
    fn size(&self) -> Pages {
        Pages::from(TOTAL_PAGES as u32)
    }
    fn style(&self) -> MemoryStyle {
        MemoryStyle::Static {
            bound: Pages::from(TOTAL_PAGES as u32),
            offset_guard_size: 0,
        }
    }
    fn grow(&mut self, delta: Pages) -> Result<Pages, MemoryError> {
        Err(MemoryError::CouldNotGrow {
            current: Pages::from(100u32),
            attempted_delta: delta,
        })
    }
    fn vmmemory(&self) -> NonNull<VMMemoryDefinition> {
        unsafe {
            NonNull::new(
                self.memory_definition
                    .as_ref()
                    .unwrap()
                    .get()
                    .as_mut()
                    .unwrap() as _,
            )
            .unwrap()
        }
    }
    fn try_clone(&self) -> Option<Box<dyn LinearMemory + 'static>> {
        None
    }
}

impl From<VMLinearMemory> for wasmer_vm::VMMemory {
    fn from(mem: VMLinearMemory) -> Self {
        Self(Box::new(mem))
    }
}

pub struct VMLinearTunables {
    path: PathBuf,
}

impl VMLinearTunables {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Tunables for VMLinearTunables {
    fn memory_style(&self, _memory: &MemoryType) -> MemoryStyle {
        MemoryStyle::Static {
            bound: Pages::from(TOTAL_PAGES as u32),
            offset_guard_size: 0,
        }
    }

    /// Construct a `TableStyle` for the provided `TableType`
    fn table_style(&self, _table: &TableType) -> TableStyle {
        TableStyle::CallerChecksSignature
    }

    fn create_host_memory(
        &self,
        _ty: &MemoryType,
        _style: &MemoryStyle,
    ) -> Result<VMMemory, MemoryError> {
        let memory = VMLinearMemory::new::<PathBuf>(
            None,
            TOTAL_PAGES * PAGE_SIZE,
            TOTAL_PAGES * PAGE_SIZE,
        )
        .unwrap();
        Ok(VMMemory::from_custom(memory))
    }

    unsafe fn create_vm_memory(
        &self,
        _ty: &MemoryType,
        _style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<VMMemory, MemoryError> {
        let memory = VMLinearMemory::new(
            Some(self.path.as_path()),
            TOTAL_PAGES * PAGE_SIZE,
            TOTAL_PAGES * PAGE_SIZE,
        )
        .unwrap();
        // now, it's important to update vm_definition_location with the memory
        // information!
        let mut ptr = vm_definition_location;
        let md = ptr.as_mut();
        let unsafecell = memory.memory_definition.as_ref().unwrap();
        let def = unsafecell.get().as_ref().unwrap();
        md.base = def.base;
        md.current_length = def.current_length;
        Ok(memory.into())
    }

    /// Create a table owned by the host given a [`TableType`] and a
    /// [`TableStyle`].
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<VMTable, String> {
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
        VMTable::from_definition(ty, style, vm_definition_location)
    }
}
