// Import necessary items
use crate::helpers::{get_error_message, Result};
use libc::{c_uint, c_ushort, c_void, sockaddr, sockaddr_in, AF_INET, IPPROTO_TCP, SOCK_STREAM};

// Define a structure for holding IPv4 socket address information
#[derive(Debug)]
pub struct SocketAddrV4 {
    octets: [u8; 4],
    port: u16,
}

impl SocketAddrV4 {
    // Create a new instance of SocketAddrV4
    pub fn new(octets: [u8; 4], port: u16) -> Self {
        Self { octets, port }
    }

    // Get the IPv4 address octets
    pub fn ip_octets(&self) -> [u8; 4] {
        self.octets
    }

    // Get the port number
    pub fn port(&self) -> u16 {
        self.port
    }
}

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
            let mut client_address: libc::sockaddr_in = std::mem::zeroed();
            let mut client_address_len = std::mem::size_of::<libc::sockaddr_in>() as c_uint;

            let client_socket = libc::accept(
                self.fd(),
                &mut client_address as *mut _ as *mut sockaddr,
                &mut client_address_len as *mut c_uint,
            ); // Syscall: accept(socket_fd, client_addr, client_addr_len)

            if client_socket == -1 {
                Err(get_error_message())?
            }

            Ok((
                TcpStream::new(client_socket),
                SocketAddrV4::new(
                    client_address.sin_addr.s_addr.to_be_bytes(),
                    client_address.sin_port,
                ),
            ))
        }
    }

    // Listen for incoming connections
    unsafe fn listen(fd: i32) -> Result<i32> {
        let result = libc::listen(fd, 10); // Syscall: listen(socket_fd, backlog)
        if result == -1 {
            libc::close(fd);
            panic!("Error start listening : {}", get_error_message());
        }
        Ok(result)
    }

    // Set up a socket for the listener
    unsafe fn setup_socket() -> Result<i32> {
        let fd = libc::socket(AF_INET, SOCK_STREAM, IPPROTO_TCP); // Syscall: socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
        if fd == -1 {
            panic!("Error setting up socket : {}", get_error_message());
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
            panic!("Error binding to address {}", get_error_message());
        }

        Ok(result)
    }
}

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
    fn get_client_fd(&self) -> i32 {
        self.client_fd
    }

    // Read data from the stream.
    pub fn read(&mut self, buff: &mut [u8]) -> Result<usize> {
        unsafe {
            let read_count = libc::read(
                self.get_client_fd(),
                buff as *mut _ as *mut c_void,
                buff.len(),
            );

            if read_count == -1 {
                Err(get_error_message())?
            }

            let data = String::from_utf8_lossy(&buff[..read_count as usize]);
            println!("[+] Client Sent: {data}");

            Ok(read_count as usize)
        }
    }

    // Write data to the stream.
    pub fn write(&mut self, buff: &[u8]) -> Result<usize> {
        unsafe {
            let write_count = libc::write(
                self.get_client_fd(),
                buff as *const _ as *const c_void,
                buff.len(),
            );

            if write_count == -1 {
                Err(get_error_message())?
            }

            Ok(write_count as usize)
        }
    }
}

impl Drop for TcpStream {
    // Close the socket when the stream is dropped
    fn drop(&mut self) {
        unsafe { libc::close(self.get_client_fd()) };
    }
}
