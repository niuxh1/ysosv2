use x86_64::{
    structures::paging::{mapper::MapToError, page::*, Page},
    VirtAddr,
};
use crate::proc::*;
use x86_64::structures::paging::mapper::UnmapError;
use super::{FrameAllocatorRef, MapperRef};
use core::ptr::copy_nonoverlapping;
// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x4000_0000_0000;
pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * crate::memory::PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_BOT: u64 = STACK_MAX - STACK_MAX_SIZE;
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;

const STACK_INIT_TOP_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(STACK_INIT_TOP));

// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_BOT: u64 = KSTACK_MAX - STACK_MAX_SIZE;
pub const KSTACK_DEF_PAGE: u64 =8;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * crate::memory::PAGE_SIZE;

pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

const KSTACK_INIT_PAGE: Page<Size4KiB> = Page::containing_address(VirtAddr::new(KSTACK_INIT_BOT));
const KSTACK_INIT_TOP_PAGE: Page<Size4KiB> =
    Page::containing_address(VirtAddr::new(KSTACK_INIT_TOP));

pub struct Stack {
    pub(super) range: PageRange<Size4KiB>,
    usage: u64,
}

impl Stack {

    pub fn fork(
        &self,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
        stack_offset_count: u64,
    ) -> Self {
        // FIXME: alloc & map new stack for child (see instructions)
        let mut child_stack_top = (self.range.start - stack_offset_count).start_address();
        let child_stack_count = self.usage;
        // FIXME: copy the *entire stack* from parent to child
        while elf::map_range(child_stack_top.as_u64(),
                            child_stack_count,
                            mapper,
                            alloc,
                            true
                            ).is_err()
        {
            trace!("Map thread stack to {:#x} failed.", child_stack_top);
            child_stack_top -= STACK_MAX_SIZE; // stack grow down
        }

        // FIXME: return the new stack
        self.clone_range(self.range.start.start_address().as_u64(),
                child_stack_top.as_u64(), 
                        child_stack_count
                        );
        let child_start = Page::containing_address(child_stack_top);
        let child_end = Page::containing_address(child_stack_top)+child_stack_count;
        let child_range = Page::range(child_start,child_end);
        Self {
            range:child_range,
            usage: child_stack_count
        }
    }
    
    /// Clone a range of memory
    ///
    /// - `src_addr`: the address of the source memory
    /// - `dest_addr`: the address of the target memory
    /// - `size`: the count of pages to be cloned
    fn clone_range(&self, cur_addr: u64, dest_addr: u64, size: u64) {
        trace!("Clone range: {:#x} -> {:#x}", cur_addr, dest_addr);
        unsafe {
            copy_nonoverlapping::<u64>(
                cur_addr as *mut u64,
                dest_addr as *mut u64,
                (size * Size4KiB::SIZE / 8) as usize,
            );
        }
    }
    pub fn new(top: Page, size: u64) -> Self {
        Self {
            range: Page::range(top - size + 1, top + 1),
            usage: size,
        }
    }

    pub const fn empty() -> Self {
        Self {
            range: Page::range(STACK_INIT_TOP_PAGE, STACK_INIT_TOP_PAGE),
            usage: 0,
        }
    }

    pub const fn kstack() -> Self {
        Self {
            range: Page::range(KSTACK_INIT_PAGE, KSTACK_INIT_TOP_PAGE),
            usage: KSTACK_DEF_PAGE,
        }
    }

    pub fn init(&mut self, mapper: MapperRef, alloc: FrameAllocatorRef) {
        debug_assert!(self.usage == 0, "Stack is not empty.");

        self.range = elf::map_range(STACK_INIT_BOT, STACK_DEF_PAGE, mapper, alloc,true).unwrap();
        self.usage = STACK_DEF_PAGE;
    }

    pub fn handle_page_fault(
        &mut self,
        addr: VirtAddr,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> bool {
        if !self.is_on_stack(addr) {
            return false;
        }

        if let Err(m) = self.grow_stack(addr, mapper, alloc) {
            error!("Grow stack failed: {:?}", m);
            return false;
        }
        if !self.is_on_stack(addr) {
            return false;
        }
        true
    }

    fn is_on_stack(&self, addr: VirtAddr) -> bool {
        let addr = addr.as_u64();
        let cur_stack_bot = self.range.start.start_address().as_u64();
        trace!("Current stack bot: {:#x}", cur_stack_bot);
        trace!("Address to access: {:#x}", addr);
        addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
    }

    fn grow_stack(
        &mut self,
        addr: VirtAddr,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Result<(), MapToError<Size4KiB>> {
        // debug_assert!(self.is_on_stack(addr), "Address is not on stack.");

        // FIXME: grow stack for page fault
        let addr_at_page = Page::<Size4KiB>::containing_address(addr);
        let start_page = self.range.start;
        let alloc_page_nums = start_page - addr_at_page;
        let original_page_size = self.range.end - start_page;

        let is_user_access = processor::get_pid() != KERNEL_PID;
        elf::map_range(
            addr_at_page.start_address().as_u64(),
            alloc_page_nums,
            mapper,
            alloc,
            is_user_access,
        )?;

        self.usage = original_page_size + alloc_page_nums;
        self.range = Page::range(addr_at_page, addr_at_page + self.usage);
        Ok(())
    }


    pub fn memory_usage(&self) -> u64 {
        self.usage * crate::memory::PAGE_SIZE
    }

    pub fn unmap(&mut self, mapper: MapperRef,
        alloc: FrameAllocatorRef
    )-> Result<(), UnmapError>{
        if self.usage == 0{
            info!("no memory to unmap");
        }
        let start = Page::containing_address(VirtAddr::new(self.range.start.start_address().as_u64(),));
        let end = start + self.range.count() as u64 - 1;
        unsafe{elf::unmap_range(mapper, alloc, Page::range(start,end), true)?;}
        Ok(())
    }
     pub fn clean_up(
        &mut self,
        // following types are defined in
        //   `pkg/kernel/src/proc/vm/mod.rs`
        mapper: MapperRef,
        dealloc: FrameAllocatorRef,
    ) -> Result<(), UnmapError> {
        if self.usage == 0 {
            warn!("Stack is empty, no need to clean up.");
            return Ok(());
        }

        // FIXME: unmap stack pages with `elf::unmap_pages`
        let deallocate_flag = processor::get_pid() != KERNEL_PID;
        let start_addr = self.range.start.start_address().as_u64();
        let range_start = Page::containing_address(VirtAddr::new(start_addr));
        let range_end = range_start + self.usage;
        let page_range = Page::range(range_start, range_end);
        unsafe{if let Err(e) = elf::unmap_range(mapper, dealloc, page_range, deallocate_flag) {
            debug!("Unmap stack failed: {:?}", e);
        }}
        self.usage = 0;

        Ok(())
    }

}

impl core::fmt::Debug for Stack {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Stack")
            .field(
                "top",
                &format_args!("{:#x}", self.range.end.start_address().as_u64()),
            )
            .field(
                "bot",
                &format_args!("{:#x}", self.range.start.start_address().as_u64()),
            )
            .finish()
    }
}