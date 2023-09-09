use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc::SyncSender, Arc, Mutex},
    task::{Wake, Waker},
};

// Define a type alias for a boxed Future that is Send and 'static
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Struct representing a Task for scheduling and managing asynchronous operations
pub struct Task {
    pub future: Mutex<BoxedFuture<'static, ()>>, // Mutex-wrapped boxed future
    pub sender: SyncSender<Arc<Task>>,           // Sender for sending tasks to the task queue
}

impl Task {
    // Function to schedule the task for execution
    pub fn schedule(self: &Arc<Self>) {
        self.sender.send(self.clone()).unwrap(); // Send a clone of the task to the task queue
    }

    // Function to create a Waker associated with the task
    pub fn waker(self: &Arc<Self>) -> Waker {
        Waker::from(self.clone()) // Create a Waker from a clone of the task
    }
}

// Implement the Wake trait for Task, allowing it to be woken up
impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.schedule(); // Wake up the task by scheduling it for execution
    }
}
