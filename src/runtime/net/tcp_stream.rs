use std::future::Future;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::error::IOError;
use crate::core::result::Result;
use crate::net;
use crate::runtime::reactor::REACTOR;

// Struct representing a TCP stream
pub struct TcpStream {
    inner: net::TcpStream,
}

impl TcpStream {
    // Constructor to create a TcpStream and register it with the reactor
    pub fn new(stream: net::TcpStream) -> TcpStream {
        // Register the stream with the reactor for both read and write events
        REACTOR.with(|current| {
            let current = current.borrow();
            current
                .register(stream.as_raw_fd(), libc::EPOLLIN | libc::EPOLLOUT)
                .unwrap();
        });

        TcpStream { inner: stream }
    }

    // Function to initiate a read operation on the stream
    pub fn read<'a>(&'a mut self, buff: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture {
            stream: &mut self.inner,
            buff: buff.as_mut(),
        }
    }

    // Function to initiate a write operation on the stream
    pub fn write<'a>(&'a mut self, buff: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture {
            stream: &mut self.inner,
            buff: buff.as_ref(),
        }
    }
}

// Future for handling asynchronous read operations
pub struct ReadFuture<'a> {
    stream: &'a mut net::TcpStream,
    buff: &'a mut [u8],
}

impl<'a> Future for ReadFuture<'a> {
    type Output = Result<isize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();

        match state.stream.read(&mut state.buff) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e == IOError::WouldBlock => {
                // Re-register with the reactor to wait for read events
                REACTOR.with(|current| {
                    current
                        .borrow_mut()
                        .modify(state.stream.as_raw_fd(), libc::EPOLLIN, cx)
                        .unwrap();
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

// Future for handling asynchronous write operations
pub struct WriteFuture<'a> {
    stream: &'a mut net::TcpStream,
    buff: &'a [u8],
}

impl<'a> Future for WriteFuture<'a> {
    type Output = Result<isize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();

        match state.stream.write(&state.buff) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e == IOError::WouldBlock => {
                // Re-register with the reactor to wait for write events
                REACTOR.with(|current| {
                    current
                        .borrow_mut()
                        .modify(state.stream.as_raw_fd(), libc::EPOLLOUT, cx)
                        .unwrap();
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

// Implementation of AsRawFd for TcpStream
impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

// Implementation of AsFd for TcpStream
impl AsFd for TcpStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

// Drop implementation to remove the TcpStream from the reactor on destruction
impl Drop for TcpStream {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            let mut current = current.borrow_mut();
            current.remove(self.inner.fd())
        })
    }
}
