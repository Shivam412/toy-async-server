mod helpers;
mod net;

use helpers::Result;
use net::{SocketAddrV4, TcpListener};

fn main() -> Result<()> {
    // Start a new TCP listener.
    let listener = TcpListener::bind(SocketAddrV4::new([0, 0, 0, 0], 8000))?;
    println!("Started listening on address {:?}", listener.address());

    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                std::thread::spawn(move || handle_client(&mut stream, addr).unwrap());
            }
            Err(err) => eprintln!("{err}"),
        }
    }
}

fn handle_client(stream: &mut net::TcpStream, addr: SocketAddrV4) -> Result<()> {
    println!("Accepted the connection on {:?}", addr);
    let mut incoming = Vec::new();
    loop {
        let mut buff = [0_u8; 1024];
        // Read from the client.
        let len = stream.read(&mut buff)?;
        incoming.extend_from_slice(&buff[..len]);

        if incoming.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    let incoming = std::str::from_utf8(&incoming)?;
    println!("Got HTTP request:\n{}", incoming);
    stream.write(b"HTTP/1.1 200 OK\r\n")?;
    stream.write(b"\r\n")?;
    stream.write(b"Hello from Toy\r\n")?;

    Ok(())
}
