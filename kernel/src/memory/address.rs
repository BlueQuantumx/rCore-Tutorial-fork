const PAGE_OFFSET_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET_BITS;
const PA_WIDTH: usize = 56;
const PPN_WIDTH: usize = PA_WIDTH - PAGE_OFFSET_BITS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn page_number(&self) -> VirtPageNum {
        VirtPageNum(self.0 >> PAGE_OFFSET_BITS)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_OFFSET_BITS) - 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub fn page_number(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> PAGE_OFFSET_BITS)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_OFFSET_BITS) - 1)
    }
}

impl From<usize> for PhysAddr {
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << PA_WIDTH) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(ppn: PhysPageNum) -> Self {
        Self(ppn.0 << PAGE_OFFSET_BITS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(usize);

impl From<usize> for PhysPageNum {
    fn from(addr: usize) -> Self {
        Self(addr)
    }
}

impl From<PhysPageNum> for usize {
    fn from(addr: PhysPageNum) -> Self {
        addr.0
    }
}
