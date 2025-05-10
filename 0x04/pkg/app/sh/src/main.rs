#![no_std]
#![no_main]

extern crate lib;
use lib::*;

fn main() -> isize {
    loop {
        print!("[>]输入指令： ");

        let binding = stdin().read_line();
        let mut command = binding.trim().split(' ');
        let op = command.next().unwrap();
        
        match op {
            "help" => {
                println!("23336197 牛渲淏");
                println!("la sys_list_app");
                println!("run /path/to/your/app");
                println!("ps sys_stat");
                println!("exit ");
            }
            "la" => {
                sys_list_app();
            }
            "run" => {
                let path = command.next().unwrap();
                let name: vec::Vec<&str> = path.rsplit('/').collect();
                let pid = sys_spawn(path);
                if pid == 0 {
                    println!("Failed to run app: {}", name[0]);
                    continue;
                } else {
                    sys_stat();
                    println!("{} exited with {}", name[0], sys_wait_pid(pid));
                }
            }
            "ps" => {
                sys_stat();
            }
            "exit" => {
                println!("Goodbye!");
                break;
            }

            
            _=> {
                println!("Unknown command: {}", op);
            }
        }
    }
    0
}

entry!(main);