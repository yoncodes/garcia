#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PacketError {
    #[error("packet is {actual} bytes; minimum is {minimum}")]
    TooShort { minimum: usize, actual: usize },
    #[error("body is {actual} bytes; minimum is {minimum}")]
    InvalidBodySize { minimum: usize, actual: usize },
    #[error("packet length is {actual}; header declares {expected}")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("packet body is too large: {0} bytes")]
    TooLarge(usize),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum KcpInputError {
    #[error("KCP datagram is shorter than its header")]
    TooShort,
    #[error("KCP conversation does not match the session")]
    WrongConversation,
    #[error("KCP segment payload is truncated")]
    TruncatedPayload,
    #[error("unknown KCP command {0}")]
    UnknownCommand(u8),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum KcpSendError {
    #[error("KCP message is empty")]
    Empty,
    #[error("KCP message needs {fragments} fragments; maximum is 255")]
    TooLarge { fragments: usize },
}
