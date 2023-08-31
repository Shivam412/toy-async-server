mod helpers;
mod net;
mod poll;

use helpers::Result;
use net::{SocketAddrV4, TcpListener};
use poll::Epoll;
use std::collections::HashMap;

use libc::{c_int, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLLRDHUP};

use crate::net::TcpStream;

fn main() -> Result<()> {
    // Start a new TCP listener.
    let listener = TcpListener::bind(SocketAddrV4::new([0, 0, 0, 0], 8000))?;
    println!("Started listening on address {:?}", listener.address());

    // Set the socket Non-Blocking.
    listener.set_nonblocking()?;

    // Create an Epoll instance.
    let epoll = Epoll::new()?;

    // Add the listener socket to the epoll instance.
    epoll.add_fd(listener.fd(), (EPOLLIN | EPOLLOUT | EPOLLET) as u32)?;

    // HashMap to save the streams.
    let mut streams: HashMap<c_int, TcpStream> = HashMap::new();

    loop {
        // Wait for epoll events
        let events = epoll.wait()?;

        // Process each event from epoll
        for event in events {
            if event.events & EPOLLIN as u32 != 0 {
                // If the event is an incoming data event, handle it
                if event.u64 == listener.fd() as u64 {
                    // Accept a new client connection.
                    let (stream, addr) = listener.accept()?;
                    println!("Accepted a new client connection from: {:?}", addr);
                    // Add the stream socket to epoll for events.
                    epoll.add_fd(
                        stream.fd(),
                        (EPOLLIN | EPOLLET | EPOLLRDHUP | EPOLLHUP) as u32,
                    )?;
                    // Add the stream to the hashmap.
                    streams.insert(stream.fd(), stream);
                } else if let Some(stream) = streams.get_mut(&(event.u64 as c_int)) {
                    // Handle data read/write for existing client stream.
                    handle_client(stream)?;
                }
            }
            // Check if the client closed the connection.
            if event.events & (EPOLLRDHUP | EPOLLHUP) as u32 != 0 {
                epoll.delete(event.u64 as c_int);
                println!("Closed the connection");
                streams.remove(&(event.u64 as c_int));
            }
        }
    }
}

// Handle client to read or write on a stream.
fn handle_client(stream: &mut net::TcpStream) -> Result<()> {
    let mut incoming = Vec::new();
    loop {
        let mut buff = [0_u8; 1024];
        // Read from the client.
        match stream.read(&mut buff) {
            Ok(count) if count > 0 => {
                incoming.extend_from_slice(&buff[..count as usize]);

                if let Err(err) = stream.write(&buff) {
                    eprintln!("Error writing to client: {err}");
                }
            }
            Ok(_) | Err(_) => break,
        }
    }

    Ok(())
}
