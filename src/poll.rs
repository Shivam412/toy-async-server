use crate::helpers::{get_error_message, Result};

use libc::{c_int, epoll_event, EPOLL_CTL_ADD, EPOLL_CTL_DEL};

pub struct Epoll {
    fd: c_int,
}

impl Epoll {
    // Create a new instance of Epoll
    pub fn new() -> Result<Epoll> {
        // Create a new epoll instance using epoll_create1 syscall
        let fd = unsafe { libc::epoll_create1(0) };

        // Check if the epoll creation was successful
        if fd == -1 {
            return Err(get_error_message())?;
        }

        Ok(Epoll { fd })
    }

    // Add a file descriptor to the epoll instance
    pub fn add_fd(&self, target_fd: i32, events: u32) -> Result<()> {
        // Create an epoll_event struct with the specified events and target_fd
        let mut event = libc::epoll_event {
            events: events,
            u64: target_fd as u64,
        };

        // Add the file descriptor to the epoll instance using epoll_ctl syscall
        let result =
            unsafe { libc::epoll_ctl(self.fd, EPOLL_CTL_ADD, target_fd as c_int, &mut event) };

        // Check if adding the file descriptor was successful
        if result == -1 {
            return Err(get_error_message())?;
        }

        Ok(())
    }

    // Remove a file descriptor from the epoll instance
    pub fn delete(&self, fd: i32) {
        unsafe {
            let mut event: epoll_event = std::mem::zeroed();
            // Remove the file descriptor from the epoll instance and close it
            libc::epoll_ctl(self.fd, EPOLL_CTL_DEL, fd, &mut event);
            libc::close(fd);
        }
    }

    // Wait for events on the epoll instance
    pub fn wait(&self) -> Result<Vec<epoll_event>> {
        // Create an array of epoll_event structs to store events
        let mut events: [epoll_event; 1024] = unsafe { std::mem::zeroed() };

        // Wait for events using epoll_wait syscall
        let num_events = unsafe { libc::epoll_wait(self.fd, events.as_mut_ptr(), 1024, -1) };

        // Check if epoll_wait was successful
        if num_events == -1 {
            return Err(get_error_message())?;
        }

        // Convert the array of events to a Vec and return it
        Ok(events[..num_events as usize].to_vec())
    }
}
