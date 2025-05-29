use super::*;
use alloc::sync::Arc;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::*, format};
use spin::{Mutex, RwLock};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {

    // FIXME: set init process as Running
    init.write().resume();

    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());
    
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    pub fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        let   now_proc=self.current();
        let mut now_proc_inner=now_proc.write();
        now_proc_inner.tick();
        // FIXME: save current process's context
        now_proc_inner.save(context);

    }

   // filepath: /home/niuxh/YatSenOS-Tutorial-Volume-2/src/0x03/pkg/kernel/src/proc/manager.rs
pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
    // 获取就绪队列并找到下一个可运行的进程
    let mut next_pid = {
        let mut queue = self.ready_queue.lock();
        
        // 从队列中查找合适的进程
        let mut selected_pid = None;
        
        // 最多检查队列中所有进程
        let queue_len = queue.len();
        for _ in 0..queue_len {
            if let Some(pid) = queue.pop_front() {
                // 检查进程状态是否有效
                if let Some(proc) = self.get_proc(&pid) {
                    if proc.read().status() != ProgramStatus::Dead {
                        // 找到合适的进程
                        selected_pid = Some(pid);
                        break;
                    }
                }
            } else {
                // 队列已空
                break;
            }
        }
        
        // 如果没有找到可运行进程，使用内核进程
        selected_pid.unwrap_or(KERNEL_PID)
    }; // 队列锁在这里被释放

    // 恢复选中进程的上下文
    let next_proc = self.get_proc(&next_pid).unwrap();
    {
        let mut next_proc_inner = next_proc.write();
        next_proc_inner.restore(context);
    } // 写锁在这里被释放

    // 更新处理器的当前PID
    processor::set_pid(next_pid);
    
    // 返回下一个进程的PID
    next_pid
}

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);
        let pid=proc.pid();
        // alloc stack for the new process base on pid
        
            
        let stack_top = proc.alloc_init_stack();
        // FIXME: set the stack frame
        proc.write().init_stack_frame(entry, stack_top);
        debug!("process status: {:#?}", proc);
        // FIXME: add to process map
        
        self.add_proc(proc.pid(), proc);
        // FIXME: push to ready queue
        self.push_ready(pid);
        // FIXME: return new process pid
        trace!("new process {:#?} has been spawned",pid);
        pid
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    // filepath: [manager.rs](http://_vscodecontentref_/3)
    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        let curr_proc = get_process_manager().current();
        if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION)
            && !err_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE)
        {
            return false;
        }
        // handle page fault in current process
        let ret = curr_proc.write().vm_mut().handle_page_fault(addr);
        ret
    }
    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output =
            String::from("  PID | PPID | Process Name |  Ticks  | Status | Memory Usage\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap
        let alloc: spin::MutexGuard<'_, memory::BootInfoFrameAllocator> = get_frame_alloc_for_sure();




        
        drop(alloc);

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
}
