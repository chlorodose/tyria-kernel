use core::alloc::GlobalAlloc;

pub struct PesudoAlloctor;
unsafe extern "C" {
    fn __non_exist_function();
}
unsafe impl GlobalAlloc for PesudoAlloctor {
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        unsafe { __non_exist_function() };
        todo!()
    }
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        unsafe { __non_exist_function() };
        todo!()
    }
}

#[cfg_attr(not(test), global_allocator)]
static __PA: PesudoAlloctor = PesudoAlloctor;
