#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    lib::init();
    println!("Hello, world!!!");

    233
}

entry!(main);
