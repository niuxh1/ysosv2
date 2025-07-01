#![no_std]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate libm;

#[macro_use]
pub mod utils;
pub use utils::regs;
pub use utils::*;
pub mod proc;
pub mod drivers;
pub mod interrupt;
pub mod memory;

use drivers::*;
use boot::BootInfo;
use uefi::{mem::memory_map, runtime::ResetType, Status};
use core::time::Duration;
use crate::ata::AtaDrive;
pub fn init(_boot_info: &'static BootInfo) {
    unsafe {
        uefi::table::set_system_table(_boot_info.system_table.cast().as_ptr());
    }
    drivers::serial::init();
    logger::init(Some("trace"));
    
    // 首先初始化内存子系统
    memory::address::init(_boot_info);
    memory::allocator::init();
    memory::gdt::init();
    memory::init(_boot_info);

    interrupt::init();
    proc::init(_boot_info);
    x86_64::instructions::interrupts::enable();
    info!("YatSenOS initialized.");
    drivers::filesystem::init();
    AtaDrive::open(0, 0);
     info!("Test stack grow.");

    grow_stack();

    info!("Stack grow test done.");
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn grow_stack() {
    const STACK_SIZE: usize = 1024 * 4;
    const STEP: usize = 64;

    let mut array = [0u64; STACK_SIZE];
    info!("Stack: {:?}", array.as_ptr());

    // test write
    for i in (0..STACK_SIZE).step_by(STEP) {
        array[i] = i as u64;
    }

    // test read
    for i in (0..STACK_SIZE).step_by(STEP) {
        assert_eq!(array[i], i as u64);
    }
}
pub fn shutdown() -> ! {
    info!("YatSenOS shutting down.");
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}

pub fn humanized_size(size: u64) -> (f64, &'static str) {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return (0.0, UNITS[0]);
    }

    let index = libm::floor(libm::log(size as f64) / libm::log(1024.0)) as usize;
    let index = index.min(UNITS.len() - 1);

    let converted_size = size as f64 / libm::pow(1024.0, index as f64);

    (converted_size, UNITS[index])
}

pub fn wait(init: proc::ProcessId) {
    loop {
        if proc::still_alive(init) {
            // Why? Check reflection question 5
            x86_64::instructions::hlt();
        } else {
            break;
        }
    }
}

