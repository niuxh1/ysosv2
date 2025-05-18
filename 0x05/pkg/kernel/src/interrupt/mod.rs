mod apic;
pub mod clock;
mod consts;
mod exceptions;
mod serial;
use crate::memory::address;
use crate::memory::physical_to_virtual;
use apic::*;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
pub mod syscall;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
            syscall::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();
    // FIXME: check and init APIC
    if let Some(_) = address::PHYSICAL_OFFSET.get() {
        if XApic::support() {
            let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
            lapic.cpu_init();
        }
    } else {
        // 如果PHYSICAL_OFFSET未初始化，记录警告信息
        warn!("PHYSICAL_OFFSET not initialized, skipping APIC initialization");
    }


    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(consts::Irq::Serial0 as u8, 0);

    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
