#![no_std]

use core::alloc::{GlobalAlloc, Layout};

mod external {
    use super::*;

    extern "C" {
        pub fn alloc(layout: HostAllocLayout) -> *mut u8;
        pub fn dealloc(ptr: *mut u8, layout: HostAllocLayout);
    }
}

pub struct HostAlloc;

unsafe impl GlobalAlloc for HostAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        external::alloc(layout.into())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        external::dealloc(ptr, layout.into())
    }
}

#[repr(C)]
pub struct HostAllocLayout {
    size: u32,
    align: u32,
}

impl From<Layout> for HostAllocLayout {
    fn from(layout: Layout) -> Self {
        Self {
            size: layout.size() as u32,
            align: layout.align() as u32,
        }
    }
}
