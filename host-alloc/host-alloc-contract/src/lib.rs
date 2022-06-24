#![no_std]
#![feature(lang_items, alloc_error_handler)]

extern crate alloc;

use host_alloc::HostAlloc;

use core::panic::PanicInfo;

use alloc::vec::Vec;

#[global_allocator]
static ALLOCATOR: HostAlloc = HostAlloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("{:?}", layout);
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn hello_world(n: u32) {
    let mut v = Vec::with_capacity(n as usize);

    for i in 0..n {
        v.push(i);
    }
}
