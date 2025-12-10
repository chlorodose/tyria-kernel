use core::{
    alloc::GlobalAlloc,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::arch::halt;

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

#[cfg_attr(not(test), unsafe(export_name = "__default_entry"))]
extern "C" fn default_start_kernel() -> ! {
    #[cfg(feature = "qemu")]
    qemu_print::qemu_println!("You enter the wrong entry(the default one)!");
    halt(); // wrong entry
}

#[cfg_attr(not(test), global_allocator)]
static __PA: PesudoAlloctor = PesudoAlloctor;

/// An guard execute an closure on drop.
/// Optionally with args, you can visit the arg's ref(mut) via this guard.
#[derive(Debug)]
pub struct DeferGuard<A, F: FnOnce(A)>(ManuallyDrop<A>, Option<F>);
impl<A, F: FnOnce(A)> DeferGuard<A, F> {
    /// Create an guard with args an closure.
    pub const fn new(args: A, f: F) -> Self {
        Self(ManuallyDrop::new(args), Some(f))
    }
    /// Foget about the closure, do not execute it anymore.
    pub fn forget(&mut self) {
        self.1.take();
    }
}
impl<A, F: FnOnce(A)> Deref for DeferGuard<A, F> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A, F: FnOnce(A)> DerefMut for DeferGuard<A, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<A, F: FnOnce(A)> Drop for DeferGuard<A, F> {
    fn drop(&mut self) {
        if let Some(f) = self.1.take() {
            f(unsafe { ManuallyDrop::take(&mut self.0) });
        } else {
            unsafe { ManuallyDrop::drop(&mut self.0) };
        }
    }
}
