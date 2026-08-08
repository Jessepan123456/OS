use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::{task::{Context, Poll, Waker}};
use crossbeam_queue::ArrayQueue;

/// An asynchronous task executor
pub struct Executor {
    /// All currently active tasks
    tasks: BTreeMap<TaskId, Task>,
    
    /// Queue containing tasks that are ready to run
    task_queue: Arc<ArrayQueue<TaskId>>,

    /// Cached wwakers fro active tasks.
    waker_cache: BTreeMap<TaskId, Waker>,
}

/// A waker associated with a specific task.
struct TaskWaker {
    /// The ID of the task associated with this waker
    task_id: TaskId,

    /// A shared reference to the executor's task queue
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    /// Creates a new empty executor
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Adds a new task to the executor.
    /// 
    /// The task is inserted into the task map and then added to the
    /// ready queue so that it will be polled by the executor.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// Runs all tasks currently waiting in the ready queue.
    /// 
    /// Each task is removed from the queue and polled. If the task
    /// completes, it is removed from the executor. If the task returns 
    /// 'Poll::Pending', it remains in the executor and will be scheduled
    /// again when its waker is called. 
    fn run_ready_tasks(&mut self) {
        // destructure 'self' to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // Continue running while there are tasks in the ready queue
        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };

            // Get the cached waker for this task or create one if it
            // does not already exist
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            // Create a Context containing the task's waker
            let mut context = Context::from_waker(waker);
            
            // Poll the task to determine whether it has completed 
            // or needs to wait for something
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    /// Runs the executor indefinitely
    /// 
    /// Repeatedly processes ready tasks and then puts the CPU to sleep when there
    /// are no tasks ready to run.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// Puts the CPU to sleep when there are no ready tasks.
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        // Disable interrupts before checking the queue.
        interrupts::disable();

        if self.task_queue.is_empty() {
            // No task
            enable_and_hlt();
        } else {
            // task is ready
            interrupts::enable();
        }
    }
}

impl TaskWaker {
    /// Wakes the associated task by adding its ID to the task queue
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }

    /// Creates a new 'Waker' for the specified task.
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        let waker = Arc::new(TaskWaker {
            task_id,
            task_queue,
        });

        waker.into()
    }
}

impl Wake for TaskWaker {
    /// Wakes the task and consumes the 'Arc'
    /// 
    /// The task is added back to the executor's ready queue
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    /// Wakes the task without consuming the 'Arc'
    /// 
    /// This is useful when the waker needs to remain available after 
    /// waking the task.
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}