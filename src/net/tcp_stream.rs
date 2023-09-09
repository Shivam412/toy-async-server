use crate::core::{error::IOError, os, result::Result};
use libc::c_void;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};

// Define a structure representing a TCP stream
#[derive(Debug)]
pub struct TcpStream {
    client_fd: i32, // File descriptor representing the TCP stream
}

impl TcpStream {
    // Create a new TCP stream from a file descriptor
    pub fn new(fd: i32) -> TcpStream {
        TcpStream { client_fd: fd }
    }

    // Get the client's file descriptor
    pub fn fd(&self) -> i32 {
        self.client_fd
    }

    // Read data from the stream.
    pub fn read(&mut self, buff: &mut [u8]) -> Result<isize> {
        // Read data from the stream into the provided buffer.
        // Perform the read syscall and store the result in read_count.
        let read_count =
            unsafe { libc::read(self.fd(), buff as *mut _ as *mut c_void, buff.len()) };

        // Check if the client closed the connection (read count = 0)
        if read_count == 0 {
            return Err(IOError::ConnectionClosed);
        }

        // Check if the read operation was successful.
        if read_count == -1 {
            let errno = os::OS::err_no();
            if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                return Err(IOError::WouldBlock);
            }
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(read_count) // Return the number of bytes read
    }

    // Write data to the stream.
    pub fn write(&mut self, buff: &[u8]) -> Result<isize> {
        // Perform the write syscall and store the result in write_count.
        let write_count =
            unsafe { libc::write(self.fd(), buff as *const _ as *const c_void, buff.len()) };

        // Check if the write operation was successful.
        if write_count == -1 {
            let errno = os::OS::err_no();
            if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                return Err(IOError::WouldBlock);
            }
            return Err(IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(write_count) // Return the number of bytes written
    }
}

// Implement the AsRawFd trait for TcpStream
impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.fd()
    }
}

// Implement the AsFd trait for TcpStream
impl AsFd for TcpStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

// Implement the Drop trait for TcpStream to close the socket when dropped
impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd()) }; // Close the socket
    }
}
