use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc,task::Wake,boxed::Box};
use crossbeam_queue::ArrayQueue;
use core::task::{Context, Poll,Waker};
use x86_64::instructions::interrupts::{self, enable_interrupts_and_hlt};
use lazy_static::lazy_static;
use spin::{Mutex,RwLock};
use rcore_thread::{Context as CoreContext,context::Registers, std_thread as thread,scheduler,ThreadPool,Processor};

// to be able to add stuff to executor in a file add:
// use crate::task::{Task,executor::EXECUTOR};
// to it's header and run the async function like this:
// EXECUTOR.write().spawn(Task::new(FUNCTION_NAME()));

const STACK_SIZE: usize = 0x2000;
const MAX_CPU_NUM: usize = 1;
const MAX_PROC_NUM: usize = 32;

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    // when a task is woken it gets pushed onto the queue of tasks to run
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }

    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

// tasks are stored in a BTreeMap and accessed by their id's. The id's get put into a queue
pub struct Executor {
    tasks: BTreeMap<TaskId, Arc<Mutex<Task>>>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Arc<Mutex<Waker>>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, Arc::new(Mutex::new(task))).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    // starts executor
    pub fn run(& mut self) -> ! {
        let scheduler = scheduler::RRScheduler::new(5);
        let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_PROC_NUM));
        unsafe {
            processor().init(0, Thread::init(), thread_pool);
        }
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    // if the tasks queue is empty halts the cpu until a new task arrives
    fn sleep_if_idle(& self) {
        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_interrupts_and_hlt();
        } else {
            interrupts::enable();
        }
    }

    fn run_ready_tasks(& mut self) {
        // destructure `self` to avoid borrow checker errors
        // let Self {
        //     tasks,
        //     task_queue,
        //     waker_cache,
        // } = self;

        for _ in 0..self.tasks.len()-1{
            thread::spawn(||{
                // let executor = Arc::new(EXECUTOR.lock());
                if let Ok(task_id) = EXECUTOR.read().task_queue.pop(){
                    let task = match EXECUTOR.read().tasks.get(&task_id) {
                        Some(task) => Arc::clone(task),
                        None => panic!(), // task no longer exists
                    };
                    // gets waker from cache or creates waker if one doesn't exist
                    let waker = Arc::clone(EXECUTOR.write().waker_cache
                        .entry(task_id)
                        .or_insert_with(|| Arc::new(Mutex::new(TaskWaker::new(task_id, EXECUTOR.read().task_queue.clone())))));
                    let wakerlock = &waker.lock();
                    let mut context = Context::from_waker(wakerlock);
                    // runs task and removes it if it's finished
                    match task.lock().poll(&mut context) {
                        Poll::Ready(()) => {
                            EXECUTOR.write().tasks.remove(&task_id);
                            EXECUTOR.write().waker_cache.remove(&task_id);
                        }
                        Poll::Pending => {}
                    };
                }
            });
        }
        // polls tasks in queue until queue is empty
        // while let Ok(task_id) = task_queue.pop() {
        //     let task = match tasks.get_mut(&task_id) {
        //         Some(task) => task,
        //         None => continue, // task no longer exists
        //     };
        //     // gets waker from cache or creates waker if one doesn't exist
        //     let waker = waker_cache
        //         .entry(task_id)
        //         .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
        //     let mut context = Context::from_waker(waker);
        //     // runs task and removes it if it's finished
        //     match task.poll(&mut context) {
        //         Poll::Ready(()) => {
        //             tasks.remove(&task_id);
        //             waker_cache.remove(&task_id);
        //         }
        //         Poll::Pending => {}
        //     }
        // }
    }
}

lazy_static! {
    pub static ref EXECUTOR: RwLock<Executor> = RwLock::new(Executor::new());
}

struct Thread {
    rsp: *mut Registers,
    stack: [u8; STACK_SIZE],
}

impl Thread {
    unsafe fn init() -> Box<Self> {
        Box::new(core::mem::uninitialized())
    }
    fn new(entry: extern "C" fn(usize) -> !, arg0: usize) -> Box<Self> {
        let mut thread = unsafe { Thread::init() };
        let stack_top = thread.stack.as_ptr() as usize + STACK_SIZE;
        thread.rsp = unsafe { Registers::new(entry, arg0, stack_top) };
        thread
    }
}

/// Implement `switch_to` for a thread
impl CoreContext for Thread {
    /// Switch to another thread.
    unsafe fn switch_to(&mut self, target: &mut dyn CoreContext) {
        let (to, _): (&mut Thread, usize) = core::mem::transmute(target);
        Registers::switch(&mut self.rsp, &mut to.rsp);
    }
}

/// Define global `Processor` for each core.
static PROCESSORS: [Processor; MAX_CPU_NUM] = [Processor::new()];

/// Now we only have one core.
fn cpu_id() -> usize {
    0
}

/// Implement dependency for `rcore_thread::std_thread`
#[no_mangle]
pub fn processor() -> &'static Processor {
    &PROCESSORS[cpu_id()]
}

/// Implement dependency for `rcore_thread::std_thread`
#[no_mangle]
pub fn new_kernel_context(entry: extern "C" fn(usize) -> !, arg0: usize) -> Box<dyn CoreContext> {
    Thread::new(entry, arg0)
}
