#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;
use ysos::drivers::input;
use crate::memory::allocator::init;


extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    init();
    loop {
        print!("> ");
        let input = input::get_line();

        match input.trim() {
            "exit" => break,
            _ => {
                
                println!("You said: {}", input);
                println!("The counter value is {}", interrupt::clock::read_counter());
            }
        }
    }

    ysos::shutdown();
}