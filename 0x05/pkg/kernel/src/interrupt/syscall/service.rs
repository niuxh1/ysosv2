use core::alloc::Layout;
use crate::proc::*;
use crate::proc;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    let path = unsafe{
        core::str::from_utf8_unchecked(
            core::slice::from_raw_parts(
                args.arg0 as *const u8,args.arg1
            )
        )
    };
    match proc::spawn(path){
        Some(pid) => pid.0 as usize,
        None => 0
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    // FIXME: call proc::write -> isize
    // FIXME: return the result as usize
    // let buf =unsafe{core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2)};
    // proc::write(args.arg0,buf)
    let buf = unsafe{core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2)};
    proc::write(args.arg0 as u8, buf) as usize
    
}

pub fn sys_wait_pid(args: &SyscallArgs,context: &mut ProcessContext){
    let pid = ProcessId(args.arg0 as u16);
    proc::wait_pid(pid,context);

}

pub fn sys_get_pid() -> usize{
    proc::get_pid().0 as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let mut buf = unsafe{core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2)};
    proc::read(args.arg0 as u8, buf) as usize
    
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcod
    proc::exit(args.arg0 as isize, context);
}

pub fn list_process() {
    // FIXME: list all processes
    proc::print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_list_app() {
    // list all processes
    proc::list_app();
}

pub fn sys_time() -> u64 {
    let time = uefi::runtime::get_time().unwrap();
    time.hour() as u64 * 3600 + time.minute() as u64 * 60 + time.second() as u64
}

pub fn sys_fork(context: &mut ProcessContext) {
    proc::fork(context);
}

// op: u8, key: u32, val: usize -> ret: any
pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}