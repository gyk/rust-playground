use std::time::Instant;

pub trait Task: Send {
    fn execute(self);
}

pub struct DelayTask<T: Task> {
    pub task: T,
    /// Enqueue time
    pub time: Instant,
}

impl<T: Task> DelayTask<T> {
    pub fn new(task: T) -> Self {
        Self {
            task,
            time: Instant::now(),
        }
    }
}
