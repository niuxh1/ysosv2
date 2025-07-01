#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    
    println!("start to sleep");
    sleep(3);
    println!("sleep end");
    233
}

entry!(main);
