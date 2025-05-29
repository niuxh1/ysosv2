#![no_std]
#![no_main]

extern crate lib;
use lib::*;

fn main() -> isize {
    print!("\x1B[2J\x1B[H");
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const BLINK: &str = "\x1b[5m";
    const DIM: &str = "\x1b[2m";  // 暗淡效果，用于制造阴影
    const R1: &str = "\x1b[91m"; // 亮红
    const R2: &str = "\x1b[93m"; // 亮黄
    const R3: &str = "\x1b[92m"; // 亮绿
    const R4: &str = "\x1b[96m"; // 亮青
    const R5: &str = "\x1b[94m"; // 亮蓝
    const R6: &str = "\x1b[95m"; // 亮洋红
    const RAINBOW: [&str; 6] = [R1, R2, R3, R4, R5, R6];
    

    println!("\n\n");
    

    let banner = [
        " __   __      _  _____ ____  _____ _   _    ___  ____  ",
        " \\ \\ / /___ _| ||_   _/ ___|| ____| \\ | |  / _ \\/ ___| ",
        "  \\ V // _` | __|| | \\___ \\|  _| |  \\| | | | | \\___ \\ ",
        "   | || (_| | |_ | |  ___) | |___| |\\  | | |_| |___) |",
        "   |_| \\__,_|\\__||_| |____/|_____|_| \\_|  \\___/|____/ ",
    ];

    for (i, line) in banner.iter().enumerate() {

        print!("   {DIM}");
        for ch in line.chars() {
            if ch != ' ' {
                print!("█");
            } else {
                print!(" ");
            }
        }
        println!("{RESET}");

        print!("\x1B[1A\x1B[2C"); 
        let color = RAINBOW[i % RAINBOW.len()];
        println!("{BOLD}{color}{}{RESET}", line);
    }
    

    let info = "学号: 23336187   姓名: 牛渲淏";
    

    print!("   {DIM}");
    for ch in info.chars() {
        if ch != ' ' {
            print!("▓");
        } else {
            print!(" ");
        }
    }
    println!("{RESET}");
    
   
    print!("\x1B[1A\x1B[2C"); 
    for (i, ch) in info.chars().enumerate() {
        let color = RAINBOW[i % RAINBOW.len()];
        print!("{BOLD}{color}{}", ch);
    }
    println!("{RESET}\n\n"); 
    
    let welcome = "✨ 欢迎使用 YatSenOS 终端 ✨";

    print!("   {DIM}");
    for ch in welcome.chars() {
        if ch != ' ' {
            print!("▒");
        } else {
            print!(" ");
        }
    }
    println!("{RESET}");
    

    print!("\x1B[1A\x1B[2C"); 
    println!("{BOLD}{BLINK}{R4}{}{RESET}", welcome);
    

    let help_text = "输入 'help' 查看可用指令";
    print!("   ");
    for (i, ch) in help_text.chars().enumerate() {
        let color = RAINBOW[i % RAINBOW.len()];
        print!("{BOLD}{color}{}", ch);
    }
    println!("{RESET}\n");
    
    loop {

        print!("{DIM}▓▒░{RESET} {BOLD}{R3}[YatSenOS]{R4}> {RESET}");

        let binding = stdin().read_line();
        let mut command = binding.trim().split(' ');
        let op = command.next().unwrap();
        
        match op {
            "help" => {

                println!("\n{BOLD}{R4}〓〓〓〓 可用命令列表 〓〓〓〓{RESET}\n");

                let commands = [
                    ("la", "列出所有可用应用"),
                    ("run <路径>", "运行指定路径的应用程序"),
                    ("ps", "显示系统状态"),
                    ("clear", "清屏"),
                    ("exit", "退出终端")
                ];
                
                for (idx, (cmd, desc)) in commands.iter().enumerate() {
                    let color1 = RAINBOW[idx % RAINBOW.len()];
                    let color2 = RAINBOW[(idx + 2) % RAINBOW.len()];
                    

                    println!("  {DIM}▓▒░ {}▒▒▒▒▒▒▒▒▒{RESET}", cmd);
                    

                    print!("\x1B[1A\x1B[1C");
                    println!(" {BOLD}{color1}▶ {}{RESET}  {color2}{}{RESET}", cmd, desc);
                }
                println!();
            }
            "la" => {
                sys_list_app();
            }
            "run" => {
                let path = command.next().unwrap();
                let name: vec::Vec<&str> = path.rsplit('/').collect();
                let pid = sys_spawn(path);
                if pid == 0 {
                    println!("{BOLD}{R1}⚠ Failed to run app: {}{RESET}", name[0]);
                    continue;
                } else {
                    sys_stat();
                    println!("{BOLD}{R3}✓ {} exited with {}{RESET}", name[0], sys_wait_pid(pid));
                }
            }
            "ps" => {
                println!("{BOLD}{R4}〓〓〓 系统状态 〓〓〓{RESET}");
                sys_stat();
            }
            "exit" => {
                let goodbye = "Goodbye! See you next time!";
                for (i, ch) in goodbye.chars().enumerate() {
                    let color = RAINBOW[i % RAINBOW.len()];
                    print!("{BOLD}{color}{}", ch);
                    for _ in 0..3000000 { /* 简单的延迟 */ }
                }
                println!("{RESET}");
                break;
            }
            "ls" =>{
                  sys_list_dir(command.next().unwrap_or("/"));
            }
            "cat" => {
                let fd = sys_open_file(command.next().unwrap_or(""));
                let buf = &mut [0u8; 1024];
                sys_read(fd, buf);
                println!(
                    "{}",
                    core::str::from_utf8(buf).unwrap_or("Failed to read file")
                );
                sys_close_file(fd);
            }
            "clear" => {
                print!("\x1B[2J\x1B[H"); 
            }
            _ => {
                println!("{BOLD}{R1}⚠ Unknown command: {}{RESET}", op);
            }
        }
    }
    0
}

entry!(main);