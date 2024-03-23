use super::page_table::PageTableEntry;

const PAGE_OFFSET_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET_BITS;
const PA_WIDTH: usize = 56;
const PPN_WIDTH: usize = PA_WIDTH - PAGE_OFFSET_BITS;
const VA_WIDTH_SV48: usize = 48;
const VPN_WIDTH_SV48: usize = VA_WIDTH_SV48 - PAGE_OFFSET_BITS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

impl VirtAddr {
    pub fn page_number_floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 >> PAGE_OFFSET_BITS)
    }

    pub fn page_number_ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_OFFSET_BITS)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_OFFSET_BITS) - 1)
    }
}

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> Self {
        // to satisfy the mmu check
        let addr = addr & ((1 << VA_WIDTH_SV48) - 1);
        if addr >= (1 << (VA_WIDTH_SV48 - 1)) {
            Self(addr | !((1 << VA_WIDTH_SV48) - 1))
        } else {
            Self(addr)
        }
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(vpn: VirtPageNum) -> Self {
        (vpn.0 << PAGE_OFFSET_BITS).into() // to satisfy the mmu check
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 4] {
        let mut vpn = self.0;
        let mut idx = [0usize; 4];
        for i in (0..4).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

impl From<VirtPageNum> for usize {
    fn from(addr: VirtPageNum) -> Self {
        addr.0
    }
}

impl From<usize> for VirtPageNum {
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << VPN_WIDTH_SV48) - 1))
    }
}

pub struct VPNRange {
    pub start: VirtPageNum,
    pub end: VirtPageNum,
}

impl VPNRange {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { start, end }
    }
    pub fn iter(&self) -> impl Iterator<Item = VirtPageNum> {
        (self.start.0..self.end.0).map(|vpn| VirtPageNum(vpn))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub fn page_number_floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> PAGE_OFFSET_BITS)
    }

    pub fn page_number_ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_OFFSET_BITS)
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
pub struct PhysPageNum(pub usize);

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
    pub unsafe fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        (pa.0 as *mut T).as_mut().unwrap()
    }
}

impl From<usize> for PhysPageNum {
    fn from(ppn: usize) -> Self {
        Self(ppn & ((1 << PPN_WIDTH) - 1))
    }
}

impl From<PhysPageNum> for usize {
    fn from(addr: PhysPageNum) -> Self {
        addr.0
    }
}
