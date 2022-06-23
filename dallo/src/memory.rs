use core::mem::MaybeUninit;

pub(crate) struct Memory;

impl Memory {
    pub fn request<T>(n: usize) -> MaybeUninit<*mut T> {
        {
            #[cfg(feature = "host")]
            crate::host_mem::HostMem::request(n)
        }
        {
            #[cfg(not(feature = "host"))]
            crate::guest_mem::GuestMem::request(n)
        }
    }

    pub fn release<T: ?Sized>(ptr: *mut T) {
        let byteptr: *mut u8 = ptr.cast();
        {
            #[cfg(feature = "host")]
            crate::host_mem::HostMem::release(byteptr)
        }
        {
            #[cfg(not(feature = "host"))]
            crate::guest_mem::GuestMem::release(byteptr)
        }
    }
}
