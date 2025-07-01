#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    lib::init();
    const HEAP_SIZE: usize = 8192;
    let heap_start = sys_brk(None).unwrap();
    let heap_end = heap_start + HEAP_SIZE;

    let ret = sys_brk(Some(heap_end)).expect("Failed to allocate heap");

    assert!(ret == heap_end, "Failed to allocate heap");
    0
}

entry!(main);