#![no_std]

pub use uefi::Status;
pub use uefi::boot::{MemoryAttribute, MemoryDescriptor, MemoryType};
pub use uefi::data_types::chars::*;
pub use uefi::data_types::*;
pub use uefi::proto::console::gop::{GraphicsOutput, ModeInfo};
use x86_64::structures::paging::Page;
use xmas_elf::program::ProgramHeader;
use arrayvec::ArrayVec;
use core::ptr::NonNull;
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::structures::paging::page::PageRangeInclusive;

pub type KernelPages = ArrayVec<PageRangeInclusive, 8>;
pub mod allocator;
pub mod config;
pub mod fs;

pub use allocator::*;
pub use fs::*;

#[macro_use]
extern crate log;

pub type MemoryMap = ArrayVec<MemoryDescriptor, 256>;
use arrayvec::ArrayString;
use xmas_elf::ElfFile;

/// App information
pub struct App<'a> {
    /// The name of app
    pub name: ArrayString<16>,
    /// The ELF file
    pub elf: ElfFile<'a>,
}
const APP_LIST_MAX:usize = 16;
pub type AppList = ArrayVec<App<'static>,APP_LIST_MAX>;
pub type AppListRef = Option<&'static ArrayVec<App<'static>, APP_LIST_MAX>>;

/// This structure represents the information that the bootloader passes to the kernel.
pub struct BootInfo {
    /// The memory map
    pub memory_map: MemoryMap,

    /// The offset into the virtual address space where the physical memory is mapped.
    pub physical_memory_offset: u64,

    /// The system table virtual address
    pub system_table: NonNull<core::ffi::c_void>,
    pub loaded_apps: Option<AppList>,
    pub kernel_pages: KernelPages,
}

/// Get current page table from CR3
pub fn current_page_table() -> OffsetPageTable<'static> {
    let p4_table_addr = Cr3::read().0.start_address().as_u64();
    let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
    unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
}

/// The entry point of kernel, set by BSP.
#[cfg(feature = "boot")]
static mut ENTRY: usize = 0;

/// Jump to ELF entry according to global variable `ENTRY`
///
/// # Safety
///
/// This function is unsafe because the caller must ensure that the kernel entry point is valid.
#[cfg(feature = "boot")]
pub fn jump_to_entry(bootinfo: *const BootInfo, stacktop: u64) -> ! {
    unsafe {
        assert!(ENTRY != 0, "ENTRY is not set");
        core::arch::asm!("mov rsp, {}; call {}", in(reg) stacktop, in(reg) ENTRY, in("rdi") bootinfo);
    }
    unreachable!()
}

/// Set the entry point of kernel
///
/// # Safety
///
/// This function is unsafe because the caller must ensure that the kernel entry point is valid.
#[inline(always)]
#[cfg(feature = "boot")]
pub fn set_entry(entry: usize) {
    unsafe {
        ENTRY = entry;
    }
}

/// This is copied from https://docs.rs/bootloader/0.10.12/src/bootloader/lib.rs.html
/// Defines the entry point function.
///
/// The function must have the signature `fn(&'static BootInfo) -> !`.
///
/// This macro just creates a function named `_start`, which the linker will use as the entry
/// point. The advantage of using this macro instead of providing an own `_start` function is
/// that the macro ensures that the function and argument types are correct.
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[unsafe(export_name = "_start")]
        pub extern "C" fn __impl_start(boot_info: &'static $crate::BootInfo) -> ! {
            // validate the signature of the program entry point
            let f: fn(&'static $crate::BootInfo) -> ! = $path;

            f(boot_info)
        }
    };
}
fn get_page_range(segment: &ProgramHeader) -> PageRangeInclusive {
    let start = segment.virtual_addr();
    let end = start + segment.mem_size() - 1;

    PageRangeInclusive {
        start: Page::containing_address(VirtAddr::new(start)),
        end: Page::containing_address(VirtAddr::new(end)),
    }
}
pub fn get_page_usage(elf: &ElfFile) -> KernelPages {
    elf.program_iter()
        .filter(|segment| segment.get_type() == Ok(xmas_elf::program::Type::Load))
        .map(|segment| get_page_range(&segment))
        .collect()
}