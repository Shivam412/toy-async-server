use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc,
};

use super::task::Task;

// Struct representing a TaskQueue for managing asynchronous tasks
pub struct TaskQueue {
    sender: SyncSender<Arc<Task>>, // Sender for sending tasks to the queue
    receiver: Receiver<Arc<Task>>, // Receiver for receiving tasks from the queue
}

impl TaskQueue {
    // Constructor to create a new TaskQueue instance
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(1024); // Create a synchronous channel
        Self { sender, receiver } // Return the TaskQueue instance
    }

    // Function to send a task (wrapped in an Arc) to the queue
    pub fn send(&self, task: Arc<Task>) {
        self.sender.send(task).unwrap(); // Send the task to the queue (blocking if full)
    }

    // Function to get a clone of the sender for external use
    pub fn sender(&self) -> SyncSender<Arc<Task>> {
        self.sender.clone() // Return a clone of the sender
    }

    // Function to get a reference to the receiver for external use
    pub fn receiver(&self) -> &Receiver<Arc<Task>> {
        &self.receiver // Return a reference to the receiver
    }
}
