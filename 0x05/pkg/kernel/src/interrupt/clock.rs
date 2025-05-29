use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};
use crate::proc;
use crate::memory::gdt;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    unsafe{idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler).set_stack_index(gdt::CLOCK_IST_INDEX);}
}

pub extern "C" fn clock(mut context: proc::ProcessContext){
    
    x86_64::instructions::interrupts::without_interrupts(|| {
        proc::switch(&mut context);
        super::ack();
    });
}

as_handler!(clock);
