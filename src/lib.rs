#![cfg_attr(not(test), no_std)]
mod pollyfill;
extern crate alloc;

#[cfg_attr(not(test), panic_handler)]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
