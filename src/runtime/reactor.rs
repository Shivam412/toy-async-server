use crate::core::result::Result;
use crate::runtime::polling::epoll;

use std::{
    cell::RefCell,
    collections::HashMap,
    os::fd::RawFd,
    task::{Context, Waker},
};

// Define a thread-local variable to hold the Reactor instance
thread_local! {
    pub static REACTOR: RefCell<Reactor> = RefCell::new(Reactor::new());
}

// Struct representing a Reactor for handling asynchronous I/O events
pub struct Reactor {
    poller: epoll::Poller,         // The underlying epoll-based event poller
    wakers: HashMap<RawFd, Waker>, // A map to associate file descriptors with task wakers
}

impl Reactor {
    // Constructor to create a new Reactor instance
    pub fn new() -> Self {
        Self {
            poller: epoll::Poller::new().unwrap(), // Initialize the epoll-based poller
            wakers: HashMap::new(),                // Initialize the map for wakers
        }
    }

    // Function to wait for and retrieve I/O events from the poller
    pub fn poll_wait(&mut self) -> Result<Vec<Waker>> {
        let events = self.poller.wait()?; // Wait for events and get a list of event descriptors
        let mut wakers: Vec<Waker> = Vec::new();

        for event in events {
            // Wake up corresponding task wakers associated with the event descriptor
            if let Some(waker) = self.wakers.remove(&event.key) {
                wakers.push(waker);
            }
        }

        Ok(wakers)
    }

    // Function to register a file descriptor with specified events for polling
    pub fn register(&self, key: RawFd, events: i32) -> Result<()> {
        self.poller
            .add(key, (libc::EPOLLONESHOT | libc::EPOLLET | events) as u32)
    }

    // Function to modify the events for an already registered file descriptor
    // and associate its task waker with the Reactor
    pub fn modify(&mut self, key: RawFd, events: i32, cx: &mut Context) -> Result<()> {
        self.poller
            .modify(key, (libc::EPOLLONESHOT | libc::EPOLLET | events) as u32)?;
        self.wakers.insert(key, cx.waker().clone()); // Associate the waker with the file descriptor
        Ok(())
    }

    // Function to remove a file descriptor and its associated task waker from the Reactor
    pub fn remove(&mut self, key: RawFd) {
        self.wakers.remove(&key); // Remove the waker associated with the file descriptor
        self.poller.delete(key); // Delete the file descriptor from the poller
    }
}
