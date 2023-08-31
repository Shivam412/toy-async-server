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
