#![cfg_attr(not(test), no_std)]

mod arch;
mod pollyfill;
extern crate alloc;

#[cfg_attr(not(test), panic_handler)]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    arch::halt();
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryType {
    Free,
    Claimed,
    Kernel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryEntry {
    pub area: *const [u8],
    pub ty: MemoryType,
}
#[allow(clippy::missing_panics_doc, clippy::needless_pass_by_value)]
pub fn main(memory_map: impl Iterator<Item = MemoryEntry> + Clone + 'static) -> ! {
    let first_free = memory_map
        .clone()
        .find(|map| map.ty == MemoryType::Free)
        .expect("failed to find free memory");
    panic!("")
}
