use crate::{MemoryEntry, MemoryType, main};
use core::ptr::{from_raw_parts, without_provenance, without_provenance_mut};
use limine::request::{EntryPointRequest, HhdmRequest, MemoryMapRequest};

#[used]
#[unsafe(link_section = ".limine")]
static MEMORY_MAP_REQ: MemoryMapRequest = MemoryMapRequest::new();
#[used]
#[unsafe(link_section = ".limine")]
static HHDM_OFFSET_REQ: HhdmRequest = HhdmRequest::new();
#[used]
#[unsafe(link_section = ".limine")]
static ENTRY_REQ: EntryPointRequest = EntryPointRequest::new().with_entry_point(start_kernel);

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
    main(
        without_provenance_mut(
            HHDM_OFFSET_REQ
                .get_response()
                .expect("failed to get hhdm response")
                .offset()
                .try_into()
                .unwrap(),
        ),
        iter,
    );
}
