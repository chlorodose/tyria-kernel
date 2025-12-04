//! Manage memory paging
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::arch::{PAGE_SIZE_BASE, PAGE_TABLE_SIZE};

/// Page index for an physical page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhyPageIndex(pub usize);

/// An trait allow user to access an physical page by create an guard.
pub trait PhyPageAccessor {
    type Guard<T>: Deref<Target = MaybeUninit<T>> + DerefMut;
    /// Access the page as the type `T`.
    /// # Panics
    /// Panics if `T`'s is not fit where "fit" is implementation defined.
    fn access<T>(&self, index: PhyPageIndex) -> Self::Guard<T>;
}

#[derive(Debug, Clone)]
pub struct HhdmPhyPageAccessor(*mut ());
impl HhdmPhyPageAccessor {
    pub fn new(offset: *mut ()) -> Self {
        HhdmPhyPageAccessor(offset)
    }
}
pub struct HhdmPhyPageAccessorGuard<T>(*mut MaybeUninit<T>);
impl<T> Deref for HhdmPhyPageAccessorGuard<T> {
    type Target = MaybeUninit<T>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for HhdmPhyPageAccessorGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
impl PhyPageAccessor for HhdmPhyPageAccessor {
    type Guard<T> = HhdmPhyPageAccessorGuard<T>;
    fn access<T>(&self, index: PhyPageIndex) -> HhdmPhyPageAccessorGuard<T> {
        assert!(
            size_of::<T>() == PAGE_SIZE_BASE,
            "T is not one page in size"
        );
        HhdmPhyPageAccessorGuard(unsafe { self.0.cast::<MaybeUninit<T>>().add(index.0) })
    }
}

/// An page table entry which maybe not present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MaybePresentPageTableEntry<const L: u8>(pub usize);

/// The page's purpose type, defines the mapping attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageType {
    /// Data (ro)
    ReadOnly,
    /// Data (rw)
    Data,
    /// Code (execute)
    Code,
    /// Buffer (rw; write combine)
    Buffer,
    /// DMA (rw; uncacheable)
    Dma,
    /// Device Reg (rw; uncacheable; strong-ordering)
    Reg,
}

/// Pointer to an physical page who stores an page table(level `L - 1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageTablePage<const L: u8>(PhyPageIndex);
impl<const L: u8> Deref for PageTablePage<L> {
    type Target = PhyPageIndex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const L: u8> PageTablePage<L> {
    #[must_use]
    pub(crate) unsafe fn new_unchecked(index: PhyPageIndex) -> Self {
        assert_ne!(L, 0);
        Self(index)
    }
    /// Access page table.
    #[allow(clippy::implied_bounds_in_impls, clippy::missing_panics_doc)]
    pub fn access<A: PhyPageAccessor>(
        &self,
        accessor: &A,
    ) -> A::Guard<[MaybePresentPageTableEntry<{ L - 1 }>; PAGE_TABLE_SIZE]>
    where
        [(); { L - 1 } as usize]:,
    {
        accessor.access(self.0)
    }
}

/// An present page table entry.
/// `L` is the level of this PTE(`0` for leaves as example)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentPageTableEntry<const L: u8> {
    Table(PageTablePage<L>),
    Leaf {
        index: PhyPageIndex,
        global: bool,
        ty: PageType,
        user: bool,
        accessed: bool,
        dirty: bool,
    },
}
