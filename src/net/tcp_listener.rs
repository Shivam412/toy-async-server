use super::{socketv4::SocketAddrV4, tcp_stream::TcpStream};
use crate::helpers::{get_error_message, Result};

use libc::{
    c_uint, c_ushort, sockaddr, sockaddr_in, AF_INET, F_GETFL, F_SETFL, IPPROTO_TCP, O_NONBLOCK,
    SOCK_NONBLOCK, SOCK_STREAM,
};

// Define a structure representing a TCP listener
#[derive(Debug)]
pub struct TcpListener {
    addr: SocketAddrV4,
    fd: i32,
}

impl TcpListener {
    // Get the address the listener is bound to
    pub fn address(&self) -> &SocketAddrV4 {
        &self.addr
    }

    // Get the file descriptor of the listener
    pub fn fd(&self) -> i32 {
        self.fd
    }

    // Create a new TCP listener and bind it to the specified address
    pub fn bind(addr: SocketAddrV4) -> Result<TcpListener> {
        unsafe {
            let fd = TcpListener::setup_socket()?; // Syscall: socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
            TcpListener::bind_to_address(fd, &addr)?; // Syscall: bind(socket_fd, sockaddr, sockaddr_len)
            TcpListener::listen(fd)?; // Syscall: listen(socket_fd, backlog)
            Ok(TcpListener { addr, fd })
        }
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
                Err(format!(
                    "Error accepting connection: {}",
                    get_error_message()
                ))?
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
                panic!("Error getting socket flags: {}", get_error_message());
            }

            // Add the O_NONBLOCK flag to the current flags
            let result = libc::fcntl(self.fd(), F_SETFL, flags | O_NONBLOCK);

            // Check if setting flags was successful
            if result == -1 {
                libc::close(self.fd());
                panic!(
                    "Error setting socket to non-blocking: {}",
                    get_error_message()
                );
            }

            Ok(()) // Return Ok if successful
        }
    }

    // Listen for incoming connections
    unsafe fn listen(fd: i32) -> Result<i32> {
        // Start listening on the specified socket file descriptor with a backlog of 10
        let result = libc::listen(fd, 10); // Syscall: listen(socket_fd, backlog)

        // Check if the listen call was successful
        if result == -1 {
            libc::close(fd);
            panic!("Error starting listening: {}", get_error_message());
        }

        Ok(result)
    }

    // Set up a socket for the listener
    unsafe fn setup_socket() -> Result<i32> {
        // Create a new socket of type AF_INET (IPv4) and SOCK_STREAM (TCP)
        let fd = libc::socket(AF_INET, SOCK_STREAM, IPPROTO_TCP); // Syscall: socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)

        // Check if the socket creation was successful
        if fd == -1 {
            panic!("Error setting up socket: {}", get_error_message());
        }

        Ok(fd)
    }

    // Bind the socket to the specified address
    unsafe fn bind_to_address(socket_fd: i32, addr: &SocketAddrV4) -> Result<i32> {
        // Convert octets to address.
        let s_addr = u32::from_be_bytes(addr.ip_octets());
        // Create a sockaddr_in.
        let mut address: sockaddr_in = std::mem::zeroed();
        address.sin_addr.s_addr = s_addr;
        address.sin_port = addr.port().to_be();
        address.sin_family = AF_INET as c_ushort;

        let result = libc::bind(
            socket_fd,
            &address as *const _ as *const sockaddr,
            std::mem::size_of::<sockaddr_in>() as c_uint,
        ); // Syscall: bind(socket_fd, sockaddr, sockaddr_len)

        if result == -1 {
            libc::close(socket_fd);
            panic!("Error binding to address: {}", get_error_message());
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
