use std::future::Future;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::error::IOError;
use crate::core::result::Result;
use crate::net::{self, SocketAddrV4};
use crate::runtime::reactor::REACTOR;

use super::tcp_stream::TcpStream;

// Struct representing a TCP listener
pub struct TcpListener {
    inner: net::TcpListener,
}

impl TcpListener {
    // Constructor to create a TcpListener and bind it to a specific address
    pub fn bind(addr: SocketAddrV4) -> Result<TcpListener> {
        // Bind a network listener to the provided address
        let listener = net::TcpListener::bind(addr)?;

        // Set the listener to non-blocking mode
        listener.set_nonblocking()?;

        // Register the listener with the reactor for event handling
        REACTOR.with(|current| {
            let current = current.borrow();
            current
                .register(listener.as_raw_fd(), libc::EPOLLIN)
                .unwrap();
        });

        // Return the TcpListener
        Ok(TcpListener { inner: listener })
    }

    // Function to initiate an accept operation on the listener
    pub fn accept(&self) -> Accept {
        Accept {
            listener: &self.inner,
        }
    }
}

// Future for handling asynchronous accept operations
impl<'listener> Future for Accept<'listener> {
    type Output = Result<(TcpStream, SocketAddrV4)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.accept() {
            Ok((stream, addr)) => Poll::Ready(Ok((TcpStream::new(stream), addr))),
            Err(err) if err == IOError::WouldBlock => {
                println!("[accept] listener would block, pause the execution");

                // Modify the reactor to wait for new events on the listener
                REACTOR.with(|reactor| {
                    let mut reactor = reactor.borrow_mut();
                    reactor
                        .modify(self.listener.as_raw_fd(), libc::EPOLLIN, cx)
                        .unwrap();
                });

                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

// Struct representing an accept operation
pub struct Accept<'listener> {
    listener: &'listener net::TcpListener,
}

// Implementation of AsRawFd for TcpListener
impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

// Implementation of AsFd for TcpListener
impl AsFd for TcpListener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

// Drop implementation to remove the TcpListener from the reactor on destruction
impl Drop for TcpListener {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            let mut current = current.borrow_mut();
            current.remove(self.inner.as_raw_fd());
        });
    }
}
