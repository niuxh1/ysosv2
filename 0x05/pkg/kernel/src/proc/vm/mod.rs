use alloc::{format, string::String};
use boot::VirtualAddress;
use stack::{STACK_DEF_PAGE, STACK_INIT_BOT, STACK_MAX, STACK_MAX_PAGES};
use x86_64::{
    structures::paging::{page::*, *},
    VirtAddr,
};
use crate::proc::*;
use crate::proc::vm::mapper::UnmapError;
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

use super::{manager::{self, ProcessManager}, PageTableContext, ProcessId};

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
        }
    }
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
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