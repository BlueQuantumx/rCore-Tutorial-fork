use core::arch::asm;

use super::{
    address::{PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
    page_table::{PTEFlags, PageTable, PageTableEntry},
};
use crate::{
    config::{MEMORY_END, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    memory::address::{PhysAddr, PAGE_SIZE},
};
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use log::trace;
use riscv::register::satp;
use spin::Mutex;

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

impl MemorySet {
    pub fn new_bare() -> Self {
        let page_table = PageTable::new();
        Self {
            page_table,
            areas: Vec::new(),
        }
    }

    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        trace!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        trace!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        trace!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        trace!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize,
            ebss as usize
        );
        trace!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        trace!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        trace!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        trace!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        trace!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        memory_set
    }

    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                trace!(
                    "mapping [{:#x}, {:#x}), offset={:#x}, file_size={:#x}",
                    usize::from(start_va),
                    usize::from(end_va),
                    ph.offset(),
                    ph.file_size()
                );
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.range.end;
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        trace!(
            "mapping user stack [{:#x}, {:#x})",
            user_stack_bottom,
            user_stack_top
        );
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        trace!("mapping TrapContext");
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}

impl MemorySet {
    pub fn activate(&self) {
        let (mode, asid, ppn) = self.page_table.satp_token();
        unsafe {
            satp::set(mode, asid.into(), ppn.into());
            asm!("sfence.vma");
        }
    }

    pub fn satp_token(&self) -> usize {
        let (mode, asid, ppn) = self.page_table.satp_token();
        ((((mode as usize) << 16) | asid) << 44) | ppn.0
    }

    fn map_trampoline(&mut self) {
        trace!("mapping trampoline");
        self.page_table
            .map(
                VirtAddr::from(TRAMPOLINE).page_number_floor(),
                PhysAddr::from(strampoline as usize).page_number_floor(),
                PTEFlags::R | PTEFlags::X,
            )
            .unwrap();
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_perm: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, map_perm),
            None,
        );
    }
    fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table).unwrap();
        if let Some(data) = data {
            area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(area);
    }
}

impl MemorySet {
    pub fn translate(&self, vpn: VirtPageNum) -> Result<&mut PageTableEntry, &'static str> {
        self.page_table.translate(vpn)
    }
}
struct MapArea {
    range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.page_number_floor();
        let end_vpn: VirtPageNum = end_va.page_number_ceil();
        Self {
            range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn map(&mut self, page_table: &mut PageTable) -> Result<(), &'static str> {
        for vpn in self.range.iter() {
            let ppn: PhysPageNum;
            let pte_flags = PTEFlags::from_bits_retain(self.map_perm.bits());
            match self.map_type {
                MapType::Identical => {
                    ppn = PhysPageNum(vpn.0);
                }
                MapType::Framed => {
                    let frame = frame_alloc().ok_or("Frame allocation failed")?;
                    ppn = frame.ppn;
                    self.data_frames.insert(vpn.into(), frame);
                }
            }
            page_table.map(vpn, ppn, pte_flags)?;
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn unmap(&mut self, page_table: &mut PageTable) -> Result<(), &'static str> {
        for vpn in self.range.iter() {
            match self.map_type {
                MapType::Identical => {}
                MapType::Framed => {
                    self.data_frames.remove(&vpn.into());
                }
            }
            page_table.unmap(vpn.into())?;
        }
        Ok(())
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.range.start;
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.0 += 1;
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub fn remap_test() {
    let kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_text.page_number_floor())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_rodata.page_number_floor())
            .unwrap()
            .writable(),
        false,
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_data.page_number_floor())
            .unwrap()
            .executable(),
        false,
    );
    println!("remap_test passed!");
}
