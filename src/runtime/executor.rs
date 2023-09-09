use std::future::Future;
use std::sync::Mutex;
use std::task::Poll::{Pending, Ready};
use std::task::{Context, Waker};
use std::{cell::RefCell, sync::Arc};

use super::reactor::REACTOR;
use super::task::Task;
use super::task_queue::TaskQueue;
use crate::core::result::Result;

// Define a thread-local variable to hold the Executor instance
thread_local! {
    static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::new());
}

// Function to block the current thread and run a Future to completion
pub fn block_on<F>(f: F) -> Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    EXECUTOR.with(|executor| -> Result<()> {
        let executor = executor.borrow();
        executor.spawn(f); // Spawn the Future onto the Executor
        executor.run() // Run the Executor to completion
    })
}

// Function to spawn a Future onto the Executor
pub fn spawn<F>(f: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        executor.spawn(f); // Spawn the Future onto the Executor
    });
}

// Struct representing an asynchronous task Executor
pub struct Executor {
    tasks: TaskQueue, // Queue to hold tasks (Futures)
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: TaskQueue::new(), // Initialize the task queue
        }
    }

    // Function to spawn a Future onto the Executor
    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.send(Arc::new(Task {
            future: Mutex::new(Box::pin(f)), // Wrap the Future in a Mutex for synchronization
            sender: self.tasks.sender().clone(),
        }));
    }

    // Function to run the Executor and process tasks
    pub fn run(&self) -> Result<()> {
        loop {
            // Process tasks from the queue and dispatch them
            while let Ok(task) = self.tasks.receiver().try_recv() {
                let waker = task.waker();
                let mut cx = Context::from_waker(&waker);
                println!("[Ex] Received Task polling Future ...");
                match task.future.lock().unwrap().as_mut().poll(&mut cx) {
                    Ready(_) => {
                        println!("[Ex] Poll ready complete on spawned task");
                    }
                    Pending => {}
                }
            }

            // Wait for I/O events from the reactor
            self.wait_for_io()?;
        }
    }

    // Function to wait for I/O events by interacting with the reactor
    pub fn wait_for_io(&self) -> Result<()> {
        println!("[wait_for_io] Waiting for the I/O events.");
        REACTOR.with(|current| -> Result<()> {
            let wakers: Vec<Waker> = {
                let mut current = current.borrow_mut();
                current.poll_wait()? // Poll for I/O events and get associated wakers
            };
            for waker in wakers {
                waker.wake(); // Wake up the associated tasks
            }

            Ok(())
        })
    }
}
