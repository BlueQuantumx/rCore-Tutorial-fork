use alloc::collections::BTreeMap;
use bitflags::*;

use super::{
    address::{PhysPageNum, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
};

struct PageTable {
    root_ppn: PhysPageNum,
    frames: BTreeMap<VirtAddr, FrameTracker>,
}

impl PageTable {
    pub fn new(root_ppn: PhysPageNum) -> Self {
        let frame = frame_alloc().unwrap();
        let mut page_table = Self {
            root_ppn,
            frames: BTreeMap::new(),
        };
        page_table.frames.insert(VirtAddr::from(0), frame);
        page_table
    }
    pub fn from_satp_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp),
            frames: BTreeMap::new(),
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &'static str> {
        let mut ppn = self.root_ppn;
        let mut result = Err("PageTableEntry not found");
        for (i, idx) in vpn.indexes().iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 3 {
                result = Ok(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().ok_or("PageTable frame allocation failed")?;
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.insert(0.into(), frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &'static str> {
        let mut ppn = self.root_ppn;
        let mut result = Err("PageTableEntry not found");
        for (i, idx) in vpn.indexes().iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 3 {
                result = Ok(pte);
                break;
            }
            if !pte.is_valid() {
                break;
            }
            ppn = pte.ppn();
        }
        result
    }
    pub fn map(
        &mut self,
        vpn: VirtPageNum,
        ppn: PhysPageNum,
        flags: PTEFlags,
    ) -> Result<(), &'static str> {
        let pte = self.find_pte_create(vpn)?;
        pte.is_valid()
            .then_some(())
            .ok_or("PageTableEntry already exists")?;
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        Ok(())
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) -> Result<(), &'static str> {
        let pte = self.find_pte(vpn)?;
        (!pte.is_valid())
            .then_some(())
            .ok_or("PageTableEntry not found")?;
        *pte = PageTableEntry::empty();
        Ok(())
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &'static str> {
        Ok(self.find_pte(vpn)?)
    }
    pub fn satp_token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

pub struct PageTableEntry {
    bits: usize,
}

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.bits >> 10)
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits as u8)
    }
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}
