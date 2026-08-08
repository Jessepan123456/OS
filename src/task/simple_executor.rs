use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// A simple asynchronous task executor.
pub struct SimpleExecutor {
    /// Queue containing tasks that are waiting to be polled
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    /// Creates a new empty 'SimpleExecutor'.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    /// Adds a task to the executor's task queue
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    /// Runns all tasks in the executor.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} //task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

/// Creates a dummy ['Waker']
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

/// Creates the raw waker used by ['dummy_waker']
fn dummy_raw_waker() -> RawWaker {
    /// Performs no operations
    fn no_op(_: *const ()) {}

    /// Creates another dummy 'RawWaker'
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // Create the vtable containing the functions used by the RawWaker
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    // Create the RawWaker with a null data pointer
    RawWaker::new(0 as *const (), vtable)
}
