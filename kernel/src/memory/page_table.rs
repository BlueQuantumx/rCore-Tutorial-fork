use alloc::vec::Vec;
use alloc::{string::String, vec};
use bitflags::*;
use riscv::register::satp;

use super::address::PAGE_SIZE;
use super::{
    address::{PhysPageNum, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
};

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        let page_table = Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        };
        page_table
    }
    pub fn from_satp_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp),
            frames: vec![],
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
                self.frames.push(frame);
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
        (!pte.is_valid())
            .then_some(())
            .ok_or("PageTableEntry already exists")?;
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        Ok(())
    }
    #[allow(dead_code)]
    pub fn unmap(&mut self, vpn: VirtPageNum) -> Result<(), &'static str> {
        let pte = self.find_pte(vpn)?;
        pte.is_valid()
            .then_some(())
            .ok_or("PageTableEntry not found")?;
        *pte = PageTableEntry::empty();
        Ok(())
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &'static str> {
        Ok(self.find_pte(vpn)?)
    }
    pub fn satp_token(&self) -> (satp::Mode, usize, PhysPageNum) {
        (satp::Mode::Sv48, 0, self.root_ppn)
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

/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_satp_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.page_number_floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.0 = vpn.0 + 1;
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_satp_token(token);
    let mut start_va = VirtAddr::from(ptr as usize);
    let mut s = String::new();
    loop {
        let vpn = start_va.page_number_floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        let mut start_offset = start_va.page_offset();
        while start_offset < PAGE_SIZE {
            let ch = ppn.get_bytes_array()[start_offset];
            if ch == 0 {
                return s;
            }
            s.push(ch as char);
            start_offset += 1;
        }
        start_va = VirtPageNum::from(vpn.0 + 1).into();
    }
}
