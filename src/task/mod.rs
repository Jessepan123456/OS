use core::{future::Future, pin::Pin, sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod keyboard;
pub mod executor;

/// Represents an asynchronous task
pub struct Task {
    /// The unique identifier for this task
    id: TaskId,

    /// The future beinig executed by this task.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Creates a new task from the given future.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Polls the task's future
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

/// A unique identifier for an asynchronous task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    /// Generates a new unique task ID.
    fn new() -> Self {
        // Global counter used to generate task IDs.
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        // Atomically obtain the current ID and increment the counter
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}