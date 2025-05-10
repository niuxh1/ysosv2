use crate::{memory::*, proc};
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt.divide_error.set_handler_fn(divide_error_handler);
    unsafe{
        idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt.page_fault
        .set_handler_fn(page_fault_handler)
        .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
    }
    // TODO: you should handle more exceptions here
    // especially gerneral protection fault (GPF)
    // see: https://wiki.osdev.org/Exceptions
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.bound_range_exceeded
        .set_handler_fn(bound_range_exceeded_handler);
    idt.cp_protection_exception
        .set_handler_fn(cp_protection_exception_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.hv_injection_exception
        .set_handler_fn(hv_injection_exception_handler);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.machine_check.set_handler_fn(machine_check_handler);
    idt.non_maskable_interrupt
        .set_handler_fn(non_maskable_interrupt);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.security_exception
        .set_handler_fn(security_exception_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.simd_floating_point
        .set_handler_fn(simd_floating_point_handler);
    idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
    idt.virtualization.set_handler_fn(virtualization_handler);
    idt.x87_floating_point
        .set_handler_fn(x87_floating_point_handler);
    idt.device_not_available
        .set_handler_fn(device_not_available_handler);
    idt.vmm_communication_exception
        .set_handler_fn(vmm_commumication_exception_handler);
}

pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    unsafe {
        *(0xffffff00000 as *mut u64) = 1234;
    }
    panic!(
        "EXCEPTION: DOUBLE FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    if !crate::proc::handle_page_fault(
        Cr2::read().unwrap_or(VirtAddr::new_truncate(0xdeadbeef)),
        err_code,
    ) {
        warn!(
            "EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\nTrying to access: {:#x}\n{:#?}",
            err_code,
            Cr2::read().unwrap_or(VirtAddr::new_truncate(0xdeadbeef)),
            stack_frame
        );
        // FIXME: print info about which process causes page fault?

        info!("Page fault occurred for process: {:?}", proc::manager::get_process_manager().current().pid());
    }
}
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
   
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: ALIGNMENT CHECK, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED \n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT \n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn cp_protection_exception_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: COPROCESSOR PROTECTION EXCEPTION, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}

pub extern "x86-interrupt" fn stack_segment_fault(stack_frame: InterruptStackFrame, err_code: u64) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn segment_not_present(stack_frame: InterruptStackFrame, err_code: u64) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: x87 FLOATING POINT\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn hv_injection_exception_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: HV INJECTION EXCEPTION\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, err_code: u64) {
    panic!(
        "EXCEPTION: INVALID TSS, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: SECURITY EXCEPTION, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn vmm_commumication_exception_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    panic!(
        "EXCEPTION: VMM COMMUNICATION EXCEPTION, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        err_code, stack_frame
    );
}