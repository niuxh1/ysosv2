#![no_std]
#![no_main]

use lib::{sync::Semaphore, *};

extern crate lib;

const THREAD_COUNT: usize = 16;
const MAX_MESSAGE_COUNT: usize = 10;
const MQ_SIZE: usize = MAX_MESSAGE_COUNT + 1;

struct MessageQueue {
    queue: [usize; MQ_SIZE],
    head: usize,
    tail: usize,
}

static mut MQ: MessageQueue = MessageQueue {
    queue: [0; MQ_SIZE],
    head: 0,
    tail: 0,
};

static EMPTY: Semaphore = Semaphore::new(0);
static FULL: Semaphore = Semaphore::new(1);
static WRITE_MUTEX: Semaphore = Semaphore::new(2);

fn main() -> isize {
    let mut pids = [0u16; THREAD_COUNT];
    EMPTY.init(MAX_MESSAGE_COUNT);
    FULL.init(0);
    WRITE_MUTEX.init(1);

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();

        if i < THREAD_COUNT / 2 {
            if pid == 0 {
                for j in 0..MAX_MESSAGE_COUNT {
                    write_message(i + j);
                }
                sys_exit(0);
            } else {
                pids[i] = pid;
            }
        } else {
            if pid == 0 {
                for _ in 0..MAX_MESSAGE_COUNT {
                    read_message();
                }
                sys_exit(0);
            } else {
                pids[i] = pid;
            }
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("Message Queue: {:?}", unsafe { MQ.queue });

    0
}

fn write_message(message: usize) {
    unsafe {
        EMPTY.wait();
        WRITE_MUTEX.wait();
        MQ.queue[MQ.tail] = message;
        MQ.tail = (MQ.tail + 1) % MQ_SIZE;
        WRITE_MUTEX.signal();
        FULL.signal();
    }
}

fn read_message() {
    unsafe {
        FULL.wait();
        WRITE_MUTEX.wait();
        MQ.queue[MQ.head] = 0;
        MQ.head = (MQ.head + 1) % MQ_SIZE;
        WRITE_MUTEX.signal();
        EMPTY.signal();
    }
}

entry!(main);