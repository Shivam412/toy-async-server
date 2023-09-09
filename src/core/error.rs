#[derive(Debug, Clone, PartialEq)]
pub enum IOError {
    WouldBlock,
    SyscallResult(String),
    ConnectionClosed,
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOError::WouldBlock => write!(f, "This operation would block."),
            IOError::SyscallResult(res) => write!(f, "{res}"),
            IOError::ConnectionClosed => write!(f, "Peer closed the connection."),
        }
    }
}
