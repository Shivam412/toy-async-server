use libc::{epoll_event, EPOLLIN, EPOLLOUT, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
use std::os::fd::RawFd;

use crate::core::{error::IOError, os, result::Result};

#[derive(Clone)]
pub struct Event {
    pub key: RawFd,
    pub readable: bool,
    pub writable: bool,
}

pub struct Poller {
    epoll_fd: RawFd,
}

impl Poller {
    pub fn new() -> Result<Poller> {
        let epoll_fd = unsafe { libc::epoll_create1(0) };

        if epoll_fd == -1 {
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(Poller { epoll_fd })
    }

    pub fn add(&self, fd: RawFd, events: u32) -> Result<()> {
        let mut event = libc::epoll_event {
            events: events,
            u64: fd as u64,
        };

        let result = unsafe { libc::epoll_ctl(self.epoll_fd, EPOLL_CTL_ADD, fd, &mut event) };

        if result == -1 {
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(())
    }

    pub fn modify(&self, fd: RawFd, events: u32) -> Result<()> {
        let mut event = libc::epoll_event {
            events: events,
            u64: fd as u64,
        };

        let result = unsafe { libc::epoll_ctl(self.epoll_fd, EPOLL_CTL_MOD, fd, &mut event) };

        if result == -1 {
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(())
    }

    pub fn delete(&self, fd: RawFd) {
        unsafe {
            let mut event: epoll_event = std::mem::zeroed();
            libc::epoll_ctl(self.epoll_fd, EPOLL_CTL_DEL, fd, &mut event);
        }
    }

    pub fn wait(&self) -> Result<Vec<Event>> {
        let mut events: [epoll_event; 1024] = unsafe { std::mem::zeroed() };
        let num_events = unsafe { libc::epoll_wait(self.epoll_fd, events.as_mut_ptr(), 1024, -1) };

        if num_events == -1 {
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(events
            .map(|event| Event {
                key: event.u64 as RawFd,
                readable: event.events & EPOLLIN as u32 != 0,
                writable: event.events & EPOLLOUT as u32 != 0,
            })
            .to_vec())
    }
}
