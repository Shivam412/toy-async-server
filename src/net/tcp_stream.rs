use libc::c_void;

use crate::helpers::{get_error_message, Result};

// Define a structure representing a TCP stream
#[allow(dead_code)]
#[derive(Debug)]
pub struct TcpStream {
    client_fd: i32,
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
        unsafe {
            // Perform the read syscall and store the result in read_count.
            let read_count = libc::read(self.fd(), buff as *mut _ as *mut c_void, buff.len());

            // Check if the read operation was successful.
            if read_count == -1 {
                Err(get_error_message())?; // Return an error with the error message
            }

            Ok(read_count) // Return the number of bytes read
        }
    }

    // Write data to the stream.
    pub fn write(&mut self, buff: &[u8]) -> Result<usize> {
        unsafe {
            // Perform the write syscall and store the result in write_count.
            let write_count = libc::write(self.fd(), buff as *const _ as *const c_void, buff.len());

            // Check if the write operation was successful.
            if write_count == -1 {
                Err(get_error_message())?; // Return an error with the error message
            }

            Ok(write_count as usize) // Return the number of bytes written
        }
    }
}

impl Drop for TcpStream {
    // Close the socket when the stream is dropped
    fn drop(&mut self) {
        unsafe { libc::close(self.fd()) };
    }
}
