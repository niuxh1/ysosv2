use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::drivers::serial;
use crate::drivers::input::push_key;
use crate::drivers::input;
use crate::drivers::serial::get_serial_for_sure;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    let mut serial = get_serial_for_sure();
    let data = serial.receive();
    drop(serial);

    if let Some(data) = data {
        input::push_key(data);
    }
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
}