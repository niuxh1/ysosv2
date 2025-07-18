use super::*;
use crate::memory::*;
use alloc::sync::Weak;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::*;
use crate::proc::vm::ProcessVm;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use crate::utils::humanized_size;

use crate::proc::vm::stack::STACK_MAX_PAGES;

#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    context: ProcessContext,
    exit_code: Option<isize>,
    proc_data: Option<ProcessData>,
    proc_vm: Option<ProcessVm>,
    
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        proc_vm: Option<ProcessVm>,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();
        let proc_vm = proc_vm.unwrap_or_else(|| ProcessVm::new(PageTableContext::new()));

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            proc_vm: Some(proc_vm),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
        // consume the Option<ProcessVm> and drop it
    }

    pub fn alloc_init_stack(&self) -> VirtAddr {
        trace!("Allocating stack for process {}#{}", self.read().name(), self.pid);
        self.write().vm_mut().init_proc_stack(self.pid)
    }


    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // FIXME: lock inner as write
        let mut  inner = self.inner.write();
        // FIXME: inner fork with parent weak ref
        let child_inner = inner.fork(Arc::downgrade(self));
        // FOR DBG: maybe print the child process info
        //          e.g. parent, name, pid, etc.
        let child_pid = ProcessId::new();
        trace!(
            "Parent {} forked: {}#{}",
            inner.name,
            child_pid,
            child_inner.name
        );
        // FIXME: make the arc of child
        let child_proc = Arc::new(Self{
            pid:child_pid,
            inner: Arc::new(RwLock::new(child_inner))
        });
        // FIXME: add child to current process's children list
        inner.children.push(child_proc.clone());
        // FIXME: set fork ret value for parent with `context.set_rax`
        inner.context.set_rax(child_pid.0 as usize);
        // FIXME: mark the child as ready & return it
        child_proc.inner.write().pause();
        return child_proc;
    }

}

impl ProcessInner {
    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr){
        self.context.init_stack_frame(entry, stack_top);
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn block(&mut self){
        self.status =  ProgramStatus::Blocked;
    }
    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.proc_vm.as_ref().unwrap().page_table.clone_level_4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    pub fn vm(&self) -> &ProcessVm {
        self.proc_vm.as_ref().unwrap()
    }

    pub fn vm_mut(&mut self) -> &mut ProcessVm {
        self.proc_vm.as_mut().unwrap()
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        self.vm_mut().handle_page_fault(addr)
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context);
        if self.status == ProgramStatus::Running {
            self.pause();
        }
        
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        // FIXME: restore the process's page table
        self.vm().page_table.load();
        self.resume();
    }


    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        self.proc_vm.take();
        self.proc_data.take();
    }
    // FIXME: load elf to process pagetable
    pub fn load_elf(&mut self , elf: &ElfFile){
        self.vm_mut().load_elf(elf)
    }


    pub fn fork(&mut self, parent: Weak<Process>) -> ProcessInner {
             
        // FIXME: calculate the real stack offset
        let stack_offset_count = ((self.children.len()+1) as u64)*STACK_MAX_PAGES;
        // FIXME: fork the process virtual memory struct   
        let child_vm = self.proc_vm.as_ref().unwrap().fork(stack_offset_count);
        
        // FIXME: update `rsp` in interrupt stack frame
        let mut  children_context: ProcessContext = self.context;
        let child_stack_top = (self.context.stack_top() & 0xFFFFFFFF)
            | (child_vm.stack_start().as_u64() & !(0xFFFFFFFF));
        children_context.update_rsp(child_stack_top);
        // FIXME: set the return value 0 for child with `context.set_rax`
        children_context.set_rax(0);
        // FIXME: clone the process data struct
        let child_data = self.proc_data.clone();
        // FIXME: construct the child process inner
        let child_inner = Self { name: self.name.clone(),
                             parent: Some(parent),
                             children: Vec::new(),
                             ticks_passed: 0, 
                             status:ProgramStatus::Ready ,
                             context: children_context,
                             exit_code: None,
                             proc_data: child_data,
                             proc_vm: Some(child_vm) };
        
        
        // NOTE: return inner because there's no pid record in inner
        return child_inner;
    }
    pub fn set_rax(&mut self,ret:usize){
        self.context.set_rax(ret);
    }
    pub fn sem_wait(&mut self, key: u32, pid: ProcessId) -> SemaphoreResult {
        self.proc_data.as_mut().unwrap().sem_wait(key, pid)
    }
    pub fn sem_signal(&mut self, key: u32) -> SemaphoreResult {
        self.proc_data.as_mut().unwrap().sem_signal(key)
    }

    pub fn new_sem(&mut self, key: u32, value: usize) -> bool {
        self.proc_data.as_mut().unwrap().new_sem(key, value)
    }

    pub fn remove_sem(&mut self, key: u32) -> bool {
        self.proc_data.as_mut().unwrap().remove_sem(key)
    }

    pub fn open_file(&mut self, path: &str) -> u8 {
        self.proc_data.as_mut().unwrap().open_file(path)
    }
    pub fn brk(&self,addr: Option<VirtAddr>) -> Option<VirtAddr>{
        self.proc_vm.as_ref().unwrap().brk(addr)
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}


impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("name", &inner.name)
            .field("parent", &inner.parent().map(|p| p.pid))
            .field("status", &inner.status)
            .field("ticks_passed", &inner.ticks_passed)
            .field("children", &inner.children.iter().map(|c| c.pid.0))
            .field("status", &inner.status)
            .field("context", &inner.context)
            .field("vm", &inner.proc_vm)
            .finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        let (size, unit) = humanized_size(inner.proc_vm.as_ref().map_or(0, |vm| vm.memory_usage()));
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:>5.1} {} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            size,
            unit,
            inner.status
        )?;
        Ok(())
    }
}