#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(incomplete_features)]
#![feature(
    generic_const_exprs,
    inherent_associated_types,
    int_lowest_highest_one,
    isolate_most_least_significant_one,
    never_type,
    ptr_metadata
)]

use log::{error, info};

pub mod arch;
mod bootloader;
mod utils;
extern crate alloc;

#[cfg(feature = "qemu")]
mod qemu;

#[cfg_attr(not(test), panic_handler)]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("Panic: {info}");
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

/// Kernel's main entry
#[allow(clippy::missing_panics_doc, clippy::needless_pass_by_value)]
pub fn main(memory_map: impl DoubleEndedIterator<Item = MemoryEntry> + Clone + 'static) -> ! {
    #[cfg(feature = "qemu")]
    qemu::set_qemu_log();

    info!("Kernel starting...");
    let first_free = memory_map
        .clone()
        .find(|map| map.ty == MemoryType::Free)
        .expect("failed to find free memory");
    info!("First free area: {first_free:?}");
    panic!("")
}
