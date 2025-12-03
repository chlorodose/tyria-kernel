#![no_std]
#![no_main]
#![feature(ptr_metadata)]
use core::ptr::{from_raw_parts, without_provenance};
use limine::request::MemoryMapRequest;
use tyria_kernel::{MemoryEntry, MemoryType, main};

#[used]
static MEMORY_MAP_REQ: MemoryMapRequest = MemoryMapRequest::new();

#[unsafe(export_name = "_start")]
extern "C" fn start_kernel() -> ! {
    let iter = MEMORY_MAP_REQ
        .get_response()
        .expect("failed to get memory map reponse")
        .entries()
        .iter()
        .map(|map| MemoryEntry {
            area: from_raw_parts(
                without_provenance::<()>(map.base.try_into().unwrap()),
                map.length.try_into().unwrap(),
            ),
            ty: match map.entry_type {
                limine::memory_map::EntryType::USABLE => MemoryType::Free,
                limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => MemoryType::Kernel,
                _ => MemoryType::Claimed,
            },
        });
    main(iter);
}
