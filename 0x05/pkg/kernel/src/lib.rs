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

use boot::BootInfo;
use uefi::{mem::memory_map, runtime::ResetType, Status};
use core::time::Duration;

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

