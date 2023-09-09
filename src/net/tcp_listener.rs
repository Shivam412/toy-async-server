use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};

use super::{socketv4::SocketAddrV4, tcp_stream::TcpStream};
use crate::core::{
    error::{self, IOError},
    os,
    result::Result,
};

use libc::{
    c_uint, c_ushort, sockaddr, sockaddr_in, AF_INET, F_GETFL, F_SETFL, IPPROTO_TCP, O_NONBLOCK,
    SOCK_NONBLOCK, SOCK_STREAM,
};

// Define a structure representing a TCP listener
#[derive(Debug)]
pub struct TcpListener {
    fd: i32,
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl AsFd for TcpListener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl TcpListener {
    // Get the file descriptor of the listener
    fn fd(&self) -> i32 {
        self.fd
    }

    // Create a new TCP listener and bind it to the specified address
    pub fn bind(addr: SocketAddrV4) -> Result<TcpListener> {
        let fd = TcpListener::setup_socket()?; // Syscall: socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
        let listener = TcpListener { fd };
        listener.bind_to_address(addr)?; // Syscall: bind(socket_fd, sockaddr, sockaddr_len)
        listener.listen()?; // Syscall: listen(socket_fd, backlog)
        Ok(listener)
    }

    // Accept a new incoming connection and return a TcpStream and client address
    pub fn accept(&self) -> Result<(TcpStream, SocketAddrV4)> {
        unsafe {
            // Initialize client address structure with zeros
            let mut client_address: libc::sockaddr_in = std::mem::zeroed();

            // Calculate the size of the client address structure
            let mut client_address_len = std::mem::size_of::<libc::sockaddr_in>() as c_uint;

            // Accept a new connection with non-blocking option using accept4 syscall
            let client_socket = libc::accept4(
                self.fd(),
                &mut client_address as *mut _ as *mut sockaddr,
                &mut client_address_len as *mut c_uint,
                SOCK_NONBLOCK,
            ); // Syscall: accept(socket_fd, client_addr, client_addr_len)

            // Check if the accept call was successful
            if client_socket == -1 {
                let errno = os::OS::err_no();
                if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                    return Err(IOError::WouldBlock);
                }
                return Err(IOError::SyscallResult(os::OS::err_msg()));
            }

            // Create a new TcpStream from the accepted client socket
            // and construct the SocketAddrV4 from the client address
            Ok((
                TcpStream::new(client_socket),
                SocketAddrV4::new(
                    client_address.sin_addr.s_addr.to_be_bytes(),
                    client_address.sin_port,
                ),
            ))
        }
    }

    // Set the socket to non-blocking mode
    pub fn set_nonblocking(&self) -> Result<()> {
        unsafe {
            // Get the current flags for the socket
            let flags = libc::fcntl(self.fd(), F_GETFL, 0);

            // Check if getting flags was successful
            if flags == -1 {
                libc::close(self.fd());
                return Err(error::IOError::SyscallResult(os::OS::err_msg()));
            }

            // Add the O_NONBLOCK flag to the current flags
            let result = libc::fcntl(self.fd(), F_SETFL, flags | O_NONBLOCK);

            // Check if setting flags was successful
            if result == -1 {
                libc::close(self.fd());
                return Err(error::IOError::SyscallResult(os::OS::err_msg()));
            }

            Ok(()) // Return Ok if successful
        }
    }

    // Listen for incoming connections
    fn listen(&self) -> Result<i32> {
        // Start listening on the specified socket file descriptor with a backlog of 10
        let result = unsafe { libc::listen(self.fd(), 10) }; // Syscall: listen(socket_fd, backlog)

        // Check if the listen call was successful
        if result == -1 {
            unsafe { libc::close(self.fd()) };
            return Err(error::IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(result)
    }

    // Set up a socket for the listener
    fn setup_socket() -> Result<i32> {
        // Create a new socket of type AF_INET (IPv4) and SOCK_STREAM (TCP)
        let fd = unsafe { libc::socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) }; // Syscall: socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)

        // Check if the socket creation was successful
        if fd == -1 {
            return Err(error::IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(fd)
    }

    // Bind the socket to the specified address
    fn bind_to_address(&self, addr: SocketAddrV4) -> Result<i32> {
        // Convert octets to address.
        let s_addr = u32::from_be_bytes(addr.ip_octets());
        // Create a sockaddr_in.
        let mut address: sockaddr_in = unsafe { std::mem::zeroed() };
        address.sin_addr.s_addr = s_addr;
        address.sin_port = addr.port().to_be();
        address.sin_family = AF_INET as c_ushort;

        let result = unsafe {
            libc::bind(
                self.fd(),
                &address as *const _ as *const sockaddr,
                std::mem::size_of::<sockaddr_in>() as c_uint,
            )
        }; // Syscall: bind(socket_fd, sockaddr, sockaddr_len)

        if result == -1 {
            unsafe { libc::close(self.fd()) };
            return Err(error::IOError::SyscallResult(os::OS::err_msg()));
        }

        Ok(result)
    }
}

impl Drop for TcpListener {
    // Close the socket when the stream is dropped
    fn drop(&mut self) {
        unsafe { libc::close(self.fd()) };
    }
}
