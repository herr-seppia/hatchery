use core::mem::MaybeUninit;

extern "C" {
    fn request(size: usize, align: usize) -> *mut u8;
    fn release(ptr: *mut u8);
}

pub(crate) struct GuestMem;

impl GuestMem {
    pub fn request<T>(n: usize) -> MaybeUninit<*mut T> {
        let size = core::mem::size_of::<T>() * n;
        let align = core::mem::align_of::<T>();
        let ptr = unsafe { request(size, align) };
        let t: *mut T = unsafe { core::mem::transmute(ptr) };
        MaybeUninit::new(t)
    }

    pub fn release<T>(ptr: *mut T) {
        let ptr: *mut u8 = unsafe { core::mem::transmute(ptr) };
        unsafe { release(ptr) }
    }
}
