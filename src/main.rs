mod core;
mod net;
mod runtime;

use net::SocketAddrV4;
use runtime::{executor, TcpListener, TcpStream};

use crate::core::result::Result;

// Entry point of the application
fn main() -> Result<()> {
    // Run the asynchronous code block using the executor
    executor::block_on(async {
        // Define the address to listen on (e.g., 0.0.0.0:8000)
        let addr = SocketAddrV4::new([0, 0, 0, 0], 8000);

        // Create a TCP listener bound to the specified address
        let listener = TcpListener::bind(addr).unwrap();
        println!("[main] Started listening on {:?}", addr);

        // Accept incoming connections and handle them asynchronously
        loop {
            let (mut stream, addr) = listener.accept().await.unwrap();

            // Spawn a new asynchronous task to handle the client
            executor::spawn(async move {
                if let Err(err) = handle_client(&mut stream, addr).await {
                    eprintln!("[main] Error occurred while handling client: {}", err);
                }
            });
        }
    })
}

// Asynchronously handle a client connection
async fn handle_client(stream: &mut TcpStream, addr: SocketAddrV4) -> Result<()> {
    println!("[handle_client] Got connection on {:?}", addr);

    let mut incoming = vec![];

    loop {
        // Read data from the client in chunks
        let mut buf = vec![0u8; 1024];
        let read = stream.read(&mut buf).await?;
        incoming.extend_from_slice(&buf[..usize::try_from(read).unwrap()]);

        // Check for the end of the HTTP request
        if incoming.len() > 4 && &incoming[incoming.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }

    // Print the received HTTP request
    println!(
        "[handle_client] Got HTTP request:\n{}",
        String::from_utf8_lossy(&incoming)
    );

    // Send an HTTP response
    stream.write(b"HTTP/1.1 200 OK\r\n").await?;
    stream.write(b"\r\n").await?;
    stream.write(b"Hello from plaque!\n").await?;

    // Close the connection
    println!("[handle_client] Closing connection for {:?}\n", addr);
    Ok(())
}
