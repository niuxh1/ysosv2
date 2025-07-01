use alloc::{format, string::String};
use boot::VirtualAddress;
use stack::{STACK_DEF_PAGE, STACK_INIT_BOT, STACK_MAX, STACK_MAX_PAGES};
use x86_64::{
    structures::paging::{page::*, *},
    VirtAddr,
};
use crate::proc::*;
use crate::proc::vm::mapper::UnmapError;
use crate::alloc::borrow::ToOwned;
use boot::KernelPages;
use crate::proc::vm::stack::STACK_MAX_SIZE;
use alloc::sync::Arc;
use crate::proc::Process;
use crate::proc::KERNEL_PID;
use crate::proc::vm::stack::STACK_DEF_SIZE;
use crate::{humanized_size, memory::*};
use crate::proc::vm::stack::STACK_INIT_TOP;
pub mod stack;
use xmas_elf::ElfFile;
use self::stack::Stack;
pub mod heap;
use crate::proc::vm::heap::Heap;
use super::{manager::{self, ProcessManager}, PageTableContext, ProcessId};
use x86_64::structures::paging::mapper::CleanUp;
type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,
    pub(super) heap: Heap,
    // stack is pre-process allocated
    pub(super) stack: Stack,
    pub(super) code: Vec<PageRangeInclusive>,
    pub(super) code_usage: u64,
}

impl ProcessVm {

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            heap: self.heap.fork(),
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
            code :Vec::new(),
            code_usage: 0,
        }
    }
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            heap:Heap::empty(),
            stack: Stack::empty(),
            code: Vec::new(),
            code_usage: 0,
        }
    }

  pub fn init_kernel_vm(mut self, pages: &KernelPages) -> Self {
    // FIXME: load `self.code` and `self.code_usage` from `pages`

    // FIXME: init kernel stack (impl the const `kstack` function)
    //        `pub const fn kstack() -> Self`
    //         use consts to init stack, same with kernel config
    self.stack = Stack::kstack();
    self.code = pages.iter().cloned().collect();
    self.code_usage = pages.iter().map(|range| range.count() as u64).sum();
    self
}

    pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
        // FIXME: calculate the stack for pid
        // FIXME: calculate the stack for pid
        let  page_table =&mut self.page_table.mapper();
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let addr=STACK_INIT_TOP -((pid.0 as u64 -1)*0x1_0000_0000);
        let addr_bot = STACK_INIT_BOT - ((pid.0 as u64 -1)*0x1_0000_0000);
        let is_user_access = pid != KERNEL_PID;
        elf::map_range(addr_bot, 1, page_table, frame_allocator,is_user_access).unwrap();
        self.stack = Stack::new(Page::containing_address(VirtAddr::new(addr)), STACK_DEF_PAGE);
        VirtAddr::new(addr)
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }
    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        
        self.stack.init(mapper, alloc);
        // FIXME: load elf to process pagetable
        elf::load_elf(elf, PHYSICAL_OFFSET.get().cloned().unwrap(), mapper, alloc,true).unwrap();
        
    }
    pub fn unmap(&mut self)-> Result<(), UnmapError>{
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();
        self.stack.unmap(mapper, alloc)?;
        Ok(())
    }
    pub fn stack_start(&self) -> VirtAddr {
        self.stack.range.start.start_address()
    }
    pub(super) fn clean_up(&mut self) -> Result<(), UnmapError> {
        let mapper = &mut self.page_table.mapper();
        let dealloc = &mut *get_frame_alloc_for_sure();

        // statistics for logging and debugging
        // NOTE: you may need to implement `frames_recycled` by yourself
        let start_count = dealloc.frames_recycled();

        // TODO...
        self.stack.clean_up(mapper, dealloc);
        if self.page_table.using_count() == 1{
            self.heap.clean_up(mapper, dealloc);
            for page_range in self.code.iter() {
                let start_addr = page_range.start.start_address().as_u64();
                let page_count = page_range.count() as u64;
                let range_start = Page::containing_address(VirtAddr::new(start_addr));
                let range_end = range_start + page_count;
                let page_range = Page::range(range_start, range_end);
                unsafe {elf::unmap_range( mapper, dealloc, page_range,true)?;}
            }
            unsafe {
                mapper.clean_up(dealloc);
                dealloc.deallocate_frame(self.page_table.reg.addr);
            }
        }

        // statistics for logging and debugging
        let end_count = dealloc.frames_recycled();

        debug!(
            "Recycled {}({:.3} MiB) frames, {}({:.3} MiB) frames in total.",
            end_count - start_count,
            ((end_count - start_count) * 4) as f32 / 1024.0,
            end_count,
            (end_count * 4) as f32 / 1024.0
        );

        Ok(())
    }
    pub fn brk(&self, addr: Option<VirtAddr>) -> Option<VirtAddr> {
        self.heap.brk(addr,&mut self.page_table.mapper(),&mut get_frame_alloc_for_sure())
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
impl Drop for ProcessVm {
    fn drop(&mut self) {
        if let Err(err) = self.unmap() {
            error!("Failed to clean up process memory: {:?}", err);
        }
    }
}