#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use core::panic;

use alloc::boxed::Box;
use alloc::vec;
use config::Config;
use elf::{load_elf, map_physical_memory, map_range};
use uefi::boot::exit_boot_services;
use uefi::{Status, entry};
use x86_64::registers::control::*;

use uefi::mem::memory_map::MemoryMap;
use x86_64::structures::paging::page::PageRange;
use ysos_boot::*;

mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    // 1. Load config
    let config = Config::parse(load_file(&mut open_file(CONFIG_PATH)));

    info!("Config: {:#x?}", config);
    let apps = if config.load_apps {
    info!("Loading apps...");
    Some(load_apps())
    } else {
        info!("Skip loading apps");
        None
    };
    // 2. Load ELF files
    let elf = xmas_elf::ElfFile::new(load_file(&mut open_file(config.kernel_path))).unwrap();

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap
    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is readonly, disable write protect (Cr0)
    unsafe {
        Cr0::update(|cr0| cr0.remove(Cr0Flags::WRITE_PROTECT));
    }

    // FIXME: map physical memory to specific virtual address offset
    let physical_memory_offset = config.physical_memory_offset;
    use elf::map_physical_memory;
    let mut frame_allocator = UEFIFrameAllocator;
    map_physical_memory(
        physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut frame_allocator,
    );

    // FIXME: load and map the kernel elf file
    match load_elf(
        &elf,
        physical_memory_offset,
        &mut page_table,
        &mut frame_allocator,
        false
    ) {
        Err(e) => panic!("Failed to load ELF: {:?}", e),
        _ => info!("Loaded ELF successfully"),
    }

    // FIXME: map kernel stack
    let (kernel_start_addr, kernel_size) = if config.kernel_stack_auto_grow == 0 {
        (config.kernel_stack_address, config.kernel_stack_size)
    } else {
        let kernrl_start_addr = config.kernel_stack_address
            + (config.kernel_stack_size - config.kernel_stack_auto_grow) * 0x1000;
        (kernrl_start_addr, config.kernel_stack_auto_grow)
    };
    use elf::map_range;
    match map_range(
        kernel_start_addr,
        kernel_size,
        &mut page_table,
        &mut frame_allocator,
        false
    ) {
        Ok(Range) => {
            info!("Mapped kernel stack: {:?}({:?})", Range.start, Range.end);
        }
        Err(e) => {
            panic!("Failed to map kernel stack: {:?}", e);
        }
    }
    // FIXME: recover write protect (Cr0)
    unsafe {
        Cr0::update(|cr0| cr0.insert(Cr0Flags::WRITE_PROTECT));
    }

    free_elf(elf);

    // 5. Pass system table to kernel
    let ptr = uefi::table::system_table_raw().expect("Failed to get system table");
    let system_table = ptr.cast::<core::ffi::c_void>();

    // 6. Exit boot and jump to ELF entry

    info!("Exiting boot services...");

    let mmap = unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA) };
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table,
        loaded_apps:apps
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}
