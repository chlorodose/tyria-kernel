use core::arch::asm;

use crate::page::{
    MaybePresentPageTableEntry, PageTablePage, PageType, PhyPageIndex, PresentPageTableEntry,
};

pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt", options(att_syntax, nomem, nostack)) };
    }
}

pub const PAGE_SIZE_BASE: usize = 4096;
pub const PAGE_TABLE_SIZE: usize = 512;

impl<const L: u8> From<PresentPageTableEntry<L>> for MaybePresentPageTableEntry<L> {
    fn from(value: PresentPageTableEntry<L>) -> Self {
        match value {
            PresentPageTableEntry::Table(index) => {
                MaybePresentPageTableEntry(
                    1
                | (index.0 << 12)
                | (1 << 1) // writeable
                | (1 << 2), // user
                )
            }
            PresentPageTableEntry::Leaf {
                index,
                global,
                ty,
                user,
                accessed,
                dirty,
            } => {
                let (writeable, unexecuteable, write_uncacheable, read_uncacheable, is_dma) =
                    match ty {
                        PageType::ReadOnly => (false, true, false, false, false),
                        PageType::Code => (false, false, false, false, false),
                        PageType::Data => (true, true, false, false, false),
                        PageType::Buffer => (true, true, false, true, false),
                        PageType::Reg => (true, true, true, true, false),
                        PageType::Dma => (true, true, true, true, true),
                    };
                match L {
                    0 => {
                        MaybePresentPageTableEntry(
                            1
                            | (index.0 << 12)
                            | (usize::from(writeable) << 1)
                            | (usize::from(user) << 2)
                            | (usize::from(write_uncacheable) << 3)
                            | (usize::from(read_uncacheable) << 4)
                            | (usize::from(accessed) << 5)
                            | (usize::from(dirty) << 6)
                            | (1 << 7) // PAT
                            | (usize::from(global) << 8)
                            | (usize::from(is_dma) << 9)
                            | (usize::from(unexecuteable) << 63),
                        )
                    }
                    1 | 2 => {
                        assert!(
                            (index.0
                                & ((1
                                    << (u32::from(L) * (PAGE_TABLE_SIZE.lowest_one().unwrap())))
                                    - 1))
                                == 0,
                            "huge page is not aligned"
                        );
                        MaybePresentPageTableEntry(
                            1 | (index.0 << 12)
                                | (usize::from(writeable) << 1)
                                | (usize::from(user) << 2)
                                | (usize::from(write_uncacheable) << 3)
                                | (usize::from(read_uncacheable) << 4)
                                | (usize::from(accessed) << 5)
                                | (usize::from(dirty) << 6)
                                | (1 << 7) // PS
                                | (1 << 12) // PAT
                                | (usize::from(global) << 8)
                                | (usize::from(is_dma) << 9)
                                | (usize::from(unexecuteable) << 63),
                        )
                    }
                    _ => panic!("page larger than 1GiB not exist on amd64"),
                }
            }
        }
    }
}
impl<const L: u8> From<isize> for MaybePresentPageTableEntry<L> {
    fn from(value: isize) -> Self {
        assert!(!value.is_negative(), "number cannot fit in PTE");
        MaybePresentPageTableEntry((usize::try_from(value).unwrap()) << 1)
    }
}
impl<const L: u8> TryFrom<MaybePresentPageTableEntry<L>> for PresentPageTableEntry<L> {
    type Error = isize;
    fn try_from(value: MaybePresentPageTableEntry<L>) -> Result<Self, Self::Error> {
        let value = value.0;
        if value & 1 == 0 {
            return Err(isize::try_from(value >> 1).unwrap());
        }

        let index = PhyPageIndex((value & ((1 << 52) - 1)) >> 12);

        Ok(match (L, (value & (1 << 7)) != 0) {
            (0, _) | (1 | 2, true) => {
                let writeable = (value & (1 << 1)) != 0;
                let user = (value & (1 << 2)) != 0;
                let unexecuteable = (value & (1 << 63)) != 0;
                let write_uncacheable = (value & (1 << 3)) != 0;
                let read_uncacheable = (value & (1 << 4)) != 0;
                let accessed = (value & (1 << 5)) != 0;
                let dirty = (value & (1 << 6)) != 0;
                let global = (value & (1 << 8)) != 0;
                let is_dma = (value & (1 << 9)) != 0;
                PresentPageTableEntry::Leaf {
                    index,
                    global,
                    ty: match (
                        writeable,
                        unexecuteable,
                        write_uncacheable,
                        read_uncacheable,
                        is_dma,
                    ) {
                        (false, true, false, false, false) => PageType::ReadOnly,
                        (false, false, false, false, false) => PageType::Code,
                        (true, true, false, false, false) => PageType::Data,
                        (true, true, false, true, false) => PageType::Buffer,
                        (true, true, true, true, false) => PageType::Reg,
                        (true, true, true, true, true) => PageType::Dma,
                        _ => panic!("unknow pattern of page attrs"),
                    },
                    user,
                    accessed,
                    dirty,
                }
            }
            _ => PresentPageTableEntry::Table(unsafe { PageTablePage::new_unchecked(index) }),
        })
    }
}
